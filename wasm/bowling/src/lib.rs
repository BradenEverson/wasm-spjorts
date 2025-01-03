//! Bevy bowling game

use bevy::prelude::*;
use bevy_rapier3d::{
    plugin::{NoUserData, RapierPhysicsPlugin},
    prelude::{RigidBody, Velocity},
};
use crossbeam_channel::Sender;
use setup::{setup, Ball, Pin, BALL_START_Z, LANE_WIDTH};
use spjorts_core::{communication::JsMessage, ActionReader, ActionSender, Communication};
use turns::{BowlingStateWrapper, BowlingTurnPlugin};
use wasm_bindgen::prelude::wasm_bindgen;

pub mod setup;
pub mod turns;

/// System responsible for running and communicating with a Bevy app
#[wasm_bindgen]
pub struct Runner {
    app: App,
    write: Sender<Communication>,
}

#[wasm_bindgen]
impl Runner {
    /// Creates a new runner
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let (write, read) = crossbeam_channel::unbounded();

        let mut app = App::new();
        app.add_plugins(DefaultPlugins)
            .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
            .add_plugins(BowlingTurnPlugin)
            .insert_resource(ActionReader(read))
            .add_systems(Startup, setup)
            .add_systems(Update, (handle_input, handle_ball, check_pins, update_ui));

        Runner { app, write }
    }

    /// Get the sender pipeline
    #[wasm_bindgen]
    pub fn get_send(&self) -> ActionSender {
        ActionSender::new(self.write.clone())
    }

    /// Run the Bevy App
    #[wasm_bindgen]
    pub fn run(&mut self) {
        self.app.run();
    }
}

/// Handles resetting the ball and pins if they go too far
fn handle_ball(
    mut ball: Query<'_, '_, (&mut Transform, &mut Ball, &mut Velocity, &mut RigidBody)>,
    state: Res<'_, BowlingStateWrapper>,
    time: Res<'_, Time>,
) {
    if let Ok((mut transform, mut ball, mut velocity, mut rigid)) = ball.get_single_mut() {
        if transform.translation.y <= -2.0 || *velocity == Velocity::zero() {
            reset_ball(&mut transform, &mut ball, &mut rigid, &mut velocity);
            state.inc_throw_num();
        } else {
            if let Some(direction) = &mut ball.moving {
                let threshold = LANE_WIDTH / 2.0;
                let dx = if *direction { 1.5 } else { -1.5 };
                transform.translation.x += dx * time.delta_secs();

                if transform.translation.x >= threshold {
                    *direction = false
                } else if transform.translation.x <= -threshold {
                    *direction = true
                }
            }
        }
    }
}

/// Updates the UI
fn update_ui(mut ui_elements: Query<'_, '_, &mut Text>, state: Res<'_, BowlingStateWrapper>) {
    let render = state.render();
    if let Ok(mut txt) = ui_elements.get_single_mut() {
        *txt = Text::new(render);
    }
}

/// Reads input from the channel and applies it to the ball’s transform or sets release velocity
fn handle_input(
    mut param_set: ParamSet<
        '_,
        '_,
        (
            Query<'_, '_, (&mut Transform, &mut Ball, &mut Velocity, &mut RigidBody)>,
            Query<'_, '_, (&mut Transform, &Pin, &mut Velocity)>,
        ),
    >,
    read: Res<'_, ActionReader>,
) {
    if let Ok(msg) = read.0.try_recv() {
        if let Ok((mut transform, mut ball, mut velocity, mut rigid)) =
            param_set.p0().get_single_mut()
        {
            match msg {
                JsMessage::ButtonA => {
                    if !ball.released && ball.moving.is_none() {
                        ball.released = true;
                        *rigid = RigidBody::Dynamic;

                        let forward = transform.local_z();
                        let curr_velocity = forward.normalize() * ball.get_speed();
                        *velocity = Velocity::linear(curr_velocity);
                    }
                }
                JsMessage::ButtonB => {
                    ball.moving = None;
                }
                JsMessage::Rotate(pitch, roll, _) => {
                    if !ball.released {
                        let new = Quat::from_euler(EulerRot::XYZ, pitch, roll, 0f32);
                        transform.rotation = new;
                        ball.rotations.push(new);
                    }
                }
            }
        }
    }
}

/// Checks for whether pins are toppled or not
pub fn check_pins(
    mut pins: Query<'_, '_, (&mut Pin, &Transform)>,
    state: Res<'_, BowlingStateWrapper>,
) {
    for (mut pin, transform) in &mut pins {
        let height = transform.translation.y;
        if height < 0.2 && !pin.toppled {
            pin.toppled = true;
            state.topple_pin();
        }
    }
}

/// Resets a ball to its initial position
pub fn reset_ball(
    transform: &mut Transform,
    ball: &mut Ball,
    rigid: &mut RigidBody,
    velocity: &mut Velocity,
) {
    transform.translation = Vec3::new(0.0, 0.3, BALL_START_Z);
    transform.rotation = Quat::IDENTITY;
    ball.velocity = Vec3::ZERO;
    ball.moving = Some(true);
    ball.rotations = vec![];
    *velocity = Velocity::zero();
    *rigid = RigidBody::KinematicPositionBased;
    ball.released = false;
}
