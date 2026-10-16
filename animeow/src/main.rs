mod actor;
mod assets;
mod controller;
mod core;
mod modes;
mod quest;
mod states;
mod types;
mod ui;

use bevy::prelude::*;

use lab_app_shell::{build_app, AppShellConfig};
use states::{GameState, InGameState};

fn main() {
    println!("Animeow starting...");

    let mut app = build_app(AppShellConfig {
        title: "Animeow".into(),
        clear_color: Color::BLACK,
        ..default()
    });

    app.init_state::<GameState>()
        .add_sub_state::<InGameState>()
        .add_observer(modes::game::on_scene_ready)
        .add_observer(modes::game::on_island_ready)
        .add_plugins((
            core::CorePlugin,
            modes::ModesPlugin,
            actor::ActorPlugin,
            controller::ControllerPlugin,
            ui::UiPlugin,
            quest::QuestPlugin,
        ));

    app.run();
}