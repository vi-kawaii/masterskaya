use crate::core::input::InputState;
use crate::states::{GameState, InGameState};
use crate::types::Player;
use bevy::prelude::*;
use bevy_tnua::builtins::{TnuaBuiltinJump, TnuaBuiltinWalk};
use bevy_tnua::prelude::*;

use super::scheme::CharacterControlScheme;

pub struct PlayerControllerPlugin;

impl Plugin for PlayerControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
                FixedUpdate,
                tnua_movement
                    .run_if(in_state(InGameState::Playing)),
            )
            .add_systems(Update, flatten_materials.run_if(in_state(InGameState::Playing)))
            .add_systems(OnExit(GameState::InGame), cleanup_player);
    }
}

fn tnua_movement(
    input: Res<InputState>,
    camera_query: Query<&Transform, (With<Camera3d>, Without<Player>)>,
    mut query: Query<&mut TnuaController<CharacterControlScheme>, With<Player>>,
) {
    let Ok(camera_transform) = camera_query.single() else {
        return;
    };
    let Ok(mut controller) = query.single_mut() else {
        return;
    };

    let mut camera_forward = camera_transform.forward().as_vec3();
    camera_forward.y = 0.0;
    let camera_forward = camera_forward.normalize_or_zero();

    let mut camera_right = camera_transform.right().as_vec3();
    camera_right.y = 0.0;
    let camera_right = camera_right.normalize_or_zero();

    let move_dir = camera_forward * input.move_dir.y + camera_right * input.move_dir.x;
    let move_dir = move_dir.normalize_or_zero();

    // Скорость НЕ умножаем — она задана в конфиге (config.speed).
    controller.initiate_action_feeding();

    controller.basis = TnuaBuiltinWalk {
        desired_motion: move_dir,   // только направление!
        desired_forward: Dir3::new(move_dir).ok(),
        ..default()
    };

    if input.jump_pressed {
        controller.action(CharacterControlScheme::Jump(TnuaBuiltinJump::default()));
    }
}

fn flatten_materials(
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<&MeshMaterial3d<StandardMaterial>>,
) {
    for material_handle in &query {
        if let Some(mut material) = materials.get_mut(&material_handle.0) {
            if !material.unlit {
                material.unlit = true;
            }
        }
    }
}

fn cleanup_player(mut commands: Commands, query: Query<Entity, With<Player>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
