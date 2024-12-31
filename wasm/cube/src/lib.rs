//! A basic cube WASM app

use bevy::prelude::*;
use crossbeam_channel::Sender;
use spjorts_core::{communication::JsMessage, ActionReader, ActionSender, Communication};
use wasm_bindgen::prelude::wasm_bindgen;

/// An app instance with internal JavaScript communications
#[wasm_bindgen]
pub struct Runner {
    app: App,
    write: Sender<Communication>,
}

/// Cube state
#[derive(Default, Component)]
pub struct Cube {
    /// The previous cube's rotation
    pub prev_rot: Quat,
}

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
        ActionSender::new(self.write.clone())
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
        Cube::default(),
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
fn move_cube(
    mut cubes: Query<'_, '_, (&Mesh3d, &mut Transform, &mut Cube)>,
    read: Res<'_, ActionReader>,
) {
    if let Ok(msg) = read.0.try_recv() {
        for (_, mut transform, mut cube_info) in &mut cubes {
            match msg {
                JsMessage::ButtonA => {
                    transform.translation += Vec3::new(1f32, 0f32, 0f32);
                }
                JsMessage::ButtonB => {
                    transform.translation += Vec3::new(-1f32, 0f32, 0f32);
                }
                JsMessage::Rotate(pitch, roll, yaw) => {
                    let new_rot = Quat::from_euler(EulerRot::XYZ, pitch, roll, yaw);
                    transform.rotation = new_rot;
                    cube_info.prev_rot = new_rot;
                }
            }
        }
    }
}
