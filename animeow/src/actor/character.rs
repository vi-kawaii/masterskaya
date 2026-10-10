use bevy::prelude::{Component, Vec3};

/// Базовые скорости для locomotion.
/// Пока только заготовка — реально используются `GameSettings`.
#[derive(Component)]
pub struct MovementStats {
    pub walk_speed: f32,
    pub run_speed: f32,
    pub sprint_speed: f32,
}

impl Default for MovementStats {
    fn default() -> Self {
        Self {
            walk_speed: 4.0,
            run_speed: 7.0,
            sprint_speed: 10.0,
        }
    }
}

/// Здоровье. Общее для игрока, NPC, ботов.
#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Default for Health {
    fn default() -> Self {
        Self {
            current: 100.0,
            max: 100.0,
        }
    }
}

/// Стоит ли персонаж на земле (cast_move вниз).
/// Пишется системой `character_movement` в FixedUpdate.
#[derive(Component, Default)]
pub struct Grounded(pub bool);
