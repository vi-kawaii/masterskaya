pub mod dialog;
pub mod fight;
pub mod hud;
pub mod move_;

use crate::states::GameState;
use avian3d::prelude::*;
use bevy::prelude::*;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<crate::types::GameSettings>()
            .init_resource::<crate::types::Score>()
            .add_systems(OnEnter(GameState::InGame), setup_game)
            .add_systems(OnExit(GameState::InGame), cleanup_game);
    }
}

#[derive(Component)]
struct Ground;

fn setup_game(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Свет
    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, -0.5, 0.0)),
    ));

    // Пол — теперь с физикой.
    // Визуально: 50x50, толщина 0.2. Коллайдер — cuboid по полуразмерам.
    // Верхняя грань на y = 0.
    commands.spawn((
        Ground,
        RigidBody::Static,
        Collider::cuboid(25.0, 0.1, 25.0),
        Mesh3d(meshes.add(Cuboid::new(50.0, 0.2, 50.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
        Transform::from_xyz(0.0, -0.1, 0.0),
    ));
}

fn cleanup_game(mut commands: Commands, ground: Query<Entity, With<Ground>>) {
    for entity in &ground {
        commands.entity(entity).despawn();
    }
}
