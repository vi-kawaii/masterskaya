mod editor;
mod loading;
mod mode_switch;
mod playback;
mod scene;
mod states;
mod types;

use bevy::prelude::*;

use lab_app_shell::{build_app, AppShellConfig};
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
        .add_systems(PreStartup, seed_document)
        .add_plugins((
            loading::LoadingPlugin,
            scene::ScenePlugin,
            editor::EditorPlugin,
            playback::PlaybackPlugin,
            mode_switch::ModeSwitchPlugin,
        ));

    app.run();
}

/// Кладём в документ стартовый треугольник до старта сцены.
fn seed_document(mut doc: ResMut<VtuberDocument>) {
    if doc.objects.is_empty() {
        let mut tri = make_triangle("Triangle", 100.0);
        tri.playback.spin_speed = 1.5;
        doc.add_object(tri);
    }
}
