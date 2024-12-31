//! Turn taking plugin for the bowling state

use bevy::{
    app::{Plugin, Update},
    prelude::{ResMut, Resource},
};

/// Bowling game current state
#[derive(Resource, Debug, Clone, Copy)]
pub struct BowlingState {
    /// What is the current frame we're at
    frame_number: u8,
    /// Which throw in the frame are we on
    throw_num: u8,
    /// Scores per frame
    frame_scores: [u8; 10],
    /// Pins currently down
    pins_down: u8,
    /// Is the current throw done
    throw_done: bool,
}

impl BowlingState {
    /// Checks if the current throw is finished
    pub fn is_throw_done(&self) -> bool {
        self.throw_done
    }

    /// Gets the current amount of pins downed
    pub fn get_pins_down(&self) -> u8 {
        self.pins_down
    }

    /// Returns the current throw
    pub fn get_throw_num(&self) -> u8 {
        self.throw_num
    }

    /// Increases the current throw
    pub fn inc_throw_num(&mut self) {
        self.throw_num += 1
    }

    /// Sets the current score for the current frame
    pub fn set_score(&mut self, score: u8) {
        self.frame_scores[self.frame_number as usize] += score
    }

    /// Increments the current frame with bounds
    pub fn inc_frame(&mut self) {
        if self.frame_number < 10 {
            self.frame_number += 1
        }
    }

    /// Resets all triggers for a new frame
    pub fn reset(&mut self) {
        self.pins_down = 0;
        self.throw_done = false;
    }
}

impl Default for BowlingState {
    fn default() -> Self {
        Self {
            frame_number: 1,
            throw_num: 1,
            frame_scores: [0; 10],
            pins_down: 0,
            throw_done: false,
        }
    }
}

/// Bowling state plugin
pub struct BowlingTurnPlugin;

impl Plugin for BowlingTurnPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<BowlingState>()
            .add_systems(Update, update_frame_logic);
    }
}

/// Check current amount of pins down if a throw is over and add that to the score, update current
/// frame or throw and reset pins if need be
fn update_frame_logic(mut bowling_state: ResMut<'_, BowlingState>) {
    if bowling_state.is_throw_done() {
        match (bowling_state.get_throw_num(), bowling_state.get_pins_down()) {
            (1, 10) => {
                // Strike
                // TODO: Make extra enum type for a score, include strike and spare or default to
                // do *ACTUAL* score calculation
                bowling_state.set_score(10);
                bowling_state.reset();
                bowling_state.inc_frame();
            }
            (2, 10) => {
                // Spare
                bowling_state.set_score(10);
                bowling_state.reset();
                bowling_state.inc_frame();
            }
            (1, _) => {
                bowling_state.inc_throw_num();
            }
            (2, val) => {
                bowling_state.set_score(val);
                bowling_state.reset();
                bowling_state.inc_frame();
            }
            _ => unreachable!("All other things"),
        }
    }
}
