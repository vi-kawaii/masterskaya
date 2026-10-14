pub mod dialog;
pub mod fight;
pub mod hud;
pub mod move_;

use crate::assets::GameAssets;
use crate::controller::scheme::CharacterControlScheme;
use crate::core::physics::CharacterControlConfigHandle;
use crate::states::GameState;
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_tnua::prelude::*;
use bevy_tnua_avian3d::TnuaAvian3dSensorShape;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<crate::types::GameSettings>()
            .init_resource::<crate::types::Score>()
            .add_systems(OnEnter(GameState::InGame), spawn_island)
            .add_systems(OnExit(GameState::InGame), cleanup_game);
    }
}

#[derive(Component)]
pub struct Ground;

fn spawn_island(mut commands: Commands, assets: Res<GameAssets>) {
    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, -0.5, 0.0)),
    ));

    commands.spawn((
        Ground,
        RigidBody::Static,
        ColliderConstructorHierarchy::new(ColliderConstructor::TrimeshFromMeshWithConfig(
            TrimeshFlags::FIX_INTERNAL_EDGES,
        )),
        WorldAssetRoot(assets.island_scene.clone()),
        Transform::default(),
    ));
}

pub fn on_island_ready(
    _trigger: On<ColliderConstructorHierarchyReady>,
    mut commands: Commands,
    assets: Res<GameAssets>,
    config_handle: Res<CharacterControlConfigHandle>,
    player_exists: Query<Entity, With<crate::types::Player>>,
) {
    if !player_exists.is_empty() {
        return;
    }

    spawn_player_entity(&mut commands, &assets, &config_handle);
}

/// Компенсация поворота модели относительно forward родителя.
/// Модель anime-girl.glb экспортирована с forward в -Z, а Tnua использует +Z.
/// Следующая модель должна быть приведена к +Z в Blender, тогда константа = 0.0.
const MODEL_YAW_OFFSET: f32 = std::f32::consts::PI;

pub fn spawn_player_entity(
    commands: &mut Commands,
    assets: &GameAssets,
    config_handle: &CharacterControlConfigHandle,
) {
    commands
        .spawn((
            crate::types::Player,
            crate::actor::character::MovementStats::default(),
            crate::types::DesiredYaw::default(),

            RigidBody::Dynamic,
            Collider::capsule(0.4, 1.0),
            LockedAxes::new().lock_rotation_x().lock_rotation_z(),

            TnuaController::<CharacterControlScheme>::default(),
            TnuaConfig::<CharacterControlScheme>(config_handle.0.clone()),
            TnuaAvian3dSensorShape(Collider::sphere(0.1)),

            Transform::from_xyz(0.0, 2.0, 0.0),
            Visibility::default(),
        ))
        .with_children(|parent| {
            parent.spawn((
                crate::types::PlayerModel,
                WorldAssetRoot(assets.player_scene.clone()),
                // Смещение по Y — эмпирическое, подгоняет ноги модели к земле.
                // Поворот по Y — компенсация forward модели (-Z → +Z).
                Transform::from_xyz(0.0, -0.90, 0.0)
                    .with_rotation(Quat::from_rotation_y(MODEL_YAW_OFFSET)),
            ));
        });
}

fn cleanup_game(mut commands: Commands, ground: Query<Entity, With<Ground>>) {
    for entity in &ground {
        commands.entity(entity).despawn();
    }
}
