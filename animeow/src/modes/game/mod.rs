pub mod dialog;
pub mod fight;
pub mod hud;
pub mod move_;

use crate::assets::GameAssets;
use crate::states::GameState;
use avian3d::prelude::*;
use bevy::prelude::*;

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

/// Спавнит только остров. Игрок появится позже — по событию готовности коллайдеров.
fn spawn_island(mut commands: Commands, assets: Res<GameAssets>) {
    // Свет (тени выключены)
    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, -0.5, 0.0)),
    ));

    // Остров — статическая геометрия с коллайдерами из мешей.
    // FIX_INTERNAL_EDGES устраняет ghost collisions на внутренних рёбрах
    // тримеша (персонаж перестаёт «парить» над плоскими участками).
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

/// Обсервер: срабатывает, когда ColliderConstructorHierarchy закончил работу.
/// Спавним игрока именно здесь, а не в OnEnter.
/// Bevy 0.17+: Trigger → On, trigger.entity — это поле, а не метод.
pub fn on_island_ready(
    _trigger: On<ColliderConstructorHierarchyReady>,
    mut commands: Commands,
    assets: Res<GameAssets>,
    player_exists: Query<Entity, With<crate::types::Player>>,
) {
    // Защита от повторного спавна (событие может сработать несколько раз)
    if !player_exists.is_empty() {
        return;
    }

    // Спавним игрока — коллайдеры острова уже в мире.
    spawn_player_entity(&mut commands, &assets);
}

/// Вынесена отдельно, чтобы вызывать из обсервера.
pub fn spawn_player_entity(commands: &mut Commands, assets: &GameAssets) {
    commands
        .spawn((
            crate::types::Player,
            crate::actor::character::Grounded(false),
            crate::actor::character::MovementStats::default(),
            crate::types::DesiredYaw::default(),
            RigidBody::Kinematic,
            CustomPositionIntegration,
            Collider::capsule(0.4, 1.0),
            Transform::from_xyz(0.0, 2.0, 0.0),
            LinearVelocity::default(),
            Visibility::default(),
        ))
        .with_children(|parent| {
            parent.spawn((
                crate::types::PlayerModel,
                WorldAssetRoot(assets.player_scene.clone()),
                Transform::from_xyz(0.0, -0.9, 0.0),
            ));
        });
}

fn cleanup_game(mut commands: Commands, ground: Query<Entity, With<Ground>>) {
    for entity in &ground {
        commands.entity(entity).despawn();
    }
}
