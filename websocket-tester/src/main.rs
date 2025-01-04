//! Simple program that connects as a controller and broadcasts random noise

use std::{f32::consts::PI, ops::Range, time::Duration};

use futures_util::{SinkExt, StreamExt};
use rand::{thread_rng, Rng};
use server::control::{msg::WsMessage, ControllerMessage};
use tokio_tungstenite::connect_async;

/// Range of all angles that encompass a unit circle (or I guess any circle)
pub const UNIT_CIRCLE_RANGE: Range<f32> = 0f32..(PI * 2f32);

#[tokio::main]
async fn main() {
    let (ws, _) = connect_async("ws://localhost:7878")
        .await
        .expect("Connect to ws");

    let (mut write, _) = ws.split();
    write
        .send(
            WsMessage::Controller(1)
                .to_ws_message()
                .expect("Convert to ws message"),
        )
        .await
        .unwrap();
    let mut rng = thread_rng();
    loop {
        /*let (pitch, yaw, roll) = (
            rng.gen_range(UNIT_CIRCLE_RANGE),
            rng.gen_range(UNIT_CIRCLE_RANGE),
            rng.gen_range(UNIT_CIRCLE_RANGE),
        );*/

        let (pitch, yaw, roll) = (2.0 * PI, 2.0 * PI, 2.0 * PI);

        write
            .send(
                ControllerMessage::AngleInfo(pitch, yaw, roll)
                    .to_ws_message()
                    .unwrap(),
            )
            .await
            .unwrap();

        std::thread::sleep(Duration::from_millis(100));

        let (pitch, yaw, roll) = (0., 0., 0.);

        write
            .send(
                ControllerMessage::AngleInfo(pitch, yaw, roll)
                    .to_ws_message()
                    .unwrap(),
            )
            .await
            .unwrap();

        write
            .send(
                ControllerMessage::ButtonPressA
                    .to_ws_message()
                    .expect("Convert to ws message"),
            )
            .await
            .unwrap();

        std::thread::sleep(Duration::from_millis(1000));

        write
            .send(
                ControllerMessage::ButtonPressB
                    .to_ws_message()
                    .expect("Convert to ws message"),
            )
            .await
            .unwrap();

        std::thread::sleep(Duration::from_millis(1000));
    }
}
