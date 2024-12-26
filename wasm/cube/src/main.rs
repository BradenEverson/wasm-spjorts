//! A basic cube WASM app
use bevy::prelude::*;
use std::sync::{Arc, LazyLock, Mutex};
use wasm_bindgen::prelude::wasm_bindgen;

/// Global Player Position
pub const POSITION: LazyLock<Arc<Mutex<Position>>> =
    LazyLock::new(|| Arc::new(Mutex::new(Position(0f32, 0f32, 0f32))));

/// A position in 3-space
pub struct Position(f32, f32, f32);

impl std::ops::AddAssign<(f32, f32, f32)> for Position {
    fn add_assign(&mut self, rhs: (f32, f32, f32)) {
        self.0 += rhs.0;
        self.1 += rhs.1;
        self.2 += rhs.2;
    }
}

impl Position {
    /// Returns a copy-able tuple of points the position contains
    pub fn get_inner(&self) -> (f32, f32, f32) {
        (self.0.clone(), self.1.clone(), self.2.clone())
    }
}

#[wasm_bindgen]
/// Move the player by appropriate deltas
pub fn move_player(dx: f32, dy: f32, dz: f32) {
    *POSITION.lock().unwrap() += (dx, dy, dz);
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, move_cube)
        .run();
}

/// System that spawns the cube, lighting and camera view
fn setup(
    mut commands: Commands<'_, '_>,
    mut meshes: ResMut<'_, Assets<Mesh>>,
    mut materials: ResMut<'_, Assets<StandardMaterial>>,
) {
    let entity_spawn = Vec3::ZERO;
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::default())),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_translation(entity_spawn),
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 4.0, 20.0).looking_at(entity_spawn, Vec3::Y),
    ));

    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(3.0, 3.0, 3.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

/// Moves a cube with respect to position
fn move_cube(mut cubes: Query<'_, '_, &mut Transform>, timer: Res<'_, Time>) {
    for mut transform in &mut cubes {
        let position = (POSITION.lock().unwrap()).get_inner();
        transform.translation += Vec3::new(position.0, position.1, position.2) * timer.delta_secs();
    }
}
