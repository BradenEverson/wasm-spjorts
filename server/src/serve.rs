//! Server State and Connection Handlers

use slotmap::SlotMap;
use tokio::sync::mpsc::Receiver;

use crate::control::{Controller, ControllerId, ControllerMessage};

pub mod service;

/// Controller metadata
pub type ControllerInfo = (ControllerId, Receiver<ControllerMessage>);

/// A current state including all connections and updates from controllers
#[derive(Default)]
pub struct SpjortState {
    /// The read channels for new controller information coming through
    controller_channels: Vec<ControllerInfo>,
    /// All controllers that exist
    controllers: SlotMap<ControllerId, Controller>,
}
