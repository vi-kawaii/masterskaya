use bevy::prelude::*;

use crate::states::AppState;
use crate::types::{MeshObjectId, Runtime, Spin, VtuberDocument};

pub struct PlaybackPlugin;

impl Plugin for PlaybackPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            playback_system.run_if(in_state(AppState::Playing)),
        );
    }
}

/// В Play: базовый трансформ читается из документа каждый кадр,
/// overlay-слой копится в Runtime, итог = base ⊕ overlay.
/// Документ НЕ меняется — Play не портит базовую позу.
fn playback_system(
    time: Res<Time>,
    doc: Res<VtuberDocument>,
    mut q: Query<(&MeshObjectId, &mut Transform, &Spin, &mut Runtime)>,
) {
    for (id, mut t, spin, mut rt) in &mut q {
        // 1. Обновляем overlay (инкрементально, но через фазу — без дрейфа).
        rt.0.spin_phase += time.delta_secs() * spin.speed;
        // Приводим фазу в [0, TAU), чтобы не терять точность f32 на длинных сессиях.
        rt.0.spin_phase = rt.0.spin_phase.rem_euclid(std::f32::consts::TAU);
        rt.0.rotation = Quat::from_rotation_z(rt.0.spin_phase);

        // 2. Читаем свежую базу из документа.
        let Some(obj) = doc.get(id.0) else { continue };

        let base_t = Vec3::from(obj.transform.translation);
        let base_r = Quat::from_euler(
            EulerRot::XYZ,
            obj.transform.rotation_deg[0].to_radians(),
            obj.transform.rotation_deg[1].to_radians(),
            obj.transform.rotation_deg[2].to_radians(),
        );
        let base_s = Vec3::from(obj.transform.scale);

        // 3. Композиция: base ⊕ overlay.
        //    rotation — локально (overlay крутится вокруг локальной Z объекта).
        //    translation, scale — аддитивно/мультипликативно.
        t.translation = base_t + rt.0.translation;
        t.rotation = base_r * rt.0.rotation;
        t.scale = base_s * rt.0.scale;
    }
}
