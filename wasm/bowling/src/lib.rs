//! Bevy bowling game

use bevy::prelude::*;
use bevy_rapier3d::{
    plugin::{NoUserData, RapierPhysicsPlugin},
    prelude::{Collider, Friction, GravityScale, Restitution, RigidBody, Velocity},
};
use crossbeam_channel::Sender;
use spjorts_core::{ActionReader, ActionSender, Communication};
use wasm_bindgen::prelude::wasm_bindgen;

/// Lane length
const LANE_LENGTH: f32 = 30.0;
/// Lane width
const LANE_WIDTH: f32 = 3.0;

/// Number of pins in a standard arrangement
const PIN_COUNT: usize = 10;

/// Distance from origin to the first pin
const PIN_START_Z: f32 = 10.0;

/// Where the ball starts
const BALL_START_Z: f32 = -5.0;

/// How fast the ball moves once “released”
const BALL_SPEED: f32 = 5.0;

/// Pin radius
const PIN_RADIUS: f32 = 0.15;
/// Pin height
const PIN_HEIGHT: f32 = 0.75;

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
            .add_systems(Startup, setup);
        //.add_systems(Update, (handle_input, update_ball_movement));

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

/// Marks the ball entity
#[derive(Component)]
struct Ball {
    // Whether the ball has been “released”
    released: bool,
    // Current velocity
    velocity: Vec3,
}

/// Marks a pin entity
#[derive(Component)]
struct Pin;

/// Spawns the lane, the ball, and pins
fn setup(
    mut commands: Commands<'_, '_>,
    mut meshes: ResMut<'_, Assets<Mesh>>,
    mut materials: ResMut<'_, Assets<StandardMaterial>>,
) {
    // Spawn Lane
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(LANE_WIDTH, 0.1, LANE_LENGTH))),
        MeshMaterial3d(materials.add(Color::hsl(33.0, 0.20, 0.76))),
        Transform::from_xyz(0.0, -0.05, LANE_LENGTH * 0.5 - 10.0),
        Name::new("Lane"),
        Collider::cuboid(LANE_WIDTH, 0.1, LANE_LENGTH),
        Restitution::coefficient(0.0),
        RigidBody::Fixed,
        Friction::coefficient(0.04),
    ));

    // Spawn pins
    let rows = how_many_rows(PIN_COUNT);
    for row in 1..=rows {
        let z_pos = PIN_START_Z + (row as f32);

        let start_pos = 0.0 - ((row - 1) as f32 / 2.0) * PIN_RADIUS * 4.0;

        for pin in 0..row {
            let x_pos = start_pos + ((pin as f32) * PIN_RADIUS * 4.0);

            commands.spawn((
                Mesh3d(meshes.add(Cylinder::new(PIN_RADIUS, PIN_HEIGHT))),
                MeshMaterial3d(materials.add(Color::hsl(33.0, 0.0, 0.93))),
                Transform::from_xyz(x_pos, PIN_HEIGHT * 0.5 + 0.05, z_pos),
                Pin,
                Name::new(format!("Pin {pin} in Row {row}")),
                Collider::cylinder(PIN_HEIGHT * 0.5, PIN_RADIUS),
                RigidBody::Dynamic,
                Restitution::coefficient(0.0),
                GravityScale(1.0),
                Velocity::linear(Vec3::ZERO),
            ));
        }
    }

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 3.0, -10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(0.0, 3.0, -13.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

/// Calculates how many rows a bowling lane should have
pub fn how_many_rows(pins: usize) -> usize {
    let mut count = 0;
    let mut pins = pins;
    while pins > 0 {
        count += 1;
        if count > pins {
            pins = 0
        } else {
            pins -= count
        }
    }

    count
}
