use bevy::prelude::*;

use crate::actor::weapon::{PlayerLoadout, WeaponState};
use crate::assets::GameAssets;
use crate::states::{GameState, InGameState};

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerLoadout>()
            .add_systems(OnEnter(GameState::InGame), setup_hud)
            .add_systems(OnExit(GameState::InGame), cleanup_hud)
            .add_systems(
                Update,
                (switch_weapon, update_hud_text)
                    .run_if(in_state(InGameState::Playing)),
            );
    }
}

#[derive(Component)]
struct HudRoot;

#[derive(Component)]
struct HudWeaponName;

#[derive(Component)]
struct HudAmmo;

#[derive(Component)]
struct HudHp;

#[derive(Component)]
struct HudCrosshair;

fn setup_hud(
    mut commands: Commands,
    mut loadout: ResMut<PlayerLoadout>,
    assets: Res<GameAssets>,
    weapon_assets: Res<Assets<crate::actor::weapon::WeaponDef>>,
) {
    println!("[hud] setup_hud");

    if loadout.weapons.is_empty() {
        let handles = [
            assets.weapon_pistol.clone(),
            assets.weapon_rifle.clone(),
            assets.weapon_shotgun.clone(),
        ];
        for handle in handles {
            if let Some(def) = weapon_assets.get(&handle) {
                loadout.weapons.push(WeaponState::new(def.clone()));
            }
        }
        println!(
            "[hud] loadout: {} weapon(s), active={}",
            loadout.weapons.len(),
            loadout.active
        );
    }

    commands
        .spawn((
            HudRoot,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                HudCrosshair,
                Text::new("+"),
                TextFont {
                    font_size: FontSize::Px(24.0),
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 1.0)),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(50.0),
                    top: Val::Percent(50.0),
                    ..default()
                },
            ));

            parent.spawn((
                HudHp,
                Text::new("HP: 100 / 100"),
                TextFont {
                    font_size: FontSize::Px(20.0),
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.4, 0.4)),
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(16.0),
                    left: Val::Px(16.0),
                    ..default()
                },
            ));

            parent.spawn((
                HudWeaponName,
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(22.0),
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(48.0),
                    right: Val::Px(24.0),
                    ..default()
                },
            ));

            parent.spawn((
                HudAmmo,
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(32.0),
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.9, 0.5)),
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(16.0),
                    right: Val::Px(24.0),
                    ..default()
                },
            ));
        });
}

fn cleanup_hud(mut commands: Commands, q: Query<Entity, With<HudRoot>>) {
    for e in &q {
        commands.entity(e).despawn();
    }
}

fn switch_weapon(
    keys: Res<ButtonInput<KeyCode>>,
    mut loadout: ResMut<PlayerLoadout>,
) {
    let mut target: Option<usize> = None;
    if keys.just_pressed(KeyCode::Digit1) {
        target = Some(0);
    } else if keys.just_pressed(KeyCode::Digit2) {
        target = Some(1);
    } else if keys.just_pressed(KeyCode::Digit3) {
        target = Some(2);
    }

    let Some(idx) = target else { return };
    if idx < loadout.weapons.len() && loadout.active != idx {
        loadout.active = idx;
        println!("[hud] switched to weapon #{idx}");
    }
}

fn update_hud_text(
    loadout: Res<PlayerLoadout>,
    mut name_q: Query<&mut Text, (With<HudWeaponName>, Without<HudAmmo>, Without<HudHp>)>,
    mut ammo_q: Query<&mut Text, (With<HudAmmo>, Without<HudWeaponName>, Without<HudHp>)>,
    mut hp_q: Query<&mut Text, (With<HudHp>, Without<HudWeaponName>, Without<HudAmmo>)>,
) {
    if let Some(weapon) = loadout.active_weapon() {
        if let Ok(mut text) = name_q.single_mut() {
            let s = weapon.def.name.clone();
            if text.0 != s {
                text.0 = s;
            }
        }
        if let Ok(mut text) = ammo_q.single_mut() {
            let s = format!("{} / {}", weapon.loaded, weapon.reserve);
            if text.0 != s {
                text.0 = s;
            }
        }
    }

    if let Ok(mut text) = hp_q.single_mut() {
        let s = "HP: 100 / 100".to_string();
        if text.0 != s {
            text.0 = s;
        }
    }
}
