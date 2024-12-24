//! Hyper service implementation

use std::{fs::File, future::Future, io::Read, pin::Pin};

use http_body_util::Full;
use hyper::{
    body::{self, Bytes},
    service::Service,
    Method, Request, Response, StatusCode,
};
use hyper_tungstenite::is_upgrade_request;
use tokio::sync::mpsc::Sender;

use crate::control::Controller;

/// Service implementation responsible for handling routes and updating new controller connections
pub struct SpjortService {
    /// Controller send channel for connecting devices
    controller_sender: Sender<Controller>,
}

impl SpjortService {
    /// Creates a new spjortservice wrapping a controller sender
    pub fn new(controller_sender: Sender<Controller>) -> Self {
        Self { controller_sender }
    }
}

impl Service<Request<body::Incoming>> for SpjortService {
    type Response = Response<Full<Bytes>>;
    type Error = hyper::http::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<body::Incoming>) -> Self::Future {
        if is_upgrade_request(&req) {
            todo!()
        } else {
            let response = Response::builder();

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
                    "/games" => {
                        println!("");
                        let mut buf = vec![];
                        let mut page =
                            File::open("frontend/index.html").expect("Failed to find file");
                        page.read_to_end(&mut buf)
                            .expect("Failed to read to buffer");
                        response
                            .status(StatusCode::OK)
                            .body(Full::new(Bytes::copy_from_slice(&buf)))
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
