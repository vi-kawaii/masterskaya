use bevy::prelude::*;

use super::data::QuestTarget;
use super::events::QuestEvent;
use super::log::{ActiveQuest, QuestLog};

/// Реагирует на `QuestEvent`, двигает шаги активных квестов.
///
/// Логика простая: для каждого активного квеста смотрим его текущий шаг.
/// Если target этого шага матчится с пришедшим событием — двигаем квест.
/// Для KillCount/CollectItem сначала инкрементируем счётчик, и только
/// когда набрали нужное количество — двигаем шаг.
pub fn quest_tracker(
    mut events: MessageReader<QuestEvent>,
    mut log: ResMut<QuestLog>,
) {
    for ev in events.read() {
        // Собираем id квестов, которые в этом событии продвинулись.
        let mut to_complete: Vec<String> = Vec::new();

        for (quest_id, active) in log.active.iter_mut() {
            // Клонируем всё, что нужно от шага, ДО мутабельного заёма active.
            // `QuestTarget` — маленький enum, клонируется дешёво.
            // Так мы снимаем конфликт: `active.current()` больше не держит
            // иммутабельный заём, пока `target_matches` берёт `&mut active`.
            let (step_id, step_desc, target) = match active.current() {
                Some(step) => (
                    step.id.clone(),
                    step.description.clone(),
                    step.target.clone(),
                ),
                None => continue,
            };

            if !target_matches(&target, ev, active) {
                continue;
            }

            println!(
                "[quest] step done: {} :: {} ({})",
                active.def.name, step_desc, step_id
            );

            active.current_step += 1;

            if active.is_finished() {
                to_complete.push(quest_id.clone());
            }
        }

        // Завершённые квесты переносим из active в completed.
        for id in to_complete {
            if let Some(q) = log.active.remove(&id) {
                println!(
                    "[quest] completed: {} (xp={}, items={:?})",
                    q.def.name, q.def.rewards.xp, q.def.rewards.items
                );
                log.completed.insert(id);
            }
        }
    }
}

/// Матчится ли target шага с пришедшим событием.
/// Для счётчиковых целей — инкрементирует счётчик в `active` и возвращает
/// true только когда набрано нужное количество.
fn target_matches(
    target: &QuestTarget,
    ev: &QuestEvent,
    active: &mut ActiveQuest,
) -> bool {
    match (target, ev) {
        (QuestTarget::TalkTo(want), QuestEvent::TalkedTo(got)) => want == got,
        (QuestTarget::ReachLocation(want), QuestEvent::ReachedLocation(got)) => want == got,

        (QuestTarget::KillCount(kind, need), QuestEvent::Killed(got_kind)) => {
            if kind != got_kind {
                return false;
            }
            let counter = active.counters.entry(kind.clone()).or_insert(0);
            *counter += 1;
            *counter >= *need
        }

        (QuestTarget::CollectItem(item, need), QuestEvent::Collected(got_item)) => {
            if item != got_item {
                return false;
            }
            let counter = active.counters.entry(item.clone()).or_insert(0);
            *counter += 1;
            *counter >= *need
        }

        _ => false,
    }
}