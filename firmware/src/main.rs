//! Main firmware driver for a controller, reading rotational data and button press events from the
//! Pi and transmitting this information to the game server over web sockets

use futures_util::{SinkExt, StreamExt};
use rppal::gpio::{Gpio, Trigger};
use server::control::{msg::WsMessage, ControllerMessage};
use std::{fs::File, io::Read, sync::mpsc::channel, time::Duration};
use tokio_tungstenite::connect_async;

/// A Button's pin
pub const BUTTON_A_PIN: u8 = 5;
/// B Button's pin
pub const BUTTON_B_PIN: u8 = 6;

#[tokio::main]
async fn main() {
    let id = read_id();
    let (write_action, read_action) = channel();

    // Connect to server
    let (ws, _) = connect_async("ws://192.168.10.137:7878")
        .await
        .expect("Connect to ws");

    let (mut write, _) = ws.split();
    write
        .send(
            WsMessage::Controller(id)
                .to_ws_message()
                .expect("Convert to ws message"),
        )
        .await
        .unwrap();

    // Set up peripherals
    let gpio = Gpio::new().expect("Initialize GPIO");
    let mut button_a = gpio
        .get(BUTTON_A_PIN)
        .expect("Get GPIO pin for A button")
        .into_input_pulldown();

    let mut button_b = gpio
        .get(BUTTON_B_PIN)
        .expect("Get GPIO pin for B button")
        .into_input_pulldown();

    let channel_a = write_action.clone();
    button_a
        .set_async_interrupt(
            Trigger::FallingEdge,
            Some(Duration::from_millis(50)),
            move |_| {
                channel_a
                    .send(ControllerMessage::ButtonPressA)
                    .expect("Send button press A");
            },
        )
        .expect("Set interupt");

    let channel_b = write_action.clone();
    button_b
        .set_async_interrupt(
            Trigger::FallingEdge,
            Some(Duration::from_millis(50)),
            move |_| {
                channel_b
                    .send(ControllerMessage::ButtonPressB)
                    .expect("Send button press B");
            },
        )
        .expect("Set interupt");

    // Read gyroscope information and send it over to web socket
    while let Ok(msg) = read_action.recv() {
        match msg {
            ControllerMessage::ButtonPressA => {
                write
                    .send(
                        ControllerMessage::ButtonPressA
                            .to_ws_message()
                            .expect("Convert to ws message"),
                    )
                    .await
                    .unwrap();
            }
            ControllerMessage::ButtonPressB => {
                write
                    .send(
                        ControllerMessage::ButtonPressB
                            .to_ws_message()
                            .expect("Convert to ws message"),
                    )
                    .await
                    .unwrap();
            }
            ControllerMessage::AngleInfo(pitch, roll, yaw) => {
                write
                    .send(
                        ControllerMessage::AngleInfo(pitch, yaw, roll)
                            .to_ws_message()
                            .unwrap(),
                    )
                    .await
                    .unwrap();
            }
            _ => {}
        }
    }
}

/// Gets the controller ID from the configuration files
fn read_id() -> u64 {
    let mut file = File::open("/home/braden/.id").expect("Failed to read identity file");
    let mut buf = String::new();
    file.read_to_string(&mut buf).expect("Failed to read");

    buf.trim().parse().expect("ID File not in proper format")
}
