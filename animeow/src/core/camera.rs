use crate::states::{GameState, InGameState};
use crate::types::{CameraRig, GameSettings, Player};
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InGame), spawn_camera_rig)
            .add_systems(
                Update,
                (
                    grab_cursor_on_click,
                    orbit_camera.run_if(in_state(InGameState::Playing)),
                    follow_player.run_if(in_state(InGameState::Playing)),
                ),
            )
            .add_systems(OnExit(GameState::InGame), cleanup_camera_rig);
    }
}

/// Высота точки, вокруг которой вращается камера, — от ЦЕНТРА капсулы.
/// Центр капсулы на 0.9 м от пола (полвысоты капсулы 1.8).
/// eye_height = 0.5  →  точка на 1.4 м от пола (уровень груди).
/// Было 1.6 — это 2.5 м от пола, выше макушки, поэтому казалось «высоко».
const EYE_HEIGHT: f32 = 0.5;

/// Спавним риг (позиция «плеча») и камеру как его ребёнка.
/// Позиция рига пересчитывается каждый кадр системой follow_player.
fn spawn_camera_rig(mut commands: Commands) {
    let rig = commands
        .spawn((
            CameraRig::default(),
            Transform::from_xyz(0.0, EYE_HEIGHT, 0.0),
            Visibility::default(),
        ))
        .id();

    commands.entity(rig).with_children(|parent| {
        parent.spawn((
            Camera3d::default(),
            Msaa::Off,
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));
    });
}

fn grab_cursor_on_click(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut cursor: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if mouse_buttons.just_pressed(MouseButton::Left) {
        if let Ok(mut options) = cursor.single_mut() {
            options.grab_mode = CursorGrabMode::Locked;
            options.visible = false;
        }
    }
}

/// Мышь и колесо пишут в target_*. Система плавно ведёт текущие значения
/// к целям через FPS-независимое экспоненциальное сглаживание.
fn orbit_camera(
    mut mouse_motion: MessageReader<MouseMotion>,
    mut mouse_wheel: MessageReader<MouseWheel>,
    settings: Res<GameSettings>,
    time: Res<Time>,
    mut rig_query: Query<(&mut CameraRig, &Children)>,
    mut camera_query: Query<&mut Transform, (With<Camera3d>, Without<CameraRig>)>,
) {
    let mut delta = Vec2::ZERO;
    for ev in mouse_motion.read() {
        delta += ev.delta;
    }

    let mut zoom_delta = 0.0_f32;
    for ev in mouse_wheel.read() {
        zoom_delta += ev.y;
    }

    let dt = time.delta_secs();
    let alpha = 1.0 - (-settings.camera_smoothing * dt).exp();

    for (mut rig, children) in &mut rig_query {
        if delta != Vec2::ZERO {
            rig.target_yaw -= delta.x * settings.mouse_sensitivity;
            rig.target_pitch -= delta.y * settings.mouse_sensitivity;
            rig.target_pitch = rig.target_pitch.clamp(rig.pitch_min, rig.pitch_max);
        }

        if zoom_delta != 0.0 {
            rig.target_distance -= zoom_delta * rig.zoom_step;
            rig.target_distance = rig
                .target_distance
                .clamp(rig.min_distance, rig.max_distance);
        }

        rig.yaw = lerp_f32(rig.yaw, rig.target_yaw, alpha);
        rig.pitch = lerp_f32(rig.pitch, rig.target_pitch, alpha);
        rig.distance = lerp_f32(rig.distance, rig.target_distance, alpha);

        let rotation = Quat::from_euler(EulerRot::YXZ, rig.yaw, rig.pitch, 0.0);
        let offset = rotation * Vec3::new(0.0, 0.0, rig.distance);

        for &child in children {
            if let Ok(mut cam_transform) = camera_query.get_mut(child) {
                cam_transform.translation = offset;
                cam_transform.rotation = rotation;
            }
        }
    }
}

#[inline]
fn lerp_f32(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Жёстко ставит риг в позицию игрока + EYE_HEIGHT.
/// Никакого сглаживания здесь — иначе укачивает.
fn follow_player(
    player_query: Query<&Transform, (With<Player>, Without<CameraRig>)>,
    mut rig_query: Query<&mut Transform, With<CameraRig>>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let target = player_transform.translation + Vec3::new(0.0, EYE_HEIGHT, 0.0);
    for mut rig_transform in &mut rig_query {
        rig_transform.translation = target;
    }
}

fn cleanup_camera_rig(mut commands: Commands, query: Query<Entity, With<CameraRig>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
