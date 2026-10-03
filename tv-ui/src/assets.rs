// tv-ui/src/assets.rs

use gpui_kit::{AssetSource, Result, SharedString};
use rust_embed::RustEmbed;
use std::borrow::Cow;

/// Встраиваем папку assets/ из корня воркспейса (tv/assets/).
/// Путь указывается относительно Cargo.toml крейта tv-ui,
/// поэтому ../assets указывает на tv/assets/.
#[derive(RustEmbed)]
#[folder = "../assets"]
pub struct TvAssets;

impl AssetSource for TvAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        // path приходит как "assets/overlay.png" — убираем префикс
        let clean_path = path.strip_prefix("assets/").unwrap_or(path);

        if let Some(file) = TvAssets::get(clean_path) {
            Ok(Some(file.data))
        } else {
            eprintln!("[tv-ui] Ассет не найден: {path} (искали {clean_path})");
            Ok(None)
        }
    }

    fn list(&self, _path: &str) -> Result<Vec<SharedString>> {
        Ok(Vec::new())
    }
}
