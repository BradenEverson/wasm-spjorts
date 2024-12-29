//! Main firmware driver for a controller, reading rotational data and button press events from the
//! Pi and transmitting this information to the game server over web sockets

use std::{fs::File, io::Read};

#[tokio::main]
async fn main() {
    let id = read_id();
    // Connect to server
}

/// Gets the controller ID from the configuration files
fn read_id() -> u64 {
    let mut file = File::open("~/.id").expect("Failed to read identity file");
    let mut buf = String::new();
    file.read_to_string(&mut buf).expect("Failed to read");

    buf.parse().expect("ID File not in proper format")
}
