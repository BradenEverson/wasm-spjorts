//! Main firmware driver for a controller, reading rotational data and button press events from the
//! Pi and transmitting this information to the game server over web sockets

use futures_util::{SinkExt, StreamExt};
use rppal::{
    gpio::{Gpio, Trigger},
    i2c::I2c,
};
use server::control::{msg::WsMessage, ControllerMessage};
use std::{fs::File, io::Read, sync::mpsc::channel, time::Duration};
use tokio_tungstenite::connect_async;

/// Poll time for angles
pub const ANGLE_WAIT_TIME: u64 = 50;

/// MPU6050 I2C address
pub const MPU6050_ADDR: u16 = 0x68;

/// MPU6050 Registers
pub const PWR_MGMT_1: u8 = 0x6B;
/// MPU6050 Accel Registers
pub const ACCEL_XOUT_H: u8 = 0x3B;

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
            Trigger::RisingEdge,
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
            Trigger::RisingEdge,
            Some(Duration::from_millis(50)),
            move |_| {
                channel_b
                    .send(ControllerMessage::ButtonPressB)
                    .expect("Send button press B");
            },
        )
        .expect("Set interupt");

    // Initialize MPU6050
    let mut i2c = I2c::with_bus(1).expect("Initialize I2C");
    i2c.set_slave_address(MPU6050_ADDR)
        .expect("Set MPU6050 address");

    // Wake up MPU6050
    i2c.smbus_write_byte(PWR_MGMT_1, 0x00).expect("Wake up");

    // Read gyroscope information and send it over to WebSocket
    let dt = ANGLE_WAIT_TIME as f32 / 1000.0;
    std::thread::spawn(move || loop {
        let data = read_mpu6050(&mut i2c, dt);
        if let Some((pitch, yaw, roll)) = data {
            let msg = ControllerMessage::AngleInfo(pitch, yaw, roll);
            write_action.send(msg).expect("Send orientation data");
        }
        std::thread::sleep(Duration::from_millis(ANGLE_WAIT_TIME));
    });

    // Read gyroscope information and send it over to web socket
    while let Ok(msg) = read_action.recv() {
        write
            .send(msg.to_ws_message().expect("Convert to ws message"))
            .await
            .unwrap();
    }
}

/// Reads raw data from MPU6050 and calculates pitch, yaw, and roll
fn read_mpu6050(i2c: &mut I2c, dt: f32) -> Option<(f32, f32, f32)> {
    let mut buf = [0; 14];
    i2c.block_read(ACCEL_XOUT_H, &mut buf)
        .expect("Read MPU6050");

    let ax = ((buf[0] as i16) << 8 | buf[1] as i16) as f32;
    let ay = ((buf[2] as i16) << 8 | buf[3] as i16) as f32;
    let az = ((buf[4] as i16) << 8 | buf[5] as i16) as f32;

    let _gx = ((buf[8] as i16) << 8 | buf[9] as i16) as f32;
    let _gy = ((buf[10] as i16) << 8 | buf[11] as i16) as f32;
    let gz = ((buf[12] as i16) << 8 | buf[13] as i16) as f32;

    // Calculate pitch, yaw, and roll (simplified)
    let pitch = ax.atan2(az);
    let roll = ay.atan2(az);
    let yaw = gz * dt;

    Some((pitch, yaw, roll))
}

/// Gets the controller ID from the configuration files
fn read_id() -> u64 {
    let mut file = File::open("/home/braden/.id").expect("Failed to read identity file");
    let mut buf = String::new();
    file.read_to_string(&mut buf).expect("Failed to read");

    buf.trim().parse().expect("ID File not in proper format")
}
