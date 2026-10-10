use bevy::prelude::*;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Vehicle;

/// Маркер узла с моделью персонажа — ребёнок физической капсулы.
/// Вращается отдельной системой, чтобы не трогать физику.
#[derive(Component)]
pub struct PlayerModel;

/// Целевой yaw модели (радианы). Пишет `character_movement`,
/// читает `rotate_model_towards_movement`.
#[derive(Component, Default)]
pub struct DesiredYaw(pub f32);

/// Корень TPS-камеры. Родитель `Camera3d`.
///
/// Разделены «текущее» и «цель»: мышь/колесо двигают target_*,
/// система `orbit_camera` плавно ведёт текущие значения к ним.
#[derive(Component)]
pub struct CameraRig {
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,

    pub target_yaw: f32,
    pub target_pitch: f32,
    pub target_distance: f32,

    pub min_distance: f32,
    pub max_distance: f32,
    pub zoom_step: f32,
    pub pitch_min: f32,
    pub pitch_max: f32,
}

impl Default for CameraRig {
    fn default() -> Self {
        // Стартовая дистанция 4.5 м — на 1.8-метровом персонаже
        // смотрится естественно. 6.0 была «широко», персонаж мелкий.
        let yaw = 0.0;
        let pitch = -0.3;
        let distance = 4.5;
        Self {
            yaw,
            pitch,
            distance,
            target_yaw: yaw,
            target_pitch: pitch,
            target_distance: distance,
            min_distance: 2.0,
            max_distance: 12.0,
            zoom_step: 0.5,
            pitch_min: -1.4,
            pitch_max: 1.4,
        }
    }
}

#[derive(Resource, Default)]
pub struct Score {
    pub value: u32,
}

#[derive(Resource)]
pub struct GameSettings {
    pub run_speed: f32,
    pub sprint_speed: f32,
    pub jump_velocity: f32,
    pub gravity: f32,
    pub mouse_sensitivity: f32,
    pub vehicle_speed: f32,
    pub turn_speed: f32,
    pub camera_smoothing: f32,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            run_speed: 7.0,
            sprint_speed: 10.0,
            jump_velocity: 6.0,
            gravity: -20.0,
            mouse_sensitivity: 0.003,
            vehicle_speed: 25.0,
            turn_speed: 12.0,
            camera_smoothing: 25.0,
        }
    }
}
