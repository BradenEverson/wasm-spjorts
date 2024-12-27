//! Bowling game!

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

#[wasm_bindgen]
impl Runner {
    /// Creates a new runner instance and initializes the message channel
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let (write, read) = crossbeam_channel::unbounded();
        let mut app = App::new();
        app.add_plugins(DefaultPlugins)
            .insert_resource(ActionReader(read))
            .add_systems(Startup, setup);

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

/// System that spawns lighting and camera view
fn setup(
    mut commands: Commands<'_, '_>,
    mut meshes: ResMut<'_, Assets<Mesh>>,
    mut materials: ResMut<'_, Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 2.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(3.0, 3.0, 3.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
