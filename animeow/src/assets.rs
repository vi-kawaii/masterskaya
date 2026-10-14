use bevy::prelude::*;

/// Ассеты, которые грузим вручную и следим за прогрессом.
/// Заполняется в `modes/loading.rs::start_loading`.
#[derive(Resource, Default)]
pub struct GameAssets {
    pub player_scene: Handle<WorldAsset>,
    pub island_scene: Handle<WorldAsset>,
}
