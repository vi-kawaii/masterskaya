use bevy::prelude::*;

use crate::actor::weapon::WeaponDef;
use crate::quest::QuestDef;

#[derive(Resource, Default)]
pub struct GameAssets {
    pub player_scene: Handle<WorldAsset>,
    pub island_scene: Handle<WorldAsset>,

    pub car_scene: Handle<WorldAsset>,
    pub wheel1_scene: Handle<WorldAsset>,
    pub wheel2_scene: Handle<WorldAsset>,
    pub wheel3_scene: Handle<WorldAsset>,
    pub wheel4_scene: Handle<WorldAsset>,

    pub weapon_pistol: Handle<WeaponDef>,
    pub weapon_rifle: Handle<WeaponDef>,
    pub weapon_shotgun: Handle<WeaponDef>,

    pub quest_first_steps: Handle<QuestDef>,
}