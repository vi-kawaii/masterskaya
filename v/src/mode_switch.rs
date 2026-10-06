use bevy::prelude::*;

use crate::states::AppState;

pub struct ModeSwitchPlugin;

impl Plugin for ModeSwitchPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            toggle_mode.run_if(not(in_state(AppState::Loading))),
        );
    }
}

fn toggle_mode(
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<State<AppState>>,
    mut next: ResMut<NextState<AppState>>,
) {
    if !keys.just_pressed(KeyCode::Tab) {
        return;
    }
    match state.get() {
        AppState::Editing => {
            info!("Editing -> Playing");
            next.set(AppState::Playing);
        }
        AppState::Playing => {
            info!("Playing -> Editing");
            next.set(AppState::Editing);
        }
        AppState::Loading => {}
    }
}
