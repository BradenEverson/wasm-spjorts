//! A basic cube WASM app

use bevy::prelude::*;
use crossbeam_channel::{Receiver, Sender};
use wasm_bindgen::prelude::wasm_bindgen;

/// An app instance with internal JS communications
#[wasm_bindgen]
pub struct Runner {
    app: App,
    write: Sender<(f32, f32, f32)>,
}

/// A JS event sender pipeline
#[wasm_bindgen]
pub struct ActionSender(Sender<(f32, f32, f32)>);

#[wasm_bindgen]
impl ActionSender {
    /// Sends x, y and z positional data to the game state
    pub fn send(&mut self, x: f32, y: f32, z: f32) {
        self.0.send((x, y, z)).expect("Send message");
    }
}

/// A JS event reader pipeline
#[derive(Resource)]
pub struct ActionReader(Receiver<(f32, f32, f32)>);

#[wasm_bindgen]
impl Runner {
    /// Creates a new runner instance and initializes the message channel
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let (write, read) = crossbeam_channel::unbounded();
        let mut app = App::new();
        app.add_plugins(DefaultPlugins)
            .insert_resource(ActionReader(read))
            .add_systems(Startup, setup)
            .add_systems(Update, move_cube);

        Runner { app, write }
    }

    /// Gets the runner's send channel
    pub fn get_send(&self) -> ActionSender {
        ActionSender(self.write.clone())
    }

    /// Runs the app as a blocking task
    pub fn run(&mut self) {
        self.app.run();
    }
}

/// System that spawns the cube, lighting and camera view
fn setup(
    mut commands: Commands<'_, '_>,
    mut meshes: ResMut<'_, Assets<Mesh>>,
    mut materials: ResMut<'_, Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::default())),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_translation(Vec3::ZERO),
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 2.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(3.0, 3.0, 3.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

/// Moves a cube with respect to position
fn move_cube(mut cubes: Query<'_, '_, (&Mesh3d, &mut Transform)>, read: Res<'_, ActionReader>) {
    if let Ok(val) = read.0.try_recv() {
        for (_, mut transform) in &mut cubes {
            let prev = transform.translation;
            transform.translation += Vec3::new(val.0, val.1, val.2) - prev;
        }
    }
}
