use bevy::prelude::*;

use crate::states::AppState;

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Loading), setup_loading)
            .add_systems(
                Update,
                (update_loading_ui, finish_loading).run_if(in_state(AppState::Loading)),
            )
            .add_systems(OnExit(AppState::Loading), cleanup_loading);
    }
}

#[derive(Resource)]
struct LoadingTimer(Timer);

#[derive(Component)]
struct LoadingUi;

#[derive(Component)]
struct LoadingProgressText;

#[derive(Component)]
struct LoadingBarFill;

fn setup_loading(mut commands: Commands) {
    commands.insert_resource(LoadingTimer(Timer::from_seconds(5.0, TimerMode::Once)));

    commands
        .spawn((
            LoadingUi,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(20.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Loading..."),
                TextFont {
                    font_size: FontSize::Px(48.0),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            parent
                .spawn(Node {
                    width: Val::Px(400.0),
                    height: Val::Px(24.0),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                })
                .insert(BorderColor::all(Color::WHITE))
                .with_children(|bar| {
                    bar.spawn((
                        LoadingBarFill,
                        Node {
                            width: Val::Percent(0.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.0, 1.0, 0.0)),
                    ));
                });

            parent.spawn((
                LoadingProgressText,
                Text::new("0%"),
                TextFont {
                    font_size: FontSize::Px(20.0),
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
        });
}

fn update_loading_ui(
    timer: Res<LoadingTimer>,
    mut text_q: Query<&mut Text, With<LoadingProgressText>>,
    mut bar_q: Query<&mut Node, With<LoadingBarFill>>,
) {
    let elapsed = timer.0.elapsed_secs();
    let total = timer.0.duration().as_secs_f32();
    let t = (elapsed / total).clamp(0.0, 1.0);
    let percent = (t * 100.0).round() as u32;

    for mut text in &mut text_q {
        let s = format!("{}%   ({:.1} / {:.1} s)", percent, elapsed, total);
        if text.0 != s {
            text.0 = s;
        }
    }
    for mut node in &mut bar_q {
        node.width = Val::Percent(t * 100.0);
    }
}

fn finish_loading(
    time: Res<Time>,
    mut timer: ResMut<LoadingTimer>,
    mut next: ResMut<NextState<AppState>>,
) {
    timer.0.tick(time.delta());
    if timer.0.just_finished() {
        info!("Loading finished -> Editing");
        next.set(AppState::Editing);
    }
}

fn cleanup_loading(mut commands: Commands, query: Query<Entity, With<LoadingUi>>) {
    for e in &query {
        commands.entity(e).despawn();
    }
    commands.remove_resource::<LoadingTimer>();
}
