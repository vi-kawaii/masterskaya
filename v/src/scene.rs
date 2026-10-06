use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};

use crate::states::AppState;
use crate::types::{
    MeshObject, MeshObjectId, Runtime, Spin, TransformData, VtuberDocument, VertexData,
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
                (rebuild_scene, update_status_text).run_if(
                    in_state(AppState::Editing).or_else(in_state(AppState::Playing)),
                ),
            )
            .add_systems(OnExit(AppState::Playing), cleanup_scene_on_leave_play);
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
/// doc здесь не читается напрямую — данные берёт rebuild_scene.
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

/// Убираем статус-текст при уходе из Play — при следующем входе
/// setup_scene его пересоздаст.
fn cleanup_scene_on_leave_play(
    mut commands: Commands,
    status: Query<Entity, With<StatusText>>,
) {
    for e in &status {
        commands.entity(e).despawn();
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
        AppState::Editing => "Mode: Editing    [Tab] - Play",
        AppState::Playing => {
            "Mode: Playing      [Tab] - Edit    (live-edit on)"
        }
        AppState::Loading => "Status: loading"
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
pub fn build_mesh(obj: &MeshObject) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());

    let positions: Vec<[f32; 3]> = obj.vertices.iter().map(|v| v.position).collect();
    let colors: Vec<[f32; 4]> = obj.vertices.iter().map(|v| v.color).collect();

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);

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
    let material_handle = materials.add(ColorMaterial::default());

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
