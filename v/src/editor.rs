use bevy::prelude::*;

use crate::scene::SyncTransforms;
use crate::types::{LiveEditEnabled, MeshObjectId, Selected, VtuberDocument};

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LiveEditEnabled>()
            // Работает И в Editing, И в Playing (real-time editing).
            // Фильтр по Loading — только чтобы не мешать загрузке.
            .add_systems(
                Update,
                (editor_move_system, sync_transforms)
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
        // Когда выключим live-edit, здесь будем писать в Runtime, а не в doc.
        // Пока оставляем заглушку.
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

/// Применяет базовый трансформ из документа к ECS.
/// В Play НЕ трогает rotation/scale — их держит Playback (base ⊕ runtime).
/// В Editing применяет всё.
fn sync_transforms(
    mut ev: MessageReader<SyncTransforms>,
    state: Res<State<crate::states::AppState>>,
    doc: Res<VtuberDocument>,
    mut q: Query<(&MeshObjectId, &mut Transform)>,
) {
    if ev.read().count() == 0 {
        return;
    }

    let in_play = matches!(state.get(), crate::states::AppState::Playing);

    for (id, mut t) in &mut q {
        if let Some(obj) = doc.get(id.0) {
            t.translation = Vec3::from(obj.transform.translation);

            // В Play rotation/scale принадлежат Playback'у.
            // Синхронизатор их не трогает, чтобы не конфликтовать.
            if !in_play {
                t.rotation = Quat::from_euler(
                    EulerRot::XYZ,
                    obj.transform.rotation_deg[0].to_radians(),
                    obj.transform.rotation_deg[1].to_radians(),
                    obj.transform.rotation_deg[2].to_radians(),
                );
                t.scale = Vec3::from(obj.transform.scale);
            }
        }
    }
}
