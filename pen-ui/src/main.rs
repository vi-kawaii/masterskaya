use bevy::prelude::*;
use lab_app_shell::{build_app, AppShellConfig};

fn main() {
    println!("pen-ui starting...");

    let mut app = build_app(AppShellConfig {
        title: "pen".into(),
        clear_color: Color::srgb(0.1, 0.1, 0.12),
        ..default()
    });

    app.run();
}
