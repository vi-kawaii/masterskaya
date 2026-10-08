use crate::actor::character::{Grounded, MovementStats};
use crate::assets::GameAssets;
use crate::core::input::InputState;
use crate::states::{GameState, InGameState};
use crate::types::{GameSettings, Player};
use avian3d::prelude::*;
use bevy::prelude::*;

pub struct PlayerControllerPlugin;

impl Plugin for PlayerControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InGame), spawn_player)
            .add_systems(
                FixedUpdate,
                character_movement
                    .chain()
                    .run_if(in_state(InGameState::Playing)),
            )
            .add_systems(OnExit(GameState::InGame), cleanup_player);
    }
}

/// Спавним «невидимую капсулу» — родителя с физикой,
/// и вешаем на неё glTF-сцену как визуал.
fn spawn_player(mut commands: Commands, assets: Res<GameAssets>) {
    commands
        .spawn((
            Player,
            Grounded(false),
            MovementStats::default(),
            RigidBody::Kinematic,
            CustomPositionIntegration,
            Collider::capsule(0.4, 1.0),
            Transform::from_xyz(0.0, 2.0, 0.0),
            LinearVelocity::default(),
            Visibility::default(),
        ))
        .with_children(|parent| {
            // glTF-сцена садится в начало координат родителя.
            // y = -0.9 — если модель стоит «ногами в 0», а центр капсулы
            // должен быть в середине тела. Подгони под свою модель.
            parent.spawn((
                WorldAssetRoot(assets.player_scene.clone()),
                Transform::from_xyz(0.0, -0.9, 0.0),
            ));
        });
}

fn character_movement(
    input: Res<InputState>,
    settings: Res<GameSettings>,
    time: Res<Time>,
    move_and_slide: MoveAndSlide,
    camera_query: Query<&Transform, (With<Camera3d>, Without<Player>)>,
    mut query: Query<(
        Entity,
        &mut Transform,
        &mut LinearVelocity,
        &Collider,
        &mut Grounded,
    ), With<Player>>,
) {
    let camera_transform = match camera_query.single() {
        Ok(cam) => cam,
        Err(_) => return,
    };

    let mut camera_forward = camera_transform.forward().as_vec3();
    camera_forward.y = 0.0;
    let camera_forward = camera_forward.normalize_or_zero();

    let mut camera_right = camera_transform.right().as_vec3();
    camera_right.y = 0.0;
    let camera_right = camera_right.normalize_or_zero();

    let dt = time.delta();
    let dt_secs = dt.as_secs_f32();

    for (entity, mut transform, mut velocity, collider, mut grounded) in &mut query {
        let target_dir = (camera_forward * input.move_dir.y + camera_right * input.move_dir.x)
            .normalize_or_zero();

        let speed = if input.sprint {
            settings.sprint_speed
        } else {
            settings.run_speed
        };

        let target_velocity = target_dir * speed;
        velocity.x = target_velocity.x;
        velocity.z = target_velocity.z;

        if grounded.0 && velocity.y <= 0.0 {
            velocity.y = 0.0;
            if input.jump_pressed {
                velocity.y = settings.jump_velocity;
            }
        } else {
            velocity.y += settings.gravity * dt_secs;
        }

        let filter = SpatialQueryFilter::from_excluded_entities([entity]);
        let output = move_and_slide.move_and_slide(
            collider,
            transform.translation,
            transform.rotation,
            velocity.0,
            dt,
            &MoveAndSlideConfig::default(),
            &filter,
            |_hit| MoveAndSlideHitResponse::Accept,
        );

        transform.translation = output.position;
        velocity.0 = output.projected_velocity;

        let hit = move_and_slide.cast_move(
            collider,
            transform.translation,
            transform.rotation,
            Vec3::NEG_Y * 0.95,
            0.01,
            &filter,
        );
        grounded.0 = hit.is_some();
    }
}

fn cleanup_player(mut commands: Commands, query: Query<Entity, With<Player>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
