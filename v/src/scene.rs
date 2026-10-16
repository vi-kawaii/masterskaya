use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::picking::mesh_picking::ray_cast::{MeshRayCast, MeshRayCastSettings};

use crate::states::AppState;
use crate::types::{
    MeshObject, MeshObjectId, Runtime, Selected, Spin, TransformData, VtuberDocument, VertexData,
};

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VtuberDocument>()
            .add_message::<RebuildScene>()
            .add_message::<SyncTransforms>()
            .add_systems(Startup, setup_camera)
            .add_systems(
                OnEnter(AppState::Editing),
                (setup_scene, reset_runtime_on_exit_play),
            )
            .add_systems(OnEnter(AppState::Playing), setup_scene)
            .add_systems(
                Update,
                (
                    rebuild_scene,
                    update_status_text,
                    selection_system,
                    highlight_selected,
                    sync_transforms,
                )
                    .run_if(in_state(AppState::Editing).or_else(in_state(AppState::Playing))),
            )
            .add_systems(OnExit(AppState::Playing), (cleanup_scene_on_leave_play, clear_selection))
            .add_systems(OnExit(AppState::Editing), clear_selection);
    }
}

/// Сообщения (Bevy 0.19: Event → Message).
#[derive(Message)]
pub struct RebuildScene;

#[derive(Message)]
pub struct SyncTransforms;

/// Статус-текст в правом верхнем углу.
#[derive(Component)]
struct StatusText;

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, Msaa::Off));
}

/// Спавнит статус-текст (если его нет) и просит сцену пересобраться.
fn setup_scene(
    mut commands: Commands,
    mut rebuild: MessageWriter<RebuildScene>,
    status_q: Query<Entity, With<StatusText>>,
) {
    if status_q.is_empty() {
        commands.spawn((
            StatusText,
            Text::new(""),
            TextFont {
                font_size: FontSize::Px(20.0),
                ..default()
            },
            TextColor(Color::srgb(0.0, 1.0, 0.0)),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(12.0),
                right: Val::Px(12.0),
                ..default()
            },
        ));
    }

    rebuild.write(RebuildScene);
}

/// При выходе из Play возвращаем сцену к базовой позе из документа
/// и обнуляем runtime-слой.
fn reset_runtime_on_exit_play(
    mut q: Query<(&MeshObjectId, &mut Transform, &mut Runtime)>,
    doc: Res<VtuberDocument>,
) {
    for (id, mut t, mut rt) in &mut q {
        if let Some(obj) = doc.get(id.0) {
            apply_base_transform(&mut t, &obj.transform);
        }
        rt.0 = Default::default();
    }
}

/// Убираем статус-текст при уходе из Play.
fn cleanup_scene_on_leave_play(
    mut commands: Commands,
    status: Query<Entity, With<StatusText>>,
) {
    for e in &status {
        commands.entity(e).despawn();
    }
}

/// Снимает `Selected` со всех сущностей.
fn clear_selection(
    mut commands: Commands,
    selected_q: Query<Entity, With<Selected>>,
) {
    for e in &selected_q {
        commands.entity(e).remove::<Selected>();
    }
}

/// Применить TransformData из документа к ECS Transform.
pub fn apply_base_transform(t: &mut Transform, data: &TransformData) {
    t.translation = Vec3::from(data.translation);
    t.rotation = Quat::from_euler(
        EulerRot::XYZ,
        data.rotation_deg[0].to_radians(),
        data.rotation_deg[1].to_radians(),
        data.rotation_deg[2].to_radians(),
    );
    t.scale = Vec3::from(data.scale);
}

fn update_status_text(
    state: Res<State<AppState>>,
    mut q: Query<&mut Text, With<StatusText>>,
) {
    let msg = match state.get() {
        AppState::Editing => {
            "Mode: Editing    [Enter] Play    [Tab] Cycle    [LMB] Select    [Arrows] Move    [Q/E] Rotate    [[/]] Scale    [Ctrl+S/O] Save/Load"
        }
        AppState::Playing => {
            "Mode: Playing    [Tab] Edit    (live-edit on)"
        }
        AppState::Loading => "Status: loading",
    };
    for mut text in &mut q {
        if text.0 != msg {
            text.0 = msg.to_string();
        }
    }
}

// ============================================================
//  ФАБРИКИ МЕШЕЙ
// ============================================================

/// Создаёт стандартный треугольник как MeshObject.
pub fn make_triangle(name: &str, r: f32) -> MeshObject {
    let half = r * 3.0_f32.sqrt() / 2.0;
    MeshObject {
        id: 0,
        name: name.to_string(),
        vertices: vec![
            VertexData { position: [0.0, r, 0.0],           color: [1.0, 0.0, 0.0, 1.0] },
            VertexData { position: [-half, -r / 2.0, 0.0],  color: [0.0, 1.0, 0.0, 1.0] },
            VertexData { position: [half, -r / 2.0, 0.0],   color: [0.0, 0.0, 1.0, 1.0] },
        ],
        indices: vec![],
        transform: Default::default(),
        playback: Default::default(),
    }
}

/// MeshObject → Bevy Mesh.
///
/// Vertex colors (`Mesh::ATTRIBUTE_COLOR`) НЕ добавляем: они умножаются на
/// `ColorMaterial.color` и мешают подсветке выделения.
pub fn build_mesh(obj: &MeshObject) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());

    let positions: Vec<[f32; 3]> = obj.vertices.iter().map(|v| v.position).collect();
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);

    if !obj.indices.is_empty() {
        mesh.insert_indices(Indices::U32(obj.indices.clone()));
    }
    mesh
}

fn spawn_mesh_object(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    obj: &MeshObject,
) -> Entity {
    let mesh_handle = meshes.add(build_mesh(obj));
    // Белый — базовый цвет. Подсветка выделения меняет его на жёлтый.
    let material_handle = materials.add(ColorMaterial::from(Color::WHITE));

    let mut t = Transform::default();
    apply_base_transform(&mut t, &obj.transform);

    commands
        .spawn((
            Mesh2d(mesh_handle),
            MeshMaterial2d(material_handle),
            t,
            MeshObjectId(obj.id),
            Spin { speed: obj.playback.spin_speed },
            Runtime::default(),
        ))
        .id()
}

fn rebuild_scene(
    mut ev: MessageReader<RebuildScene>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    doc: Res<VtuberDocument>,
    existing: Query<Entity, With<MeshObjectId>>,
) {
    if ev.read().count() == 0 {
        return;
    }
    for e in &existing {
        commands.entity(e).despawn();
    }
    for obj in &doc.objects {
        spawn_mesh_object(&mut commands, &mut meshes, &mut materials, obj);
    }
}

// ============================================================
//  ВЫДЕЛЕНИЕ
// ============================================================

/// Система, которая обрабатывает клики мышью через `MeshRayCast`.
///
/// Луч выходит из камеры через позицию курсора в плоскости Z=0.
/// `MeshRayCast` возвращает список попаданий, отсортированных по дистанции,
/// мы берём первое, у которого есть компонент `MeshObjectId`.
///
/// Это точнее, чем поиск по дистанции до центра: попадание проверяется
/// по реальной геометрии меша.
fn selection_system(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut commands: Commands,
    objects: Query<&MeshObjectId>,
    selected_q: Query<Entity, With<Selected>>,
    mut ray_cast: MeshRayCast,
) {
    if !mouse_buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = windows.single() else { return };
    let Ok((camera, camera_transform)) = camera_q.single() else { return };

    // Получаем мировые координаты курсора.
    let Some(cursor_world_pos) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor).ok())
    else {
        return;
    };

    // В 2D объекты лежат в плоскости Z=0. Луч из точки в направлении -Z.
    // В Bevy 0.19 `cast_ray` принимает `Ray3d` и `&MeshRayCastSettings`.
    let ray = Ray3d::new(cursor_world_pos.extend(10.0), Dir3::NEG_Z);

    let settings = MeshRayCastSettings::default();
    let hits = ray_cast.cast_ray(ray, &settings);

    // Ищем первую сущность с MeshObjectId среди попаданий.
    let clicked = hits
        .iter()
        .map(|(entity, _hit)| *entity)
        .find(|e| objects.contains(*e));

    // Снимаем выделение со всех.
    for entity in &selected_q {
        commands.entity(entity).remove::<Selected>();
    }

    // Выделяем новый (если нашли).
    if let Some(entity) = clicked {
        info!("Selected entity: {:?}", entity);
        commands.entity(entity).insert(Selected);
    } else {
        info!("Deselected");
    }
}

/// Подсвечивает выделенный объект.
fn highlight_selected(
    mut materials: ResMut<Assets<ColorMaterial>>,
    mesh_q: Query<&MeshMaterial2d<ColorMaterial>>,
    added_q: Query<Entity, Added<Selected>>,
    mut removed: RemovedComponents<Selected>,
    mut prev_selected: Local<Option<Entity>>,
) {
    // 1. Кто потерял Selected — вернуть белый.
    for entity in removed.read() {
        if let Ok(mat_handle) = mesh_q.get(entity) {
            if let Some(mut material) = materials.get_mut(&mat_handle.0) {
                material.color = Color::WHITE;
            }
        }
    }

    // 2. Кто получил Selected — покрасить нового жёлтым,
    //    а у предыдущего вернуть белый.
    for entity in &added_q {
        if let Some(prev) = *prev_selected {
            if prev != entity {
                if let Ok(mat_handle) = mesh_q.get(prev) {
                    if let Some(mut material) = materials.get_mut(&mat_handle.0) {
                        material.color = Color::WHITE;
                    }
                }
            }
        }

        if let Ok(mat_handle) = mesh_q.get(entity) {
            if let Some(mut material) = materials.get_mut(&mat_handle.0) {
                material.color = Color::srgb(1.0, 1.0, 0.0);
            }
        }

        *prev_selected = Some(entity);
    }
}

// ============================================================
//  СИНХРОНИЗАЦИЯ ТРАНСФОРМОВ
// ============================================================

/// Применяет базовый трансформ из документа к ECS.
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