use crate::assets::GameAssets;
use crate::states::GameState;
use bevy::asset::LoadState;
use bevy::prelude::*;

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_loading_camera)
            .add_systems(OnEnter(GameState::Loading), (setup_loading, start_loading))
            .add_systems(
                Update,
                (check_loading, update_loading_ui)
                    .chain()
                    .run_if(in_state(GameState::Loading)),
            );
        // cleanup_loading и cleanup_loading_camera больше НЕ на OnExit(Loading).
        // Их вызывает on_island_ready в modes/game/mod.rs, когда остров
        // реально готов. Это убирает визуальную паузу между удалением UI
        // и появлением игры.
    }
}

#[derive(Component)]
pub struct LoadingCamera;

#[derive(Component)]
pub struct LoadingUi;

#[derive(Component)]
struct LoadingProgressText;

#[derive(Component)]
struct LoadingBarFill;

/// Хендлы, за которыми следим. Счётчик done/total.
#[derive(Resource, Default)]
pub struct LoadingHandles {
    pub handles: Vec<UntypedHandle>,
    pub done: usize,
    pub failed: bool,
    pub min_display: f32,
    pub elapsed: f32,
}

/// Camera2d для рендера UI загрузки.
fn spawn_loading_camera(mut commands: Commands) {
    println!("[loading] spawn_loading_camera");
    commands.spawn((Camera2d, Msaa::Off, LoadingCamera));
}

fn setup_loading(mut commands: Commands) {
    println!("[loading] setup_loading");
    commands
        .spawn((
            LoadingUi,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(20.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Loading..."),
                TextFont {
                    font_size: FontSize::Px(48.0),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            parent
                .spawn(Node {
                    width: Val::Px(400.0),
                    height: Val::Px(24.0),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                })
                .insert(BorderColor::all(Color::WHITE))
                .with_children(|bar| {
                    bar.spawn((
                        LoadingBarFill,
                        Node {
                            width: Val::Percent(0.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.0, 1.0, 0.0)),
                    ));
                });

            parent.spawn((
                LoadingProgressText,
                Text::new("0%"),
                TextFont {
                    font_size: FontSize::Px(20.0),
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
        });
}

fn start_loading(mut commands: Commands, asset_server: Res<AssetServer>) {
    println!("[loading] start_loading");
    let player_scene: Handle<WorldAsset> =
        asset_server.load("models/anime-girl.glb#Scene0");
    let island_scene: Handle<WorldAsset> =
        asset_server.load("models/main.glb#Scene0");

    let car_scene: Handle<WorldAsset> =
        asset_server.load("models/fast-car.glb#Scene0");
    let wheel1_scene: Handle<WorldAsset> =
        asset_server.load("models/wheel1.glb#Scene0");
    let wheel2_scene: Handle<WorldAsset> =
        asset_server.load("models/wheel2.glb#Scene0");
    let wheel3_scene: Handle<WorldAsset> =
        asset_server.load("models/wheel3.glb#Scene0");
    let wheel4_scene: Handle<WorldAsset> =
        asset_server.load("models/wheel4.glb#Scene0");

    commands.insert_resource(GameAssets {
        player_scene: player_scene.clone(),
        island_scene: island_scene.clone(),
        car_scene: car_scene.clone(),
        wheel1_scene: wheel1_scene.clone(),
        wheel2_scene: wheel2_scene.clone(),
        wheel3_scene: wheel3_scene.clone(),
        wheel4_scene: wheel4_scene.clone(),
    });

    let mut loading = LoadingHandles::default();
    loading.handles.push(player_scene.untyped());
    loading.handles.push(island_scene.untyped());
    loading.handles.push(car_scene.untyped());
    loading.handles.push(wheel1_scene.untyped());
    loading.handles.push(wheel2_scene.untyped());
    loading.handles.push(wheel3_scene.untyped());
    loading.handles.push(wheel4_scene.untyped());
    loading.min_display = 0.5;
    commands.insert_resource(loading);
}

fn check_loading(
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut loading: ResMut<LoadingHandles>,
    assets: Option<Res<GameAssets>>,
    mut commands: Commands,
    ground_exists: Query<Entity, With<crate::modes::game::Ground>>,
) {
    if loading.failed {
        return;
    }

    loading.elapsed += time.delta_secs();

    let mut done = 0;
    for handle in &loading.handles {
        match asset_server.get_load_state(handle) {
            Some(LoadState::Loaded) => done += 1,
            Some(LoadState::Failed(_)) => {
                error!("Ассет не загрузился: {:?}", handle);
                loading.failed = true;
                return;
            }
            _ => {}
        }
    }
    loading.done = done;

    let all_loaded = done == loading.handles.len();
    let min_time_passed = loading.elapsed >= loading.min_display;

    if all_loaded && min_time_passed && ground_exists.is_empty() {
        let Some(assets) = assets else { return };
        println!("[loading] ассеты загружены — спавним остров");
        crate::modes::game::spawn_island(&mut commands, &assets);
    }
}

/// Прогресс-бар. Ресурс `LoadingHandles` берётся как `Option`,
/// чтобы не паниковать, если он уже удалён (например, когда
/// on_island_ready почистил его, а Loading ещё не полностью ушёл).
fn update_loading_ui(
    loading: Option<Res<LoadingHandles>>,
    mut text_q: Query<&mut Text, With<LoadingProgressText>>,
    mut bar_q: Query<&mut Node, With<LoadingBarFill>>,
) {
    let Some(loading) = loading else {
        return;
    };

    let total = loading.handles.len().max(1);
    let asset_progress = loading.done as f32 / total as f32;

    let time_progress = if loading.min_display > 0.0 {
        (loading.elapsed / loading.min_display).min(1.0)
    } else {
        1.0
    };
    let t = asset_progress.max(time_progress);
    let percent = (t * 100.0).round() as u32;

    for mut text in &mut text_q {
        let s = format!("{}%   ({} / {})", percent, loading.done, total);
        if text.0 != s {
            text.0 = s;
        }
    }
    for mut node in &mut bar_q {
        node.width = Val::Percent(t * 100.0);
    }
}
