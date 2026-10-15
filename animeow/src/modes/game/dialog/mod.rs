use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

use crate::states::InGameState;

pub struct DialogPlugin;

impl Plugin for DialogPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActiveDialog>()
            // Opening the dialog — only in Playing.
            .add_systems(
                Update,
                start_debug_dialog.run_if(in_state(InGameState::Playing)),
            )
            // UI and input — only in Dialog.
            .add_systems(OnEnter(InGameState::Dialog), setup_dialog_ui)
            .add_systems(
                Update,
                (handle_dialog_input, update_dialog_ui)
                    .chain()
                    .run_if(in_state(InGameState::Dialog)),
            )
            .add_systems(OnExit(InGameState::Dialog), cleanup_dialog_ui);
    }
}

// ============================================================
//  DATA MODEL
// ============================================================

#[derive(Debug, Clone, Deserialize)]
pub struct DialogTree {
    pub id: String,
    pub start_node: String,
    pub nodes: HashMap<String, DialogNode>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DialogNode {
    pub text: String,
    #[serde(default)]
    pub options: Vec<DialogOption>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DialogOption {
    pub text: String,
    #[serde(default)]
    pub next_node: String,
}

// ============================================================
//  ACTIVE DIALOG
// ============================================================

#[derive(Resource, Default)]
pub struct ActiveDialog {
    pub tree: Option<DialogTree>,
    pub current_node_id: String,
}

// ============================================================
//  UI COMPONENTS
// ============================================================

#[derive(Component)]
struct DialogUiRoot;

#[derive(Component)]
struct DialogText;

#[derive(Component)]
struct DialogOptionsRoot;

#[derive(Component)]
struct DialogOptionHint;

// ============================================================
//  DEBUG START
// ============================================================

fn start_debug_dialog(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<InGameState>>,
    mut active_dialog: ResMut<ActiveDialog>,
) {
    if !keys.just_pressed(KeyCode::KeyT) {
        return;
    }
    println!("[dialog] starting test dialog");

    // Hardcoded test dialog. Will be replaced by
    // assets/dialogs/*.ron loading + NPC trigger later.
    let mut nodes = HashMap::new();
    nodes.insert(
        "start".to_string(),
        DialogNode {
            text: "Hello, traveler! What do you need?".to_string(),
            options: vec![
                DialogOption {
                    text: "I need help.".to_string(),
                    next_node: "help".to_string(),
                },
                DialogOption {
                    text: "Just passing by.".to_string(),
                    next_node: "bye".to_string(),
                },
            ],
        },
    );
    nodes.insert(
        "help".to_string(),
        DialogNode {
            text: "I can point you the way. Where are you headed?".to_string(),
            options: vec![
                DialogOption {
                    text: "To the nearest village.".to_string(),
                    next_node: "village".to_string(),
                },
                DialogOption {
                    text: "Nowhere, thanks.".to_string(),
                    next_node: "bye".to_string(),
                },
            ],
        },
    );
    nodes.insert(
        "village".to_string(),
        DialogNode {
            text: "Head north. You'll cross a river, then the village.".to_string(),
            options: vec![],
        },
    );
    nodes.insert(
        "bye".to_string(),
        DialogNode {
            text: "Safe travels!".to_string(),
            options: vec![],
        },
    );

    active_dialog.tree = Some(DialogTree {
        id: "npc_test_1".to_string(),
        start_node: "start".to_string(),
        nodes,
    });
    active_dialog.current_node_id = "start".to_string();

    next_state.set(InGameState::Dialog);
}

// ============================================================
//  UI
// ============================================================

fn setup_dialog_ui(mut commands: Commands) {
    println!("[dialog] setup_dialog_ui");
    commands
        .spawn((
            DialogUiRoot,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(50.0),
                left: Val::Px(50.0),
                right: Val::Px(50.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(24.0)),
                row_gap: Val::Px(16.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
        ))
        .with_children(|parent| {
            parent.spawn((
                DialogText,
                Text::new("..."),
                TextFont {
                    font_size: FontSize::Px(24.0),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            parent.spawn((
                DialogOptionsRoot,
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(8.0),
                    ..default()
                },
            ));

            parent.spawn((
                DialogOptionHint,
                Text::new("[1] / [2] to choose.  [Esc] to close."),
                TextFont {
                    font_size: FontSize::Px(14.0),
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
            ));
        });
}

fn cleanup_dialog_ui(mut commands: Commands, q: Query<Entity, With<DialogUiRoot>>) {
    for e in &q {
        commands.entity(e).despawn();
    }
}

// ============================================================
//  INPUT
// ============================================================

fn handle_dialog_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut active_dialog: ResMut<ActiveDialog>,
    mut next_state: ResMut<NextState<InGameState>>,
) {
    // Esc — force-close the dialog.
    if keys.just_pressed(KeyCode::Escape) {
        active_dialog.tree = None;
        active_dialog.current_node_id.clear();
        next_state.set(InGameState::Playing);
        return;
    }

    let Some(tree) = &active_dialog.tree else {
        return;
    };
    let Some(node) = tree.nodes.get(&active_dialog.current_node_id) else {
        return;
    };

    // Only 1 and 2 for now. Multi-choice is a one-line extension.
    let mut selected: Option<usize> = None;
    if keys.just_pressed(KeyCode::Digit1) {
        selected = Some(0);
    }
    if keys.just_pressed(KeyCode::Digit2) {
        selected = Some(1);
    }

    let Some(idx) = selected else { return };
    let Some(opt) = node.options.get(idx) else { return };

    // Empty next_node or a missing node = end of dialog.
    if opt.next_node.is_empty() || !tree.nodes.contains_key(&opt.next_node) {
        println!("[dialog] end of dialog");
        active_dialog.tree = None;
        active_dialog.current_node_id.clear();
        next_state.set(InGameState::Playing);
    } else {
        active_dialog.current_node_id = opt.next_node.clone();
    }
}

// ============================================================
//  UI UPDATE
// ============================================================

fn update_dialog_ui(
    active_dialog: Res<ActiveDialog>,
    mut text_q: Query<&mut Text, (With<DialogText>, Without<DialogOptionHint>)>,
    options_root_q: Query<Entity, With<DialogOptionsRoot>>,
    mut commands: Commands,
) {
    if !active_dialog.is_changed() {
        return;
    }
    let Some(tree) = &active_dialog.tree else {
        return;
    };
    let Some(node) = tree.nodes.get(&active_dialog.current_node_id) else {
        return;
    };

    if let Ok(mut text) = text_q.single_mut() {
        text.0 = node.text.clone();
    }

    if let Ok(root) = options_root_q.single() {
        commands.entity(root).despawn_related::<Children>();
        for (i, opt) in node.options.iter().enumerate() {
            let btn_text = format!("[{}] {}", i + 1, opt.text);
            commands.entity(root).with_children(|parent| {
                parent.spawn((
                    Text::new(btn_text),
                    TextFont {
                        font_size: FontSize::Px(20.0),
                        ..default()
                    },
                    TextColor(Color::srgb(0.8, 0.85, 1.0)),
                ));
            });
        }
    }
}
