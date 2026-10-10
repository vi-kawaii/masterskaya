use avian3d::prelude::*;
use bevy::prelude::*;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Gravity(Vec3::new(0.0, -20.0, 0.0)))
            .add_plugins(PhysicsPlugins::default())
            // Визуализация коллайдеров — только в debug-сборке.
            // В release не подключается, чтобы не тратить FPS.
            .add_plugins(PhysicsDebugPlugin::default());
    }
}
