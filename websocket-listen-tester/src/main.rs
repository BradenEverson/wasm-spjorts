//! Simple program that connects as a controller and reads random noise

use std::{f32::consts::PI, ops::Range};

use deku::DekuContainerRead;
use futures_util::{SinkExt, StreamExt};
use server::control::{msg::WsMessage, ControllerMessage};
use tokio_tungstenite::{connect_async, tungstenite::Message};

/// Range of all angles that encompass a unit circle (or I guess any circle)
pub const UNIT_CIRCLE_RANGE: Range<f32> = 0f32..(PI * 2f32);

#[tokio::main]
async fn main() {
    let (ws, _) = connect_async("ws://localhost:7878")
        .await
        .expect("Connect to ws");

    let (mut write, mut read) = ws.split();
    write
        .send(
            WsMessage::Establish(1)
                .to_ws_message()
                .expect("Convert to ws message"),
        )
        .await
        .unwrap();

    while let Some(Ok(msg)) = read.next().await {
        match msg {
            Message::Binary(bin) => {
                let message = ControllerMessage::from_bytes((bin.as_slice(), 0))
                    .expect("Read as controller message");

                println!("{message:?}");
            }
            _ => println!("Non binary message found"),
        }
    }
}
