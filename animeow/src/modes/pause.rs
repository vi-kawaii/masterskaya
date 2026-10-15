use std::time::Duration;

use crate::states::{GameState, InGameState};
use bevy::prelude::*;

pub struct PausePlugin;

impl Plugin for PausePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, toggle_pause.run_if(in_state(GameState::InGame)))
            .add_systems(OnEnter(InGameState::Paused), setup_pause_ui)
            .add_systems(OnExit(InGameState::Paused), cleanup_pause_ui);
    }
}

#[derive(Component)]
struct PauseUi;

/// Пауза и снятие паузы делаются **в том же кадре**, что и переключение
/// стейта. Иначе между `next.set(Paused)` и `OnEnter(Paused)` успевает
/// пройти один `FixedUpdate` — и физика делает лишний тик.
///
/// `Time<Virtual>::pause()` останавливает и `Time`, и `Time<Fixed>`.
/// Дополнительно сбрасываем аккумулятор `Time<Fixed>`, чтобы не было
/// «хвоста» из уже накопленной, но не потраченной дельты.
fn toggle_pause(
    keys: Res<ButtonInput<KeyCode>>,
    current: Res<State<InGameState>>,
    mut next: ResMut<NextState<InGameState>>,
    mut virtual_time: ResMut<Time<Virtual>>,
    mut fixed_time: ResMut<Time<Fixed>>,
) {
    if !keys.just_pressed(KeyCode::Escape) {
        return;
    }
    match current.get() {
        InGameState::Playing => {
            println!("[pause] time paused");
            virtual_time.pause();
            // Сбросить накопленный overstep FixedUpdate,
            // чтобы не было «догоняющих» тиков физики после паузы.
            fixed_time.discard_overstep(Duration::MAX);
            next.set(InGameState::Paused);
        }
        InGameState::Paused => {
            println!("[pause] time unpaused");
            virtual_time.unpause();
            next.set(InGameState::Playing);
        }
    }
}

fn setup_pause_ui(mut commands: Commands) {
    commands
        .spawn((
            PauseUi,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("PAUSED"),
                TextFont {
                    font_size: FontSize::Px(64.0),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
            parent.spawn((
                Text::new("Press Esc to resume"),
                TextFont {
                    font_size: FontSize::Px(24.0),
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
            ));
        });
}

fn cleanup_pause_ui(mut commands: Commands, query: Query<Entity, With<PauseUi>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
