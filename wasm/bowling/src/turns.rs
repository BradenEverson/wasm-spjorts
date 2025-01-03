//! Turn taking plugin for the bowling state

use std::sync::{Arc, RwLock};

use bevy::{
    app::{Plugin, Update},
    prelude::{Query, Res, Resource, Text, Transform, Visibility},
};
use bevy_rapier3d::prelude::Velocity;

use crate::setup::{FinalScore, Hideable, Pin, ScorecardBg};

/// Type of score a score can be (strike, spare, normal)
pub enum Score {
    /// A non-special score
    Normal(u8),
    /// A strike
    Strike,
    /// A spare
    Spare,
    /// No score yet
    None,
}

/// Bowling game current state
#[derive(Debug, Clone, Copy)]
pub struct BowlingState {
    /// What is the current frame we're at
    frame_number: usize,
    /// Which throw in the frame are we on
    throw_num: u8,
    /// Scores per frame
    frame_scores: [u8; 10],
    /// Pins currently down
    pins_down: u8,
    /// Is the current throw done
    throw_done: bool,
}

/// Send + Sync wrapper around BowlingState
#[derive(Resource, Debug, Clone, Default)]
pub struct BowlingStateWrapper(Arc<RwLock<BowlingState>>);

impl BowlingState {
    /// Returns the string representation of the state
    pub fn render(&self) -> String {
        let renderables: Vec<String> = self
            .frame_scores
            .iter()
            .enumerate()
            .map(|(idx, val)| {
                if idx + 1 < self.frame_number {
                    format!("{:^2}", val)
                } else if idx + 1 == self.frame_number {
                    "__".to_string()
                } else {
                    "--".to_string()
                }
            })
            .collect();
        format!(
            r#"
    +-------+----+----+----+----+----+----+----+----+----+----+
    | Ctrlr | 1  | 2  | 3  | 4  | 5  | 6  | 7  | 8  | 9  | 10 |
    +-------+----+----+----+----+----+----+----+----+----+----+
    |  #1   | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |
    +-------+----+----+----+----+----+----+----+----+----+----+
            "#,
            renderables[0],
            renderables[1],
            renderables[2],
            renderables[3],
            renderables[4],
            renderables[5],
            renderables[6],
            renderables[7],
            renderables[8],
            renderables[9]
        )
    }
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
        self.throw_num += 1;
        self.throw_done = true;
    }

    /// Sets the current score for the current frame
    pub fn set_score(&mut self, score: u8) {
        self.frame_scores[self.frame_number as usize - 1] = score
    }

    /// Increments the current frame with bounds
    pub fn inc_frame(&mut self) -> bool {
        if self.frame_number < 10 {
            self.frame_number += 1;
            false
        } else {
            true
        }
    }

    /// Resets all triggers for a new frame
    pub fn reset(&mut self) {
        self.pins_down = 0;
        self.throw_done = false;
        self.throw_num = 1;
    }

    /// Toggles the throw status back
    pub fn set_throw_not_done(&mut self) {
        self.throw_done = false;
    }

    /// Gets the total score
    pub fn get_score(&self) -> usize {
        self.frame_scores.iter().map(|val| *val as usize).sum()
    }
}

impl BowlingStateWrapper {
    /// Gets the total score
    pub fn get_score(&self) -> usize {
        self.0.read().unwrap().get_score()
    }
    /// Renders the current state as a scorecard
    pub fn render(&self) -> String {
        self.0.read().unwrap().render()
    }
    /// Checks if the current throw is finished
    pub fn is_throw_done(&self) -> bool {
        self.0.read().unwrap().is_throw_done()
    }

    /// Gets the current amount of pins downed
    pub fn get_pins_down(&self) -> u8 {
        self.0.read().unwrap().get_pins_down()
    }

    /// Returns the current throw
    pub fn get_throw_num(&self) -> u8 {
        self.0.read().unwrap().get_throw_num()
    }

    /// Increases the current throw
    pub fn inc_throw_num(&self) {
        self.0.write().unwrap().inc_throw_num()
    }

    /// Sets the current score for the current frame
    pub fn set_score(&self, score: u8) {
        self.0.write().unwrap().set_score(score)
    }

    /// Increments the current frame with bounds
    pub fn inc_frame(&self) -> bool {
        self.0.write().unwrap().inc_frame()
    }

    /// Resets all triggers for a new frame
    pub fn reset(&self) {
        self.0.write().unwrap().reset()
    }

    /// Increments the current amount of toppled pins
    pub fn topple_pin(&self) {
        self.0.write().unwrap().pins_down += 1
    }

    /// Toggles the throw status back
    pub fn set_throw_not_done(&self) {
        self.0.write().unwrap().set_throw_not_done()
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
        app.init_resource::<BowlingStateWrapper>()
            .add_systems(Update, update_frame_logic);
    }
}

/// Check current amount of pins down if a throw is over and add that to the score, update current
/// frame or throw and reset pins if need be
fn update_frame_logic(
    bowling_state: Res<'_, BowlingStateWrapper>,
    mut pins: Query<'_, '_, (&mut Transform, &mut Pin, &mut Velocity)>,
    mut hideable: Query<'_, '_, (&Hideable, &mut Visibility)>,
    mut score_card: Query<'_, '_, (&mut Text, &FinalScore)>,
    mut bg: Query<'_, '_, (&mut Visibility, &ScorecardBg)>,
) {
    if bowling_state.is_throw_done() {
        let game_over = match (
            bowling_state.get_throw_num() - 1,
            bowling_state.get_pins_down(),
        ) {
            (1, 10) => {
                // Strike
                // TODO: Make extra enum type for a score, include strike and spare or default to
                // do *ACTUAL* score calculation
                bowling_state.set_score(10);
                bowling_state.reset();
                pins.iter_mut()
                    .for_each(|(mut transformation, mut pin, mut velocity)| {
                        pin.reset(&mut transformation, &mut velocity)
                    });
                Some(bowling_state.inc_frame())
            }
            (2, 10) => {
                // Spare
                bowling_state.set_score(10);
                bowling_state.reset();
                pins.iter_mut()
                    .for_each(|(mut transformation, mut pin, mut velocity)| {
                        pin.reset(&mut transformation, &mut velocity)
                    });
                Some(bowling_state.inc_frame())
            }
            (1, _) => {
                bowling_state.set_throw_not_done();
                None
            }
            (_, val) => {
                bowling_state.set_score(val);
                bowling_state.reset();
                pins.iter_mut()
                    .for_each(|(mut transformation, mut pin, mut velocity)| {
                        pin.reset(&mut transformation, &mut velocity)
                    });
                Some(bowling_state.inc_frame())
            }
        };

        if let Some(true) = game_over {
            for (_, mut vis) in hideable.iter_mut() {
                *vis = Visibility::Hidden
            }

            if let Ok((mut vis, _)) = bg.get_single_mut() {
                *vis = Visibility::Visible
            }

            if let Ok((mut text, _)) = score_card.get_single_mut() {
                let final_score = format!("Final Score: {}", bowling_state.get_score());
                *text = Text::new(final_score);
            }
        }
    }
}
