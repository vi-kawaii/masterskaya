pub mod character;
pub mod shoot;
pub mod vehicle;

use bevy::prelude::*;

pub struct ActorPlugin;

impl Plugin for ActorPlugin {
    fn build(&self, _app: &mut App) {}
}

/// Общая сущность: игрок, бот, NPC.
/// Пока пустой маркер — наполним позже.
#[derive(Component)]
pub struct Actor;
