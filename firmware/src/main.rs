//! Main firmware driver for a controller, reading rotational data and button press events from the
//! Pi and transmitting this information to the game server over web sockets

use futures_util::{SinkExt, StreamExt};
use rppal::{
    gpio::{Gpio, Trigger},
    i2c::I2c,
};
use server::control::{msg::WsMessage, ControllerMessage};
use std::{
    fs::File,
    io::Read,
    sync::mpsc::channel,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use tokio_tungstenite::connect_async;

/// Poll time for angles (ms)
pub const ANGLE_WAIT_TIME: u64 = 50;

/// MPU6050 I2C address
pub const MPU6050_ADDR: u16 = 0x68;

/// MPU6050 Registers
pub const PWR_MGMT_1: u8 = 0x6B;
/// MPU6050 Registers
pub const ACCEL_XOUT_H: u8 = 0x3B;

/// A Button pins
pub const BUTTON_A_PIN: u8 = 5;
/// B Button pins
pub const BUTTON_B_PIN: u8 = 6;

/// Sensitivity constants (assuming ±2g and ±250 deg/s)
/// (Check the datasheet if you changed full-scale ranges.)
const ACCEL_SENS: f32 = 16384.0; // LSB/g
const GYRO_SENS: f32 = 131.0; // LSB/(deg/s)

// Complementary filter alpha parameter
// Typically in the 0.90 - 0.98 range. Adjust as needed.
const ALPHA: f32 = 0.98;

/// Repeadetly tries to connect to a websocket until successful, waiting a given duration each time
/// it fails
async fn connect_with_retries(
    url: &str,
    retry_interval: Duration,
) -> tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>> {
    loop {
        match connect_async(url).await {
            Ok((ws, _)) => return ws,
            Err(e) => {
                eprintln!(
                    "Failed to connect: {}. Retrying in {:?}...",
                    e, retry_interval
                );
                std::thread::sleep(retry_interval);
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let id = read_id();
    let (tx_main, rx_main) = channel();

    // Connect to server
    let ws = connect_with_retries("ws://192.168.10.137:7878", Duration::from_secs(15)).await;

    let (mut write, _read) = ws.split();
    write
        .send(
            WsMessage::Controller(id)
                .to_ws_message()
                .expect("Convert to ws message"),
        )
        .await
        .unwrap();

    // Set up GPIO and button interrupts
    let gpio = Gpio::new().expect("Initialize GPIO");
    let mut button_a = gpio
        .get(BUTTON_A_PIN)
        .expect("Get GPIO pin for A button")
        .into_input_pulldown();

    let mut button_b = gpio
        .get(BUTTON_B_PIN)
        .expect("Get GPIO pin for B button")
        .into_input_pulldown();

    let tx_a = tx_main.clone();
    button_a
        .set_async_interrupt(
            Trigger::RisingEdge,
            Some(Duration::from_millis(50)),
            move |_| {
                tx_a.send(ControllerMessage::ButtonPressA)
                    .expect("Send button press A");
            },
        )
        .expect("Set interrupt for Button A");

    let tx_b = tx_main.clone();
    button_b
        .set_async_interrupt(
            Trigger::RisingEdge,
            Some(Duration::from_millis(50)),
            move |_| {
                tx_b.send(ControllerMessage::ButtonPressB)
                    .expect("Send button press B");
            },
        )
        .expect("Set interrupt for Button B");

    // Initialize MPU6050
    let mut i2c = I2c::with_bus(1).expect("Initialize I2C");
    i2c.set_slave_address(MPU6050_ADDR)
        .expect("Set MPU6050 address");

    // Wake up MPU6050
    i2c.smbus_write_byte(PWR_MGMT_1, 0x00)
        .expect("Wake up MPU6050");

    // First, calibrate the gyro offsets
    let (gx_offset, gy_offset, gz_offset) = calibrate_gyro(&mut i2c);
    println!(
        "Calibrated offsets: gx={}, gy={}, gz={}",
        gx_offset, gy_offset, gz_offset
    );

    // Shared angles protected by a mutex so the thread can update them
    let angles = Arc::new(Mutex::new((0f32, 0f32, 0f32))); // (pitch, roll, yaw)

    // Spawn a thread to continuously read and update angles
    let angles_clone = angles.clone();
    let tx_main_clone = tx_main.clone();
    thread::spawn(move || {
        let mut prev_pitch = 0.0;
        let mut prev_roll = 0.0;
        let mut prev_yaw = 0.0;

        // We'll track time in each loop for the gyro integration
        let dt = ANGLE_WAIT_TIME as f32 / 1000.0;

        loop {
            if let Some((pitch, roll, yaw)) = read_mpu6050(
                &mut i2c, dt, gx_offset, gy_offset, gz_offset, prev_pitch, prev_roll, prev_yaw,
            ) {
                // Update local copy
                prev_pitch = pitch;
                prev_roll = roll;
                prev_yaw = yaw;

                // Update the shared angles
                if let Ok(mut lock) = angles_clone.lock() {
                    *lock = (pitch, roll, yaw);
                }

                // Send a message to the main thread
                let msg = ControllerMessage::AngleInfo(pitch, yaw, roll);
                if tx_main_clone.send(msg).is_err() {
                    // If sending fails (main thread closed?), just break
                    break;
                }
            }

            std::thread::sleep(Duration::from_millis(ANGLE_WAIT_TIME));
        }
    });

    // Main loop: read messages from both the angle thread and button interrupts, then
    // send them over websocket
    while let Ok(msg) = rx_main.recv() {
        let ws_msg = msg.to_ws_message().expect("Convert to ws message");
        if let Err(e) = write.send(ws_msg).await {
            eprintln!("WebSocket send error: {}", e);
            break;
        }
    }
}

/// Reads raw data from MPU6050, performs a simple complementary filter, and returns (pitch, roll, yaw).
///
/// - `gx_offset, gy_offset, gz_offset`: offsets found by calibration
/// - `(prev_pitch, prev_roll, prev_yaw)`: the angles from previous iteration for the gyro integration
fn read_mpu6050(
    i2c: &mut I2c,
    dt: f32,
    gx_offset: f32,
    gy_offset: f32,
    gz_offset: f32,
    prev_pitch: f32,
    prev_roll: f32,
    prev_yaw: f32,
) -> Option<(f32, f32, f32)> {
    let mut buf = [0; 14];
    if i2c.block_read(ACCEL_XOUT_H, &mut buf).is_err() {
        eprintln!("Failed to read from MPU6050");
        return None;
    }

    // Convert raw bytes to signed 16-bit
    let ax_raw = i16::from_be_bytes([buf[0], buf[1]]) as f32;
    let ay_raw = i16::from_be_bytes([buf[2], buf[3]]) as f32;
    let az_raw = i16::from_be_bytes([buf[4], buf[5]]) as f32;
    // let temp_raw = i16::from_be_bytes([buf[6], buf[7]]) as f32; // if you want temperature
    let gx_raw = i16::from_be_bytes([buf[8], buf[9]]) as f32;
    let gy_raw = i16::from_be_bytes([buf[10], buf[11]]) as f32;
    let gz_raw = i16::from_be_bytes([buf[12], buf[13]]) as f32;

    // Convert to "g" units and deg/s
    let ax = ax_raw / ACCEL_SENS;
    let ay = ay_raw / ACCEL_SENS;
    let az = az_raw / ACCEL_SENS;
    let gx_deg_s = (gx_raw - gx_offset) / GYRO_SENS;
    let gy_deg_s = (gy_raw - gy_offset) / GYRO_SENS;
    let gz_deg_s = (gz_raw - gz_offset) / GYRO_SENS;

    // Convert deg/s to rad/s if you prefer working in radians
    let gx_rad_s = gx_deg_s.to_radians();
    let gy_rad_s = gy_deg_s.to_radians();
    let gz_rad_s = gz_deg_s.to_radians();

    let accel_pitch = ax.atan2((ay * ay + az * az).sqrt());
    let accel_roll = -ay.atan2((ax * ax + az * az).sqrt());

    // Integrate the gyro for pitch, roll, yaw
    let mut pitch = prev_pitch + gx_rad_s * dt;
    let mut roll = prev_roll + gy_rad_s * dt;
    let yaw = prev_yaw + gz_rad_s * dt;

    pitch = ALPHA * pitch + (1.0 - ALPHA) * accel_pitch;
    roll = ALPHA * roll + (1.0 - ALPHA) * accel_roll;

    Some((pitch, roll, yaw))
}

/// Calibrate gyro offsets by averaging samples while the MPU6050 is still.
fn calibrate_gyro(i2c: &mut I2c) -> (f32, f32, f32) {
    let samples = 100;
    let mut gx_sum = 0.0;
    let mut gy_sum = 0.0;
    let mut gz_sum = 0.0;

    for _ in 0..samples {
        let mut buf = [0; 6];
        if i2c.block_read(0x43, &mut buf).is_ok() {
            let gx_raw = i16::from_be_bytes([buf[0], buf[1]]) as f32;
            let gy_raw = i16::from_be_bytes([buf[2], buf[3]]) as f32;
            let gz_raw = i16::from_be_bytes([buf[4], buf[5]]) as f32;
            gx_sum += gx_raw;
            gy_sum += gy_raw;
            gz_sum += gz_raw;
        }
        thread::sleep(Duration::from_millis(10));
    }

    // Average raw values
    let gx_off = gx_sum / (samples as f32);
    let gy_off = gy_sum / (samples as f32);
    let gz_off = gz_sum / (samples as f32);

    (gx_off, gy_off, gz_off)
}

/// Gets the controller ID from the configuration file
fn read_id() -> u64 {
    let mut file = File::open("/home/braden/.id").expect("Failed to read identity file");
    let mut buf = String::new();
    file.read_to_string(&mut buf)
        .expect("Failed to read ID file");
    buf.trim().parse().expect("ID File not in proper format")
}
