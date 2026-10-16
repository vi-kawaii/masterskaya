use bevy::prelude::*;
use std::path::PathBuf;

use crate::scene::{RebuildScene, SyncTransforms};
use crate::states::AppState;
use crate::types::VtuberDocument;

/// Куда пишем и откуда читаем документ.
/// Путь относительный — от cwd процесса (обычно корень крейта `v/`).
/// При запуске через `cargo run -p v` из корня мастерской cwd будет
/// корнем мастерской; но `lab run` заходит в директорию пакета,
/// поэтому файл окажется в `v/document.ron`.
pub const DEFAULT_DOCUMENT_PATH: &str = "document.ron";

pub struct PersistencePlugin;

impl Plugin for PersistencePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (save_document_system, load_document_system)
                .run_if(in_state(AppState::Editing)),
        );
    }
}

/// `Ctrl+S` — сохранить документ.
///
/// Проверяем именно `ControlLeft || ControlRight` вместе с `S`,
/// чтобы случайное нажатие `S` в других контекстах ничего не делало.
/// (В будущем, когда появятся текстовые поля, `S` надо будет фильтровать
///  по фокусу ввода.)
fn save_document_system(
    keys: Res<ButtonInput<KeyCode>>,
    doc: Res<VtuberDocument>,
) {
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    if !ctrl || !keys.just_pressed(KeyCode::KeyS) {
        return;
    }

    let path = PathBuf::from(DEFAULT_DOCUMENT_PATH);
    match doc.save_to_file(&path) {
        Ok(()) => info!("[save] document saved to {}", path.display()),
        Err(e) => error!("[save] failed: {e}"),
    }
}

/// `Ctrl+O` — загрузить документ.
///
/// После успешной загрузки:
///   - заменяем ресурс `VtuberDocument` целиком;
///   - просим сцену пересобраться (`RebuildScene`), чтобы ECS-сущности
///     пересоздались из нового списка объектов;
///   - просим `SyncTransforms`, чтобы трансформы соответствовали документу
///     (на случай, если rebuild сам не выставит их — сейчас выставит,
///      но сообщение не помешает).
fn load_document_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut doc: ResMut<VtuberDocument>,
    mut rebuild_ev: MessageWriter<RebuildScene>,
    mut sync_ev: MessageWriter<SyncTransforms>,
) {
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    if !ctrl || !keys.just_pressed(KeyCode::KeyO) {
        return;
    }

    let path = PathBuf::from(DEFAULT_DOCUMENT_PATH);
    match VtuberDocument::load_from_file(&path) {
        Ok(loaded) => {
            info!(
                "[load] document loaded from {} ({} objects, format_version={})",
                path.display(),
                loaded.objects.len(),
                loaded.format_version
            );
            *doc = loaded;
            rebuild_ev.write(RebuildScene);
            sync_ev.write(SyncTransforms);
        }
        Err(e) => error!("[load] failed: {e}"),
    }
}