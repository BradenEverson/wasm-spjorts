//! Controller message protocol

use deku::{DekuContainerWrite, DekuError, DekuRead, DekuWrite};
use tokio_tungstenite::tungstenite::Message;

/// Messages a controller can send through
#[derive(DekuRead, DekuWrite, Debug, Clone, Copy, PartialEq)]
#[deku(id_type = "u8")]
pub enum ControllerMessage {
    /// Keep-alive signal
    #[deku(id = 0x01)]
    Heartbeat,
    /// Press A button
    #[deku(id = 0x02)]
    ButtonPressA,
    /// Press B button
    #[deku(id = 0x03)]
    ButtonPressB,
    /// Update current angle (pitch, roll, yaw)
    #[deku(id = 0x04)]
    AngleInfo(f32, f32, f32),
}

/// Messages a web socket connection can send before it's upgraded to a Controller or kept as is
#[derive(DekuRead, DekuWrite, Debug, Clone, Copy, PartialEq, Eq)]
#[deku(id_type = "u8")]
pub enum WsMessage {
    /// Establish a connection with a controller that has a certain ID
    #[deku(id = 0x01)]
    Establish(u64),
    /// Establish connection as a controller with the provided ID
    #[deku(id = 0x02)]
    Controller(u64),
}

impl ControllerMessage {
    /// Converts message to binary and then to a tokio tungstenite Message type
    pub fn to_ws_message(&self) -> Result<Message, DekuError> {
        let bytes = self.to_bytes()?;
        Ok(Message::Binary(bytes))
    }
}

impl WsMessage {
    /// Converts message to binary and then to a tokio tungstenite Message type
    pub fn to_ws_message(&self) -> Result<Message, DekuError> {
        let bytes = self.to_bytes()?;
        Ok(Message::Binary(bytes))
    }
}
