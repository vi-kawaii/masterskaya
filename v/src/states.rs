use bevy::prelude::*;

/// Глобальное состояние приложения.
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    Loading,
    Editing,
    Playing,
}
