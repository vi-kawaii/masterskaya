pub mod character;
pub mod shoot;
pub mod vehicle;
pub mod weapon;

use bevy::prelude::*;

pub struct ActorPlugin;

impl Plugin for ActorPlugin {
    fn build(&self, _app: &mut App) {}
}

#[derive(Component)]
pub struct Actor;
