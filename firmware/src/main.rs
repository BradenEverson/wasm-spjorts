//! Main firmware driver for a controller, reading rotational data and button press events from the
//! Pi and transmitting this information to the game server over web sockets

use std::{fs::File, io::Read, time::Duration};

use rppal::gpio::{Gpio, Trigger};

/// A Button's pin
pub const BUTTON_A_PIN: u8 = 5;
/// B Button's pin
pub const BUTTON_B_PIN: u8 = 6;

#[tokio::main]
async fn main() {
    let id = read_id();

    // Connect to server

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

    button_a
        .set_async_interrupt(
            Trigger::FallingEdge,
            Some(Duration::from_millis(50)),
            move |event| {
                println!("Button A Event: {event:?}");
            },
        )
        .expect("Set interupt");

    button_b
        .set_async_interrupt(
            Trigger::FallingEdge,
            Some(Duration::from_millis(50)),
            move |event| {
                println!("Button B Event: {event:?}");
            },
        )
        .expect("Set interupt");

    // Read gyroscope information and send it over to websocket
    loop {}
}

/// Gets the controller ID from the configuration files
fn read_id() -> u64 {
    let mut file = File::open("/home/braden/.id").expect("Failed to read identity file");
    let mut buf = String::new();
    file.read_to_string(&mut buf).expect("Failed to read");

    buf.trim().parse().expect("ID File not in proper format")
}
