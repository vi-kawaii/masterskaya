//! lab-app-shell — общий каркас Bevy-приложения для мастерской.

use std::num::NonZero;

use bevy::dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin};
use bevy::prelude::*;
use bevy::window::{MonitorSelection, PresentMode, WindowMode};
use bevy::winit::{UpdateMode, WinitSettings};

pub struct AppShellConfig {
    pub title: String,
    pub clear_color: Color,
    pub fps_font_size: f32,
    pub fps_color: Color,
}

impl Default for AppShellConfig {
    fn default() -> Self {
        Self {
            title: "App".into(),
            clear_color: Color::BLACK,
            fps_font_size: 24.0,
            fps_color: Color::srgb(0.0, 1.0, 0.0),
        }
    }
}

pub fn build_app(config: AppShellConfig) -> App {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: config.title,
            // Возвращаем BorderlessFullscreen — падений нет.
            mode: WindowMode::BorderlessFullscreen(MonitorSelection::Current),
            // AutoNoVsync: отключает vsync, но без риска tearing.
            present_mode: PresentMode::Immediate,
            desired_maximum_frame_latency: NonZero::new(1),
            ..default()
        }),
        ..default()
    }));

    // Цикл событий не спит — гоним кадры максимально часто.
    app.insert_resource(WinitSettings {
        focused_mode: UpdateMode::Continuous,
        unfocused_mode: UpdateMode::reactive_low_power(std::time::Duration::from_millis(16)),
    });

    app.add_plugins(FpsOverlayPlugin {
        config: FpsOverlayConfig {
            text_config: TextFont {
                font_size: FontSize::Px(config.fps_font_size),
                ..default()
            },
            text_color: config.fps_color,
            ..default()
        },
    });

    app.insert_resource(ClearColor(config.clear_color));

    app
}
