//! Main server driver for the project. Maintains controller connections via web-socket and can
//! connect them to active web sessions. Aside from that however the only other thing managed by
//! the site itself is *what* game the controller is currently in (there is no user data, all is
//! linked and contained via controller). The game logic itself is handled in WASM on the frontend

use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use server::server::{service::SpjortService, SpjortState};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let mut state = SpjortState::default();
    let listener = TcpListener::bind("0.0.0.0:7878")
        .await
        .expect("Failed to bind to server");

    println!("🏂");
    println!("Listening on http://localhost:7878");

    loop {
        let (socket, _) = listener
            .accept()
            .await
            .expect("Failed to accept connection");

        let io = TokioIo::new(socket);

        let service = SpjortService;
        tokio::spawn(async move {
            if let Err(e) = http1::Builder::new().serve_connection(io, service).await {
                eprintln!("Error serving connection: {}", e);
            }
        });
    }
}
