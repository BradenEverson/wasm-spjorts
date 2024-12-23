//! Controller Logic Handling

use slotmap::new_key_type;
use tokio::sync::mpsc::Sender;

pub mod msg;
pub use msg::ControllerMessage;

new_key_type! {
    /// Controller Slotmap ID
    pub struct ControllerId;
}

/// A controller's held metadata
pub struct Controller {
    /// Message send channel
    sender: Sender<ControllerMessage>,
    /// Controller pitch
    pitch: f32,
    /// Controller roll
    roll: f32,
    /// Controller yaw
    yaw: f32,
}
