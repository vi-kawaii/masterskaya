pub mod data;
pub mod events;
pub mod log;
pub mod tracker;

use bevy::prelude::*;

use crate::assets::GameAssets;
use crate::states::InGameState;

// Facade re-exports: only the types actually used outside the `quest` module
// are visible from `crate::quest::...`:
//   - QuestDef, QuestLog  → assets.rs, modes/loading.rs, ui/hud.rs
//   - QuestEvent          → modes/game/dialog/mod.rs
// The rest (QuestStep, QuestReward, QuestTarget, ActiveQuest) stays internal
// and can be reached via `quest::data::*` / `quest::log::*` if needed.
#[allow(unused_imports)]
pub use data::QuestDef;
#[allow(unused_imports)]
pub use events::QuestEvent;
#[allow(unused_imports)]
pub use log::QuestLog;

pub struct QuestPlugin;

impl Plugin for QuestPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<QuestLog>()
            .add_message::<QuestEvent>()
            // `start_debug_quest` — only in Playing (Q key should not
            // fire while the player is inside a dialog).
            .add_systems(
                Update,
                start_debug_quest.run_if(in_state(InGameState::Playing)),
            )
            // `quest_tracker` runs in every in-game substate (Playing,
            // Paused, Dialog) so that QuestEvents emitted by the dialog
            // (in `InGameState::Dialog`) are actually consumed.
            // Otherwise events expire in the message buffer before the
            // tracker gets a chance to read them.
            .add_systems(Update, tracker::quest_tracker);
    }
}

/// Debug trigger: pressing `Q` starts the `first_steps` quest.
/// Later this will be replaced with quest activation from a dialog.
fn start_debug_quest(
    keys: Res<ButtonInput<KeyCode>>,
    assets: Res<GameAssets>,
    quest_defs: Res<Assets<QuestDef>>,
    mut log: ResMut<QuestLog>,
) {
    if !keys.just_pressed(KeyCode::KeyQ) {
        return;
    }

    let Some(def) = quest_defs.get(&assets.quest_first_steps) else {
        println!("[quest] first_steps still not loaded");
        return;
    };

    if log.start(def.clone()) {
        println!("[quest] debug: started first_steps");
    } else {
        println!("[quest] debug: first_steps already active or completed");
    }
}