use crate::actor::character::Grounded;
use crate::types::Player;
use bevy::prelude::*;

/// Отладочный текст с координатами игрока и флагом Grounded.
#[derive(Component)]
pub struct DebugCoordsText;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_debug_coords)
            .add_systems(Update, update_debug_coords);
    }
}

fn spawn_debug_coords(mut commands: Commands) {
    commands.spawn((
        DebugCoordsText,
        Text::new("Pos: ..."),
        TextFont {
            font_size: FontSize::Px(16.0),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 0.0)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(12.0),
            bottom: Val::Px(12.0),
            ..default()
        },
    ));
}

/// Читает позицию игрока и Grounded, пишет в текст.
/// Работает в Update, читает актуальный Transform после FixedUpdate.
fn update_debug_coords(
    player: Query<(&Transform, &Grounded), With<Player>>,
    mut text: Query<&mut Text, With<DebugCoordsText>>,
) {
    let Ok((t, g)) = player.single() else {
        return;
    };
    let p = t.translation;
    for mut txt in &mut text {
        txt.0 = format!(
            "Pos: {:.1}, {:.1}, {:.1}   Grounded: {}",
            p.x, p.y, p.z, g.0
        );
    }
}
