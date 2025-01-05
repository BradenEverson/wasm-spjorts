//! Shared struct and utilities for all WASM games

use bevy::prelude::Resource;
use communication::JsMessage;
use crossbeam_channel::{Receiver, Sender};
use wasm_bindgen::prelude::wasm_bindgen;

pub mod communication;

/// What is JavaScript sending back and forth
pub type Communication = JsMessage;

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
    /// Press the A button
    pub fn press_a(&mut self) {
        self.0.send(JsMessage::ButtonA).expect("Press A Button")
    }

    /// Press the B button
    pub fn press_b(&mut self) {
        self.0.send(JsMessage::ButtonB).expect("Press B Button")
    }

    /// Rotate data with pitch, roll and yaw
    pub fn rotate(&mut self, pitch: f32, roll: f32, yaw: f32) {
        self.0
            .send(JsMessage::Rotate(pitch, roll, yaw))
            .expect("Rotate")
    }

    /// Set the number of players in the game
    pub fn set_players(&mut self, players: usize) {
        self.0
            .send(JsMessage::SetPlayers(players))
            .expect("Set num of players")
    }
}

/// A JavaScript event reader pipeline
#[derive(Resource)]
pub struct ActionReader(pub Receiver<Communication>);
