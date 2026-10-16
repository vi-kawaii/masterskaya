use bevy::prelude::*;

use crate::states::AppState;
use crate::types::{MeshObjectId, Selected};

pub struct ModeSwitchPlugin;

impl Plugin for ModeSwitchPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (enter_playing, tab_cycle_or_exit_playing)
                .run_if(not(in_state(AppState::Loading))),
        );
    }
}

/// `Enter` в Editing → Playing.
///
/// `Tab` больше не переключает в Play — он занят под цикл по объектам
/// (см. `tab_cycle_or_exit_playing`). Обратно из Play — тоже `Tab`,
/// потому что в Play циклировать нечего.
fn enter_playing(
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<State<AppState>>,
    mut next: ResMut<NextState<AppState>>,
) {
    if !matches!(state.get(), AppState::Editing) {
        return;
    }
    if !keys.just_pressed(KeyCode::Enter) {
        return;
    }
    info!("Editing -> Playing");
    next.set(AppState::Playing);
}

/// `Tab` в Playing → обратно в Editing.
/// `Tab` в Editing → цикл по объектам: снять Selected со всех и поставить
/// на «следующий по doc-id».
///
/// Если в сцене ничего нет — тихо выходим, ничего не делаем.
fn tab_cycle_or_exit_playing(
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<State<AppState>>,
    mut next: ResMut<NextState<AppState>>,
    mut commands: Commands,
    objects: Query<(Entity, &MeshObjectId)>,
    selected_q: Query<Entity, With<Selected>>,
) {
    if !keys.just_pressed(KeyCode::Tab) {
        return;
    }

    match state.get() {
        AppState::Playing => {
            info!("Playing -> Editing");
            next.set(AppState::Editing);
        }
        AppState::Editing => {
            // Собираем список объектов и сортируем по doc-id.
            // Entity использовать как ключ сортировки нельзя — он может
            // переиспользоваться после despawn, а `id` в документе — стабильный.
            let mut sorted: Vec<(Entity, u64)> =
                objects.iter().map(|(e, id)| (e, id.0)).collect();
            if sorted.is_empty() {
                return;
            }
            sorted.sort_by_key(|(_, doc_id)| *doc_id);

            let current = selected_q.iter().next();

            // Снимаем выделение со всех.
            for e in &selected_q {
                commands.entity(e).remove::<Selected>();
            }

            // Выбираем следующий.
            let next_entity = match current {
                None => sorted[0].0,
                Some(cur) => {
                    let pos = sorted.iter().position(|(e, _)| *e == cur);
                    match pos {
                        Some(i) => sorted[(i + 1) % sorted.len()].0,
                        None => sorted[0].0,
                    }
                }
            };

            info!("Tab-cycle: selected {:?}", next_entity);
            commands.entity(next_entity).insert(Selected);
        }
        AppState::Loading => {}
    }
}