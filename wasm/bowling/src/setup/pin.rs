//! Pin struct and reset handling

use bevy::prelude::{Component, Transform};
use bevy_rapier3d::prelude::Velocity;

/// Marks a pin entity
#[derive(Component)]
pub struct Pin {
    /// The Pin's initial state it will return to
    pub initial_coords: Transform,
    /// Is this pin toppled
    pub toppled: bool,
}

impl Pin {
    /// Initializes a Pin with initial coordinates
    pub fn new(initial_coords: Transform) -> Self {
        Self {
            initial_coords,
            toppled: false,
        }
    }
    /// Resets a transform and rotation and velocity according to an initial pin position
    pub fn reset(&mut self, transform: &mut Transform, velocity: &mut Velocity) {
        *transform = self.initial_coords;
        *velocity = Velocity::zero();
        self.toppled = false;
    }
}
