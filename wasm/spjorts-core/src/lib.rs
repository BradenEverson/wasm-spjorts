//! Shared struct and utilities for all WASM games

use bevy::prelude::Resource;
use crossbeam_channel::{Receiver, Sender};
use wasm_bindgen::prelude::wasm_bindgen;

/// What is JavaScript sending back and forth
pub type Communication = (f32, f32, f32);

/// A JavaScript event sender pipeline
#[wasm_bindgen]
pub struct ActionSender(Sender<Communication>);

impl ActionSender {
    /// Creates a new sender
    pub fn new(sender: Sender<Communication>) -> Self {
        Self(sender)
    }
}

#[wasm_bindgen]
impl ActionSender {
    /// Sends x, y and z positional data to the game state
    pub fn send(&mut self, x: f32, y: f32, z: f32) {
        self.0.send((x, y, z)).expect("Send message");
    }
}

/// A JavaScript event reader pipeline
#[derive(Resource)]
pub struct ActionReader(pub Receiver<(f32, f32, f32)>);
