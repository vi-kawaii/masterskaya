use bevy::prelude::*;

/// Ассеты, которые грузим вручную и следим за прогрессом.
/// Заполняется в `modes/loading.rs::start_loading`.
#[derive(Resource, Default)]
pub struct GameAssets {
    pub player_scene: Handle<WorldAsset>,
    pub island_scene: Handle<WorldAsset>,

    pub car_scene: Handle<WorldAsset>,
    pub wheel1_scene: Handle<WorldAsset>,
    pub wheel2_scene: Handle<WorldAsset>,
    pub wheel3_scene: Handle<WorldAsset>,
    pub wheel4_scene: Handle<WorldAsset>,
}
