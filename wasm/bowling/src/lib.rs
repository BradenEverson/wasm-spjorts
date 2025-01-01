//! Bevy bowling game

use bevy::prelude::*;
use bevy_rapier3d::{
    plugin::{NoUserData, RapierPhysicsPlugin},
    prelude::{RigidBody, Velocity},
};
use crossbeam_channel::Sender;
use setup::{setup, Ball, Pin, BALL_SPEED, BALL_START_Z, LANE_WIDTH};
use spjorts_core::{communication::JsMessage, ActionReader, ActionSender, Communication};
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
            .insert_resource(ActionReader(read))
            .add_systems(Startup, setup)
            .add_systems(Update, (handle_input, handle_ball));

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
    mut param_set: ParamSet<
        '_,
        '_,
        (
            Query<'_, '_, (&mut Transform, &mut Ball, &mut Velocity, &mut RigidBody)>,
            Query<'_, '_, (&mut Transform, &Pin, &mut Velocity)>,
        ),
    >,
    time: Res<'_, Time>,
) {
    if let Ok((mut transform, mut ball, mut velocity, mut rigid)) = param_set.p0().get_single_mut()
    {
        if transform.translation.y <= -2.0 {
            reset_ball(&mut transform, &mut ball, &mut rigid, &mut velocity);
            param_set
                .p1()
                .iter_mut()
                .for_each(|(mut transformation, pin, mut velocity)| {
                    pin.reset(&mut transformation, &mut velocity)
                });
        } else {
            if let Some(direction) = &mut ball.moving {
                let threshold = LANE_WIDTH / 2.0;
                let dx = if *direction { 0.1 } else { -0.1 };
                transform.translation.x += dx * time.delta_secs();

                if transform.translation.x >= threshold || transform.translation.x <= -threshold {
                    *direction = !*direction
                }
            }
        }
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
                        ball.velocity = forward.normalize() * BALL_SPEED;
                        *velocity = Velocity::linear(ball.velocity);
                    }
                }
                JsMessage::ButtonB => {
                    ball.moving = None;
                }
                JsMessage::Rotate(pitch, roll, _) => {
                    if !ball.released {
                        // todo!("Based on previous rotation and velocity, calculate new speed of ball and set new rotation");
                        let new = Quat::from_euler(EulerRot::XYZ, pitch, roll, 0f32);
                        transform.rotation = new;
                        ball.curr_rotation = new;
                    }
                }
            }
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
    *velocity = Velocity::zero();
    *rigid = RigidBody::KinematicPositionBased;
    ball.released = false;
}
