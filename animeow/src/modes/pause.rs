use crate::states::{GameState, InGameState};
use bevy::prelude::*;

pub struct PausePlugin;

impl Plugin for PausePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, toggle_pause.run_if(in_state(GameState::InGame)))
            .add_systems(
                OnEnter(InGameState::Paused),
                (setup_pause_ui, pause_time),
            )
            .add_systems(
                OnExit(InGameState::Paused),
                (cleanup_pause_ui, unpause_time),
            );
    }
}

#[derive(Component)]
struct PauseUi;

fn toggle_pause(
    keys: Res<ButtonInput<KeyCode>>,
    current: Res<State<InGameState>>,
    mut next: ResMut<NextState<InGameState>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        match current.get() {
            InGameState::Playing => next.set(InGameState::Paused),
            InGameState::Paused => next.set(InGameState::Playing),
        }
    }
}

/// Пауза физики, анимаций, cooldown-таймеров и всего, что зависит от Time.
/// `Time<Virtual>` управляет и обычным `Time`, и `Time<Fixed>`.
/// Когда виртуальное время на паузе, `FixedUpdate` получает delta = 0,
/// и avian (вместе с Tnua) не двигает тела.
fn pause_time(mut time: ResMut<Time<Virtual>>) {
    println!("[pause] time paused");
    time.pause();
}

fn unpause_time(mut time: ResMut<Time<Virtual>>) {
    println!("[pause] time unpaused");
    time.unpause();
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
