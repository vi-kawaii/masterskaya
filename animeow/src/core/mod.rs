pub mod camera;
pub mod debug;
pub mod input;
pub mod physics;

use bevy::prelude::{App, Plugin};

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            camera::CameraPlugin,
            input::InputPlugin,
            physics::PhysicsPlugin,
            debug::DebugPlugin,
        ));
    }
}
