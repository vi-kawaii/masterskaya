mod actor;
mod assets;
mod controller;
mod core;
mod modes;
mod states;
mod types;

use bevy::prelude::*;

use lab_app_shell::{build_app, AppShellConfig};
use states::{GameState, InGameState};

fn main() {
    println!("Animeow starting...");

    let mut app = build_app(AppShellConfig {
        title: "Animeow".into(),
        // Стартовый фон — ЧЁРНЫЙ. Это фон загрузки и промежутка до неё.
        // Синеватый фон мира включается в on_island_ready, когда игра готова.
        clear_color: Color::BLACK,
        ..default()
    });

    app.init_state::<GameState>()
        .add_sub_state::<InGameState>()
        // Обсервер: сцена полностью инстанцирована (меши, материалы, текстуры).
        .add_observer(modes::game::on_scene_ready)
        // Обсервер: коллайдеры построены. Переход в InGame — только
        // когда оба события пришли (см. IslandReady).
        .add_observer(modes::game::on_island_ready)
        .add_plugins((
            core::CorePlugin,
            modes::ModesPlugin,
            actor::ActorPlugin,
            controller::ControllerPlugin,
        ));

    app.run();
}
