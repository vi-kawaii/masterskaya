use bevy::prelude::*;
use lab_app_shell::{build_app, AppShellConfig};

fn main() {
    println!("cinema-ui starting...");

    let mut app = build_app(AppShellConfig {
        title: "cinema".into(),
        clear_color: Color::srgb(0.05, 0.05, 0.07),
        ..default()
    });

    app.run();
}
