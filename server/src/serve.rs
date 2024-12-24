//! Server State and Connection Handlers

use std::collections::HashMap;

use slotmap::SlotMap;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::control::{Controller, ControllerId, ControllerMessage};

pub mod service;

/// How many heartbeat checks before a controller should be dropped
pub const HEARTBEAT_LIMIT: usize = 50;

/// Controller metadata
pub type ControllerInfo = (ControllerId, ControllerMessage);

/// A current state including all connections and updates from controllers
pub struct SpjortState {
    /// The read channels for new controller information coming through
    controller_channels: Receiver<ControllerInfo>,
    /// All controllers that exist
    controllers: SlotMap<ControllerId, Controller>,
    /// How long ago controllers have checked in to the server, they will be kicked if passing a
    /// tick threshold
    time_since_heartbeat: HashMap<ControllerId, usize>,
}

impl SpjortState {
    /// Creates a new spjort state and controller connector
    pub fn new(
        queue_limit: usize,
    ) -> (
        Self,
        Sender<Controller>,
        Receiver<Controller>,
        Sender<ControllerInfo>,
    ) {
        let (sender, receiver) = tokio::sync::mpsc::channel(queue_limit);
        let (sender_controllers, receiver_controllers) = tokio::sync::mpsc::channel(queue_limit);
        (
            Self {
                controller_channels: receiver_controllers,
                controllers: SlotMap::default(),
                time_since_heartbeat: HashMap::new(),
            },
            sender,
            receiver,
            sender_controllers,
        )
    }

    /// Polls any new info from the read queue
    pub async fn poll(&mut self) {
        if let Some((id, msg)) = self.controller_channels.recv().await {
            match msg {
                _ => println!("Not yet implemeneted buddy boy, but here's the ID {id:?}"),
            }
        }
    }

    /// Connects a new controller to the context
    pub fn connect(&mut self, controller: Controller) {
        let controller_id = self.controllers.insert(controller);
        self.time_since_heartbeat.insert(controller_id, 0);
    }

    /// Checks all heart beats and removes any connections that are higher than the limit
    pub fn heartbeat(&mut self) {
        let mut naughty = vec![];
        self.time_since_heartbeat.iter_mut().for_each(|(key, val)| {
            *val += 1;

            if *val >= HEARTBEAT_LIMIT {
                naughty.push(*key);
            }
        });

        naughty.iter().for_each(|key| {
            self.controllers.remove(*key);
            self.time_since_heartbeat.remove(key);
        });
    }
}
