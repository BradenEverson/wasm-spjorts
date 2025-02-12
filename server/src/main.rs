//! Main server driver for the project. Maintains controller connections via web-socket and can
//! connect them to active web sessions. Aside from that however the only other thing managed by
//! the site itself is *what* game the controller is currently in (there is no user data, all is
//! linked and contained via controller). The game logic itself is handled in WASM on the frontend

use std::sync::Arc;

use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use server::serve::{service::SpjortService, SpjortState};
use tokio::{net::TcpListener, sync::Mutex};

/// How many controller connections are allowed to be queued
pub const CONTROLLER_QUEUE_LIMIT: usize = 15;

#[tokio::main]
async fn main() {
    let (state, controller_write, mut controller_read) = SpjortState::new(15);
    let state = Arc::new(Mutex::new(state));

    let listener = TcpListener::bind("0.0.0.0:7878")
        .await
        .expect("Failed to bind to server");

    println!("🏂🎾⛳");
    println!("Listening on http://localhost:7878");

    let state_clone_server = state.clone();
    tokio::spawn(async move {
        loop {
            let (socket, _) = listener
                .accept()
                .await
                .expect("Failed to accept connection");

            let io = TokioIo::new(socket);

            let service = SpjortService::new(controller_write.clone(), state_clone_server.clone());
            tokio::spawn(async move {
                if let Err(e) = http1::Builder::new()
                    .serve_connection(io, service)
                    .with_upgrades()
                    .await
                {
                    eprintln!("Error serving connection: {}", e);
                }
            });
        }
    });

    // Connection handler thread
    while let Some(controller) = controller_read.recv().await {
        state.lock().await.connect(controller).await;
    }

    // TODO Later if controller persistence is really an issue
    // Dead controller disconnect loop :)
    /*loop {
        state.lock().await.heartbeat();
        std::thread::sleep(Duration::from_secs(30));
    }*/
}
