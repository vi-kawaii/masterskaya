mod actor;
mod assets;
mod controller;
mod core;
mod modes;
mod states;
mod types;

use bevy::prelude::*;

use bevy_asset_loader::prelude::*;
use lab_app_shell::{build_app, AppShellConfig};
use states::{GameState, InGameState};

fn main() {
    println!("Animeow starting...");

    let mut app = build_app(AppShellConfig {
        title: "Animeow".into(),
        clear_color: Color::srgb(0.05, 0.05, 0.1),
        ..default()
    });

    app.init_state::<GameState>()
        .add_sub_state::<InGameState>()
        .add_loading_state(
            LoadingState::new(GameState::Loading)
                .continue_to_state(GameState::InGame)
                .load_collection::<assets::GameAssets>(),
        )
        .add_plugins((
            core::CorePlugin,
            modes::ModesPlugin,
            actor::ActorPlugin,
            controller::ControllerPlugin,
        ));

    app.run();
}
