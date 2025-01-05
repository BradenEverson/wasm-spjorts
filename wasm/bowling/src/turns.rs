//! Turn taking plugin for the bowling state

use std::{
    fmt::Display,
    sync::{Arc, RwLock},
};

use bevy::{
    app::{Plugin, Update},
    prelude::{ParamSet, Query, Res, Resource, Text, Transform, Visibility},
};
use bevy_rapier3d::prelude::Velocity;

use crate::setup::{FinalScore, Hideable, Pin, ScorecardBg};

/// Type of score a score can be (strike, spare, normal)
#[derive(Debug, Clone, Copy)]
pub enum Score {
    /// A non-special score
    Normal(usize),
    /// A strike
    Strike,
    /// A spare
    Spare,
    /// No score yet
    None,
}

impl Display for Score {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = match self {
            Self::Normal(0) => "--".to_string(),
            Self::Normal(val) => format!("{}", val),
            Self::Strike => "X".to_string(),
            Self::Spare => "/".to_string(),
            Self::None => "".to_string(),
        };
        write!(f, "{}", val)
    }
}

/// Bowling game current state
#[derive(Debug, Clone)]
pub struct BowlingState {
    /// What is the current frame we're at
    frame_number: usize,
    /// Which throw in the frame are we on
    throw_num: u8,
    /// Scores per frame for each player
    player_frame_scores: Vec<[Score; 10]>,
    /// Pins currently down
    pins_down: u8,
    /// Is the current throw done
    throw_done: bool,
    /// Current player's turn
    turn: usize,
}

/// Send + Sync wrapper around BowlingState
#[derive(Resource, Debug, Clone, Default)]
pub struct BowlingStateWrapper(Arc<RwLock<BowlingState>>);

impl BowlingState {
    /// Returns the string representation of the state
    pub fn render(&self) -> String {
        let scores: Vec<String> = self
            .player_frame_scores
            .iter()
            .enumerate()
            .map(|(player, score)| {
                let renderables = score
                    .iter()
                    .enumerate()
                    .map(|(idx, val)| {
                        if idx + 1 <= self.frame_number {
                            let rendered = format!("{}", val);
                            format!("{:^2}", rendered)
                        } else {
                            "##".to_string()
                        }
                    })
                    .collect::<Vec<_>>();

                let arrow = if player == self.turn { "-->" } else { "   " };

                format!(
                    r#"
{} |  {:^2}   | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |
    +-------+----+----+----+----+----+----+----+----+----+----+"#,
                    arrow,
                    player,
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
            })
            .collect();

        let mut start_str = r#"
    +-------+----+----+----+----+----+----+----+----+----+----+
    | Plr # | 1  | 2  | 3  | 4  | 5  | 6  | 7  | 8  | 9  | 10 |
    +-------+----+----+----+----+----+----+----+----+----+----+"#
            .to_string();

        for player in scores {
            start_str = format!("{}{}", start_str, player)
        }

        start_str
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
        self.player_frame_scores[self.turn][self.frame_number as usize - 1] =
            Score::Normal(score as usize)
    }

    /// Sets the current score for the current frame to a spare
    pub fn set_spare(&mut self) {
        self.player_frame_scores[self.turn][self.frame_number as usize - 1] = Score::Spare
    }

    /// Sets the current score for the current frame to a strike
    pub fn set_strike(&mut self) {
        self.player_frame_scores[self.turn][self.frame_number as usize - 1] = Score::Strike
    }

    /// Increments the current frame with bounds
    pub fn inc_frame(&mut self) -> bool {
        if self.frame_number < 10 {
            if self.turn >= self.player_frame_scores.len() - 1 {
                self.turn = 0;
                self.frame_number += 1;
            } else {
                self.turn += 1
            }
            false
        } else if self.frame_number == 10 {
            if self.turn >= self.player_frame_scores.len() - 1 {
                true
            } else {
                self.turn += 1;
                false
            }
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
    pub fn get_score(&self) -> Vec<(usize, usize)> {
        self.player_frame_scores
            .iter()
            .enumerate()
            .map(|(id, score)| (id, get_score(score)))
            .collect()
    }

    /// Sets the number of players in a game
    pub fn set_players(&mut self, num: usize) {
        self.player_frame_scores = vec![[Score::None; 10]; num]
    }
}

impl BowlingStateWrapper {
    /// Sets the current score for the current frame to a spare
    pub fn set_spare(&self) {
        self.0.write().unwrap().set_spare()
    }

    /// Sets the current score for the current frame to a strike
    pub fn set_strike(&self) {
        self.0.write().unwrap().set_strike()
    }
    /// Gets the total score
    pub fn get_score(&self) -> Vec<(usize, usize)> {
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

    /// Sets the number of players in the current game
    pub fn set_players(&self, num: usize) {
        self.0.write().unwrap().set_players(num)
    }
}

impl Default for BowlingState {
    fn default() -> Self {
        Self {
            frame_number: 1,
            throw_num: 1,
            player_frame_scores: vec![[Score::None; 10]],
            turn: 0,
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
    mut queries: ParamSet<
        '_,
        '_,
        (
            Query<'_, '_, (&mut Transform, &mut Pin, &mut Velocity)>,
            Query<'_, '_, (&Hideable, &mut Visibility)>,
            Query<'_, '_, (&mut Text, &FinalScore)>,
            Query<'_, '_, (&mut Visibility, &ScorecardBg)>,
        ),
    >,
) {
    if bowling_state.is_throw_done() {
        let game_over =
            match (
                bowling_state.get_throw_num() - 1,
                bowling_state.get_pins_down(),
            ) {
                (1, 10) => {
                    // Strike
                    bowling_state.set_strike();
                    bowling_state.reset();
                    queries.p0().iter_mut().for_each(
                        |(mut transformation, mut pin, mut velocity)| {
                            pin.reset(&mut transformation, &mut velocity)
                        },
                    );
                    Some(bowling_state.inc_frame())
                }
                (2, 10) => {
                    // Spare
                    bowling_state.set_spare();
                    bowling_state.reset();
                    queries.p0().iter_mut().for_each(
                        |(mut transformation, mut pin, mut velocity)| {
                            pin.reset(&mut transformation, &mut velocity)
                        },
                    );
                    Some(bowling_state.inc_frame())
                }
                (1, _) => {
                    bowling_state.set_throw_not_done();
                    None
                }
                (_, val) => {
                    bowling_state.set_score(val);
                    bowling_state.reset();
                    queries.p0().iter_mut().for_each(
                        |(mut transformation, mut pin, mut velocity)| {
                            pin.reset(&mut transformation, &mut velocity)
                        },
                    );
                    Some(bowling_state.inc_frame())
                }
            };

        if let Some(true) = game_over {
            for (_, mut vis) in queries.p1().iter_mut() {
                *vis = Visibility::Hidden
            }

            if let Ok((mut vis, _)) = queries.p3().get_single_mut() {
                *vis = Visibility::Visible
            }

            if let Ok((mut text, _)) = queries.p2().get_single_mut() {
                let scores = bowling_state.get_score();
                let (winner, score) = scores
                    .iter()
                    .max_by(|(_, prev_score), (_, score)| prev_score.cmp(score))
                    .unwrap();
                let final_score = format!(
                    "Game Over!\nPlayer {} wins with a final score of: {}\n\n\n\n\nPlease Restart the Page to Return Home :)",
                winner, score);
                *text = Text::new(final_score);
            }
        }
    }
}

/// Returns the score for a completed scorecard
pub fn get_score(scores: &[Score]) -> usize {
    let mut total_score = 0;

    for i in 0..scores.len() {
        total_score += match scores[i] {
            Score::Normal(pins) => pins,
            Score::Spare => 10 + next_roll_score(&scores, i + 1),
            Score::Strike => 10 + next_two_rolls_score(&scores, i + 1),
            _ => unreachable!("Score shouldn't be calculated until game is over"),
        };
    }

    total_score
}

fn next_roll_score(scores: &[Score], frame_index: usize) -> usize {
    if frame_index >= scores.len() {
        return 0;
    }

    match scores[frame_index] {
        Score::Normal(pins) => pins,
        Score::Spare => 10,
        Score::Strike => 10,
        _ => unreachable!("Score shouldn't be calculated until game is over"),
    }
}

fn next_two_rolls_score(scores: &[Score], frame_index: usize) -> usize {
    let mut score = 0;
    let mut rolls_counted = 0;

    for i in frame_index..scores.len() {
        if rolls_counted == 2 {
            break;
        }

        match scores[i] {
            Score::Normal(pins) => {
                score += pins;
                rolls_counted += 1;
            }
            Score::Spare => {
                score += 10;
                rolls_counted += 1;
            }
            Score::Strike => {
                score += 10;
                rolls_counted += 1;
            }
            _ => unreachable!("Score shouldn't be calculated until game is over"),
        }
    }

    score
}
