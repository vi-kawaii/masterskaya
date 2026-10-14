use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_tnua::builtins::{TnuaBuiltinJumpConfig, TnuaBuiltinWalkConfig};
use bevy_tnua::prelude::*;
use bevy_tnua_avian3d::*;

use crate::controller::scheme::{CharacterControlScheme, CharacterControlSchemeConfig};

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            |mut commands: Commands,
             mut configs: ResMut<Assets<CharacterControlSchemeConfig>>| {
                let config = CharacterControlSchemeConfig {
                    basis: TnuaBuiltinWalkConfig {
                        // Ниже полвысоты капсулы (0.9) — прижимает тело к земле.
                        // Низ капсулы уходит в землю на ~0.1, визуально не заметно.
                        float_height: 0.80,
                        // Убираем «запас» пружины — меньше паразитного подъёма.
                        cling_distance: 0.05,
                        spring_strength: 100.0,
                        spring_dampening: 1.0,

                        speed: 6.0,
                        acceleration: 1000.0,
                        air_acceleration: 100.0,

                        ..default()
                    },
                    jump: TnuaBuiltinJumpConfig {
                        height: 1.2,
                        takeoff_extra_gravity: 15.0,
                        ..default()
                    },
                };
                let handle = configs.add(config);
                commands.insert_resource(CharacterControlConfigHandle(handle));
            },
        );

        app.insert_resource(Gravity(Vec3::new(0.0, -20.0, 0.0)))
            .add_plugins(PhysicsPlugins::default())
            .add_plugins((
                TnuaControllerPlugin::<CharacterControlScheme>::new(FixedUpdate),
                TnuaAvian3dPlugin::new(FixedUpdate),
            ))
            .add_plugins(PhysicsDebugPlugin::default());
    }
}

#[derive(Resource)]
pub struct CharacterControlConfigHandle(pub Handle<CharacterControlSchemeConfig>);
