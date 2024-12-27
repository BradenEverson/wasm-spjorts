//! Controller Logic Handling

pub mod msg;
use std::sync::Arc;

use futures::SinkExt;
pub use msg::ControllerMessage;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;

use crate::serve::service::WebsocketWriteStream;

/// Controller ID
pub type ControllerId = u64;

/// A controller's held metadata
pub struct Controller {
    /// ID
    pub id: u64,
    /// Web Socket streams listening to the controller
    listeners: Vec<Arc<Mutex<WebsocketWriteStream>>>,
}

impl Controller {
    /// Creates a new controller
    pub fn new(id: u64) -> Self {
        Self {
            id,
            listeners: vec![],
        }
    }

    /// Adds a new listener to the controller
    pub fn new_listener(&mut self, listener: Arc<Mutex<WebsocketWriteStream>>) {
        self.listeners.push(listener);
    }

    /// Broadcast a binary message to all listeners connected
    pub async fn broadcast(&mut self, msg: &[u8]) {
        let mut drop_queue = vec![];
        for (idx, listener) in self.listeners.iter().enumerate() {
            if listener
                .lock()
                .await
                .send(Message::binary(msg))
                .await
                .is_err()
            {
                drop_queue.push(idx);
            }
        }

        let filtered: Vec<_> = self
            .listeners
            .clone()
            .into_iter()
            .enumerate()
            .filter(|(idx, _)| !drop_queue.contains(idx))
            .map(|(_, val)| val)
            .collect();

        self.listeners = filtered
    }
}
