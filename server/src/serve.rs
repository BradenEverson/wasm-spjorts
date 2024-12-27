//! Server State and Connection Handlers

use std::{collections::HashMap, sync::Arc};

use tokio::sync::{
    mpsc::{Receiver, Sender},
    Mutex,
};

use crate::control::{Controller, ControllerId, ControllerMessage};

pub mod registry;
pub mod service;

/// How many heartbeat checks before a controller should be dropped
pub const HEARTBEAT_LIMIT: usize = 50;

/// Controller metadata
pub type ControllerInfo = (ControllerId, ControllerMessage);

/// A current state including all connections and updates from controllers
pub struct SpjortState {
    /// All controllers that exist
    controllers: HashMap<ControllerId, Arc<Mutex<Controller>>>,
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
        Sender<Arc<Mutex<Controller>>>,
        Receiver<Arc<Mutex<Controller>>>,
    ) {
        let (sender, receiver) = tokio::sync::mpsc::channel(queue_limit);
        (
            Self {
                controllers: HashMap::new(),
                time_since_heartbeat: HashMap::new(),
            },
            sender,
            receiver,
        )
    }

    /// Connects a new controller to the context
    pub async fn connect(&mut self, controller: Arc<Mutex<Controller>>) {
        let id = { controller.lock().await.id };
        self.controllers.insert(id, controller);
        self.time_since_heartbeat.insert(id, 0);
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
            self.controllers.remove(key);
            self.time_since_heartbeat.remove(key);
        });
    }
}

/// Type of websocket connection
pub enum WsConnectionType {
    /// Controller with an ID
    Controller(u64),
    /// Listener listening to a controller with ID
    Listener(u64),
    /// Nothing yet
    None,
}
