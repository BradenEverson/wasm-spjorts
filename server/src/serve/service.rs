//! Hyper service implementation

use std::{fs::File, future::Future, io::Read, pin::Pin, sync::Arc};

use deku::DekuContainerRead;
use futures::{stream::SplitSink, StreamExt};
use http_body_util::Full;
use hyper::{
    body::{self, Bytes},
    service::Service,
    upgrade::Upgraded,
    Method, Request, Response, StatusCode,
};
use hyper_tungstenite::is_upgrade_request;
use hyper_util::rt::TokioIo;
use tokio::sync::{mpsc::Sender, Mutex};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};
use url::Url;

use crate::{
    control::{msg::WsMessage, Controller},
    serve::{registry::GAMES, SpjortState, WsConnectionType},
};

use super::registry::render_id_connection;

/// Web socket write stream
pub type WebsocketWriteStream = SplitSink<WebSocketStream<TokioIo<Upgraded>>, Message>;

/// Service implementation responsible for handling routes and updating new controller connections
pub struct SpjortService {
    /// Controller send channel for connecting devices
    controller_sender: Sender<Arc<Mutex<Controller>>>,
    /// The current state
    state: Arc<Mutex<SpjortState>>,
}

impl SpjortService {
    /// Creates a new spjort service wrapping a controller sender
    pub fn new(
        controller_sender: Sender<Arc<Mutex<Controller>>>,
        state: Arc<Mutex<SpjortState>>,
    ) -> Self {
        Self {
            controller_sender,
            state,
        }
    }
}

async fn handle_ws_binary(
    buf: &[u8],
    controller_type: &mut WsConnectionType,
    sender: Sender<Arc<Mutex<Controller>>>,
    state: Arc<Mutex<SpjortState>>,
    write_stream: Arc<Mutex<WebsocketWriteStream>>,
) {
    match controller_type {
        WsConnectionType::Controller(id) => {
            match buf[0] {
                0x05 => {
                    // Controller ID wants to be paired
                    {
                        state.lock().await.set_pairing_id(*id);
                    }
                }
                _ => {
                    let controller = &state.lock().await.controllers[&id];
                    let mut controller = controller.lock().await;
                    controller.broadcast(buf).await
                }
            }
        }
        WsConnectionType::None => {
            let (_, val) = WsMessage::from_bytes((buf, 0)).unwrap();
            match val {
                WsMessage::Controller(id) => {
                    *controller_type = WsConnectionType::Controller(id);
                    let new_controller = Arc::new(Mutex::new(Controller::new(id)));
                    sender
                        .send(new_controller)
                        .await
                        .expect("Send new controller");
                }
                WsMessage::Establish(id) => {
                    *controller_type = WsConnectionType::Listener(id);
                    let controller = &state.lock().await.controllers[&id];
                    let mut controller = controller.lock().await;
                    controller.new_listener(write_stream);
                }
            }
        }
        WsConnectionType::Listener(_) => {
            unreachable!("Listeners should only listen")
        }
    }
}

impl Service<Request<body::Incoming>> for SpjortService {
    type Response = Response<Full<Bytes>>;
    type Error = hyper::http::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, mut req: Request<body::Incoming>) -> Self::Future {
        if is_upgrade_request(&req) {
            let (response, websocket) =
                hyper_tungstenite::upgrade(&mut req, None).expect("Upgrade to WebSocket");

            let mut controller_type = WsConnectionType::None;
            let sender = self.controller_sender.clone();
            let state = self.state.clone();
            tokio::spawn(async move {
                let (ws_write, mut ws_read) = websocket.await.expect("Await websocket").split();
                let ws_write = Arc::new(Mutex::new(ws_write));
                while let Some(Ok(msg)) = ws_read.next().await {
                    match msg {
                        Message::Binary(buf) => {
                            handle_ws_binary(
                                &buf,
                                &mut controller_type,
                                sender.clone(),
                                state.clone(),
                                ws_write.clone(),
                            )
                            .await
                        }
                        _ => {}
                    }
                }
            });

            Box::pin(async { Ok(response) })
        } else {
            let mut response = Response::builder();

            let res = match *req.method() {
                Method::GET => match req.uri().path() {
                    "/" => {
                        let mut buf = vec![];
                        let mut page =
                            File::open("frontend/index.html").expect("Failed to find file");
                        page.read_to_end(&mut buf)
                            .expect("Failed to read to buffer");
                        response
                            .status(StatusCode::OK)
                            .body(Full::new(Bytes::copy_from_slice(&buf)))
                    }
                    "/game" => {
                        let mut buf = vec![];
                        let mut page =
                            File::open("frontend/game.html").expect("Failed to find file");
                        page.read_to_end(&mut buf)
                            .expect("Failed to read to buffer");
                        response
                            .status(StatusCode::OK)
                            .body(Full::new(Bytes::copy_from_slice(&buf)))
                    }
                    "/games" => {
                        let games = GAMES
                            .iter()
                            .map(|game| game.render_html())
                            .collect::<Vec<_>>()
                            .join(" ");
                        response
                            .header("content-type", "application/json")
                            .status(StatusCode::OK)
                            .body(Full::new(Bytes::copy_from_slice(games.as_bytes())))
                    }
                    "/controllers" => {
                        let ids = {
                            futures::executor::block_on(self.state.lock()).get_pairing_devices()
                        };
                        let controller_ids = ids
                            .iter()
                            .map(|id| render_id_connection(*id))
                            .collect::<Vec<_>>()
                            .join(" ");
                        response
                            .header("content-type", "application/json")
                            .status(StatusCode::OK)
                            .body(Full::new(Bytes::copy_from_slice(controller_ids.as_bytes())))
                    }
                    "/favicon.ico" => {
                        let mut buf = vec![];
                        let mut page =
                            File::open("frontend/favicon.ico").expect("Failed to find file");
                        page.read_to_end(&mut buf)
                            .expect("Failed to read to buffer");
                        response
                            .status(StatusCode::OK)
                            .body(Full::new(Bytes::copy_from_slice(&buf)))
                    }
                    "/connect" => {
                        let uri = req.uri().to_string();
                        let request_url =
                            Url::parse(&format!("https://dumbfix.com/{}", uri)).unwrap();
                        let potential_id = request_url.query_pairs().find(|(key, _)| key == "id");
                        if let Some((_, id)) = potential_id {
                            if let Ok(id) = id.parse() {
                                let id_exists = {
                                    futures::executor::block_on(self.state.lock())
                                        .connect_controller(id)
                                };

                                if id_exists {
                                    let res = response
                                        .header("content-type", "application/json")
                                        .status(StatusCode::OK)
                                        .body(Full::new(Bytes::copy_from_slice(b"true")));

                                    return Box::pin(async { res });
                                }
                            }
                        }

                        response
                            .header("content-type", "application/json")
                            .status(StatusCode::OK)
                            .body(Full::new(Bytes::copy_from_slice(b"false")))
                    }
                    fs if fs.starts_with("/frontend/") || fs.starts_with("/wasm") => {
                        let mut buf = vec![];
                        let mut page = File::open(&fs[1..]).expect("Failed to find file");
                        page.read_to_end(&mut buf)
                            .expect("Failed to read to buffer");
                        if fs.starts_with("/wasm") {
                            if fs.ends_with("js") {
                                response = response.header("content-type", "text/javascript");
                            } else if fs.ends_with("wasm") {
                                response = response.header("content-type", "application/wasm");
                            }
                        }

                        response
                            .status(StatusCode::OK)
                            .body(Full::new(Bytes::copy_from_slice(&buf)))
                    }
                    game if game.starts_with("/sports/") => {
                        let game = GAMES
                            .iter()
                            .find(|g| game.contains(g.name))
                            .expect("Valid game from query");
                        let game = game.render_game_scene();
                        response
                            .status(StatusCode::OK)
                            .body(Full::new(Bytes::copy_from_slice(game.as_bytes())))
                    }
                    _ => response
                        .status(StatusCode::NOT_FOUND)
                        .body(Full::new(Bytes::from_static(b"Not Found"))),
                },
                _ => response
                    .status(StatusCode::METHOD_NOT_ALLOWED)
                    .body(Full::new(Bytes::from_static(b"Method Not Allowed"))),
            };

            Box::pin(async { res })
        }
    }
}
