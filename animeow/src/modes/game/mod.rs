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
use bevy::world_serialization::WorldInstanceReady;
use bevy_tnua::prelude::*;
use bevy_tnua_avian3d::TnuaAvian3dSensorShape;

/// Цвет фона мира. Отличается от фона загрузки (чёрный в main.rs).
const WORLD_CLEAR_COLOR: Color = Color::srgb(0.05, 0.05, 0.1);

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<crate::types::GameSettings>()
            .init_resource::<crate::types::Score>()
            .init_resource::<IslandReady>()
            .add_systems(
                Update,
                finish_loading.run_if(in_state(GameState::Loading)),
            )
            .add_systems(OnExit(GameState::InGame), cleanup_game);
    }
}

#[derive(Component)]
pub struct Ground;

/// Флаг готовности острова: сцена инстанцирована + коллайдеры построены.
#[derive(Resource, Default)]
pub struct IslandReady {
    pub scene_ready: bool,
    pub colliders_ready: bool,
    pub player_spawned: bool,
    pub finished: bool,
}

impl IslandReady {
    pub fn is_ready(&self) -> bool {
        self.scene_ready && self.colliders_ready
    }
}

pub fn spawn_island(commands: &mut Commands, assets: &GameAssets) {
    println!("[game] spawn_island");
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

pub fn on_scene_ready(
    _trigger: On<WorldInstanceReady>,
    mut ready: ResMut<IslandReady>,
) {
    println!("[game] on_scene_ready — сцена инстанцирована");
    ready.scene_ready = true;
}

pub fn on_island_ready(
    _trigger: On<ColliderConstructorHierarchyReady>,
    mut commands: Commands,
    assets: Res<GameAssets>,
    config_handle: Res<CharacterControlConfigHandle>,
    mut ready: ResMut<IslandReady>,
) {
    if ready.player_spawned {
        return;
    }

    println!("[game] on_island_ready — коллайдеры готовы, спавним игрока");
    spawn_player_entity(&mut commands, &assets, &config_handle);
    ready.colliders_ready = true;
    ready.player_spawned = true;
}

/// Переключение в InGame сразу, как только остров готов.
/// Визуальный промежуток между удалением Loading UI и появлением
/// моделей на экране будет убран позже — пока оставляем как есть.
fn finish_loading(
    mut ready: ResMut<IslandReady>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
    mut clear: ResMut<ClearColor>,
    ui: Query<Entity, With<crate::modes::loading::LoadingUi>>,
    cam: Query<Entity, With<crate::modes::loading::LoadingCamera>>,
) {
    if !ready.is_ready() || ready.finished {
        return;
    }

    println!("[game] остров готов — переход в InGame");

    clear.0 = WORLD_CLEAR_COLOR;

    for e in &ui {
        commands.entity(e).despawn();
    }
    for e in &cam {
        commands.entity(e).despawn();
    }

    ready.finished = true;
    next_state.set(GameState::InGame);
}

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
                Transform::from_xyz(0.0, -0.92, 0.0)
                    .with_rotation(Quat::from_rotation_y(MODEL_YAW_OFFSET)),
            ));
        });
}

fn cleanup_game(mut commands: Commands, ground: Query<Entity, With<Ground>>) {
    for entity in &ground {
        commands.entity(entity).despawn();
    }
}
