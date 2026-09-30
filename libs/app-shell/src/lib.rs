//! lab-app-shell — общий каркас Bevy-приложения для мастерской.
//!
//! Инкапсулирует то, что повторяется в каждом проекте:
//! - borderless fullscreen окно на текущем мониторе
//! - FPS-оверлей с настраиваемым цветом и размером
//! - ClearColor
//! - заголовок окна
//!
//! Проект создаёт App через `build_app(config)` и добавляет свои
//! плагины, стейты, системы поверх.

use bevy::dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin};
use bevy::prelude::*;
use bevy::window::{MonitorSelection, WindowMode};

/// Конфигурация каркаса. Всё, что может отличаться между проектами.
pub struct AppShellConfig {
    /// Заголовок окна.
    pub title: String,
    /// Цвет фона по умолчанию.
    pub clear_color: Color,
    /// Размер шрифта FPS-оверлея в пикселях.
    pub fps_font_size: f32,
    /// Цвет текста FPS-оверлея.
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

/// Собирает готовый `App` с общим каркасом.
///
/// Проект дальше добавляет свои плагины, стейты, системы:
///
/// ```ignore
/// let mut app = lab_app_shell::build_app(AppShellConfig {
///     title: "VTuber".into(),
///     clear_color: Color::BLACK,
///     ..default()
/// });
/// app.init_state::<AppState>();
/// app.add_plugins(MyPlugins);
/// app.run();
/// ```
pub fn build_app(config: AppShellConfig) -> App {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: config.title,
            mode: WindowMode::BorderlessFullscreen(MonitorSelection::Current),
            ..default()
        }),
        ..default()
    }));

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
