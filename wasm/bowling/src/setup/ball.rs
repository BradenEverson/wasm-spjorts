//! Ball logic and velocity calculation methods

use bevy::{
    math::{Quat, Vec3},
    prelude::Component,
};

/// Marks the ball entity
#[derive(Component)]
pub struct Ball {
    /// Whether the ball has been “released”
    pub released: bool,
    /// Current velocity
    pub velocity: Vec3,
    /// Current rotation
    pub rotations: Vec<Quat>,
    /// If the ball is in X-axis toggle mode:
    /// * `None` if stopped,
    /// * `Some(true)` if moving positively towards (0 + LANE_WIDTH / 2)
    /// * `Some(false)` if moving negatively towards (0 - LANE_WIDTH / 2)
    pub moving: Option<bool>,
}

impl Default for Ball {
    fn default() -> Self {
        Self {
            released: Default::default(),
            velocity: Default::default(),
            rotations: Default::default(),
            moving: Some(true),
        }
    }
}

impl Ball {
    /// Uses the ball's rotational history to get a speed it would have at release on that angle
    pub fn get_speed(&self) -> f32 {
        if self.rotations.len() < 2 {
            return 1.0;
        }

        // TODO: Delta time would not be 60fps, but I'm not sure of the best way to get a timestamp
        // unless we register rotations as (Quat, Timestamp) or something, lets just see how bad
        // this is first
        let delta_time = 1.0 / 60.0;

        let q1 = self.rotations[self.rotations.len() - 2];
        let q2 = self.rotations[self.rotations.len() - 1];

        let dot_product = q1.dot(q2).clamp(-1.0, 1.0);
        let angular_velocity = (2.0 * dot_product.acos()) / delta_time;

        let scaling_factor = 10.0;
        let min_speed = 2.0;
        let max_speed = 20.0;

        let speed = scaling_factor * angular_velocity;

        speed.clamp(min_speed, max_speed)
    }
}
