mod editor;
mod loading;
mod mode_switch;
mod persistence;
mod playback;
mod scene;
mod states;
mod types;

use bevy::picking::mesh_picking::MeshPickingPlugin;
use bevy::prelude::*;
use std::path::PathBuf;

use lab_app_shell::{build_app, AppShellConfig};
use persistence::DEFAULT_DOCUMENT_PATH;
use scene::make_triangle;
use states::AppState;
use types::VtuberDocument;

fn main() {
    println!("VTuber starting...");

    let mut app = build_app(AppShellConfig {
        title: "VTuber".into(),
        clear_color: Color::BLACK,
        ..default()
    });

    app.init_state::<AppState>()
        // Пытаемся прочитать `document.ron`. Если файла нет — пустой
        // документ с актуальной версией формата. `seed_document_if_empty`
        // ниже либо доложит стартовые объекты (если пусто), либо нет
        // (если что-то загрузилось из файла).
        .insert_resource(initial_document())
        .add_systems(PreStartup, seed_document_if_empty)
        .add_plugins((
            // В Bevy 0.19 `MeshPickingPlugin` НЕ входит в `DefaultPlugins`.
            // Нужен, чтобы `MeshRayCast` в `selection_system` наполнялся
            // попаданиями и клики выделяли объекты.
            MeshPickingPlugin,
            loading::LoadingPlugin,
            scene::ScenePlugin,
            editor::EditorPlugin,
            playback::PlaybackPlugin,
            mode_switch::ModeSwitchPlugin,
            persistence::PersistencePlugin,
        ));

    app.run();
}

/// Стартовый документ: если файл `document.ron` существует — читаем его;
/// иначе — пустой документ с актуальной `format_version`.
///
/// Читаем синхронно в `main` до `app.run()`: документ маленький, и это
/// избавляет от возни с асинхронной загрузкой в `Startup`. Когда документы
/// станут большими — переедем в `Startup` с прогрессом.
fn initial_document() -> VtuberDocument {
    let path = PathBuf::from(DEFAULT_DOCUMENT_PATH);

    if path.exists() {
        match VtuberDocument::load_from_file(&path) {
            Ok(doc) => {
                info!(
                    "[startup] document loaded from {} ({} objects, format_version={})",
                    path.display(),
                    doc.objects.len(),
                    doc.format_version,
                );
                return doc;
            }
            Err(e) => {
                error!(
                    "[startup] failed to load {}: {e}. Starting empty.",
                    path.display()
                );
            }
        }
    }

    VtuberDocument::new_empty("untitled")
}

/// Если документ пуст — кладём два стартовых треугольника.
/// Если что-то загрузили из файла — не трогаем.
///
/// Большой по центру крутится, маленький слева стоит — чтобы было
/// по чему кликать и что циклировать по `Tab`.
fn seed_document_if_empty(mut doc: ResMut<VtuberDocument>) {
    if !doc.objects.is_empty() {
        return;
    }

    let mut big = make_triangle("Triangle A", 100.0);
    big.transform.translation = [0.0, 0.0, 0.0];
    big.playback.spin_speed = 1.5;
    doc.add_object(big);

    let mut small = make_triangle("Triangle B", 50.0);
    small.transform.translation = [-200.0, 0.0, 0.0];
    small.playback.spin_speed = 0.0;
    doc.add_object(small);
}