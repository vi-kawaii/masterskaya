use bevy::prelude::*;

use crate::scene::SyncTransforms;
use crate::types::{LiveEditEnabled, MeshObjectId, Selected, VtuberDocument};

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LiveEditEnabled>()
            // Работает И в Editing, И в Playing (real-time editing).
            // Фильтр по Loading — только чтобы не мешать загрузке.
            // `sync_transforms` теперь живёт в ScenePlugin.
            .add_systems(
                Update,
                (
                    editor_move_system,
                    editor_rotate_system,
                    editor_scale_system,
                )
                    .run_if(not(in_state(crate::states::AppState::Loading))),
            );
    }
}

/// Перемещение выделенного объекта стрелками.
/// Пишет в документ — источник истины. Playback сам подхватит.
fn editor_move_system(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    live: Res<LiveEditEnabled>,
    mut doc: ResMut<VtuberDocument>,
    mut sync_ev: MessageWriter<SyncTransforms>,
    selected: Query<&MeshObjectId, With<Selected>>,
) {
    if !live.0 {
        return;
    }

    let speed = 100.0 * time.delta_secs();
    let mut moved = false;

    for id in &selected {
        if let Some(obj) = doc.get_mut(id.0) {
            if keys.pressed(KeyCode::ArrowLeft)  { obj.transform.translation[0] -= speed; moved = true; }
            if keys.pressed(KeyCode::ArrowRight) { obj.transform.translation[0] += speed; moved = true; }
            if keys.pressed(KeyCode::ArrowUp)    { obj.transform.translation[1] += speed; moved = true; }
            if keys.pressed(KeyCode::ArrowDown)  { obj.transform.translation[1] -= speed; moved = true; }
        }
    }
    if moved {
        sync_ev.write(SyncTransforms);
    }
}

/// Поворот выделенного объекта: `Q` — против часовой, `E` — по часовой.
/// Крутим вокруг локальной оси Z (2D-плоскость), поэтому меняем только
/// `rotation_deg[2]`.
///
/// `sync_transforms` в Editing применит поворот к ECS; в Playing поворот
/// базового трансформа ставится при входе в Play (`rebuild_scene`), а
/// runtime-слой спиновый и живёт отдельно (см. `playback_system`).
fn editor_rotate_system(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    live: Res<LiveEditEnabled>,
    mut doc: ResMut<VtuberDocument>,
    mut sync_ev: MessageWriter<SyncTransforms>,
    selected: Query<&MeshObjectId, With<Selected>>,
) {
    if !live.0 {
        return;
    }

    let speed_deg = 90.0 * time.delta_secs();
    let mut rotated = false;

    for id in &selected {
        if let Some(obj) = doc.get_mut(id.0) {
            if keys.pressed(KeyCode::KeyQ) {
                obj.transform.rotation_deg[2] -= speed_deg;
                rotated = true;
            }
            if keys.pressed(KeyCode::KeyE) {
                obj.transform.rotation_deg[2] += speed_deg;
                rotated = true;
            }
        }
    }
    if rotated {
        sync_ev.write(SyncTransforms);
    }
}

/// Масштаб выделенного объекта: `[` — уменьшить, `]` — увеличить.
/// Меняем по обеим осям (X, Y) одинаково, чтобы сохранять пропорции.
/// Клэмпим в разумных пределах, чтобы объект не исчез и не заполнил экран.
fn editor_scale_system(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    live: Res<LiveEditEnabled>,
    mut doc: ResMut<VtuberDocument>,
    mut sync_ev: MessageWriter<SyncTransforms>,
    selected: Query<&MeshObjectId, With<Selected>>,
) {
    if !live.0 {
        return;
    }

    const MIN_SCALE: f32 = 0.1;
    const MAX_SCALE: f32 = 10.0;

    let factor_per_sec = 1.0_f32; // 1.0 = ничего, растёт/падает экспоненциально
    let dt = time.delta_secs();
    let mut scaled = false;

    // [ уменьшает, ] увеличивает. Используем экспоненту:
    // new = old * exp(-k*dt) или exp(+k*dt). Так масштаб никогда не уйдёт в 0.
    let down = keys.pressed(KeyCode::BracketLeft);
    let up = keys.pressed(KeyCode::BracketRight);

    if !down && !up {
        return;
    }

    let k = 2.0 * factor_per_sec; // скорость
    let factor = if up {
        (k * dt).exp()
    } else {
        (-k * dt).exp()
    };

    for id in &selected {
        if let Some(obj) = doc.get_mut(id.0) {
            let s = &mut obj.transform.scale;
            s[0] = (s[0] * factor).clamp(MIN_SCALE, MAX_SCALE);
            s[1] = (s[1] * factor).clamp(MIN_SCALE, MAX_SCALE);
            s[2] = (s[2] * factor).clamp(MIN_SCALE, MAX_SCALE);
            scaled = true;
        }
    }
    if scaled {
        sync_ev.write(SyncTransforms);
    }
}