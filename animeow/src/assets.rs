use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

/// Всё, что грузим на старте.
/// Модель персонажа лежит в `assets/models/anime-girl.glb`.
/// `#Scene0` — первая сцена внутри glTF (обычно корневой узел).
#[derive(AssetCollection, Resource)]
pub struct GameAssets {
    #[asset(path = "models/anime-girl.glb#Scene0")]
    pub player_scene: Handle<WorldAsset>,
}
