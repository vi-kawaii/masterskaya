use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

/// Всё, что грузим на старте. Пока пусто — модель добавим позже.
/// Как только появится .glb — раскомментируй поля и положи файл в assets/models/.
#[derive(AssetCollection, Resource)]
pub struct GameAssets {
    // #[asset(path = "models/player.glb#Scene0")]
    // pub player_scene: Handle<Scene>,
}
