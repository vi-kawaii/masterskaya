use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// ============================================================
//  СЕРИАЛИЗУЕМЫЕ ДАННЫЕ (то, что попадёт в .ron)
// ============================================================

/// Одна вершина меша.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VertexData {
    pub position: [f32; 3],
    pub color: [f32; 4],
}

/// Данные проигрывания, привязанные к объекту.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlaybackData {
    pub spin_speed: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformData {
    pub translation: [f32; 3],
    pub rotation_deg: [f32; 3],
    pub scale: [f32; 3],
}

impl Default for TransformData {
    fn default() -> Self {
        Self {
            translation: [0.0; 3],
            rotation_deg: [0.0; 3],
            scale: [1.0; 3],
        }
    }
}

/// Один объект документа.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshObject {
    pub id: u64,
    pub name: String,
    pub vertices: Vec<VertexData>,
    /// Пусто = TriangleList по порядку вершин.
    pub indices: Vec<u32>,
    pub transform: TransformData,
    #[serde(default)]
    pub playback: PlaybackData,
}

/// Документ проекта — единственный источник истины для "базовой позы".
#[derive(Resource, Debug, Clone, Serialize, Deserialize, Default)]
pub struct VtuberDocument {
    pub name: String,
    pub objects: Vec<MeshObject>,
    pub next_id: u64,
}

impl VtuberDocument {
    pub fn add_object(&mut self, mut obj: MeshObject) -> u64 {
        obj.id = self.next_id;
        self.next_id += 1;
        let id = obj.id;
        self.objects.push(obj);
        id
    }

    pub fn get(&self, id: u64) -> Option<&MeshObject> {
        self.objects.iter().find(|o| o.id == id)
    }

    pub fn get_mut(&mut self, id: u64) -> Option<&mut MeshObject> {
        self.objects.iter_mut().find(|o| o.id == id)
    }
}

// ============================================================
//  КОМПОНЕНТЫ ECS
// ============================================================

/// Связь ECS-сущности с объектом документа.
#[derive(Component, Debug, Clone, Copy)]
pub struct MeshObjectId(pub u64);

/// Маркер выделенного объекта (в редакторе).
#[derive(Component)]
pub struct Selected;

/// Вращение в Play-режиме.
#[derive(Component)]
pub struct Spin {
    pub speed: f32,
}

// ============================================================
//  RUNTIME-СЛОЙ (не сериализуется, живёт только во время Play)
// ============================================================

/// Runtime-трансформ — overlay поверх базового трансформа из документа.
///
/// Итоговый трансформ сцены = base(from doc) ⊕ runtime(this).
/// Смысл: Play НЕ портит документ, только накладывает временный слой.
#[derive(Debug, Clone, Copy)]
pub struct RuntimeTransform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
    /// Фаза спина в радианах (накопительная). Хранить фазу, а не кватернион —
    /// чтобы избежать дрейфа float при длительном Play.
    pub spin_phase: f32,
}

impl Default for RuntimeTransform {
    fn default() -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
            spin_phase: 0.0,
        }
    }
}

/// Компонент: runtime-состояние объекта. Пишется PlaybackPlugin,
/// сбрасывается при выходе из Play.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Runtime(pub RuntimeTransform);

// ============================================================
//  ФЛАГИ ПРИЛОЖЕНИЯ
// ============================================================

/// Включён ли режим real-time редактирования (изменения идут в документ
/// сразу, даже во время Play). По умолчанию включён.
#[derive(Resource)]
pub struct LiveEditEnabled(pub bool);

impl Default for LiveEditEnabled {
    fn default() -> Self {
        Self(true)
    }
}
