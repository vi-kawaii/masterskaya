use bevy::prelude::*;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
pub enum WeaponKind {
    Pistol,
    Rifle,
    Shotgun,
}

#[derive(Debug, Clone, Deserialize, Asset, TypePath)]
pub struct WeaponDef {
    pub id: String,
    pub name: String,
    pub kind: WeaponKind,
    pub damage: f32,
    pub fire_rate: f32,
    pub mag_size: u32,
    pub reserve: u32,
    pub range: f32,
}

#[derive(Debug, Clone)]
pub struct WeaponState {
    pub def: WeaponDef,
    pub loaded: u32,
    pub reserve: u32,
}

impl WeaponState {
    pub fn new(def: WeaponDef) -> Self {
        Self {
            loaded: def.mag_size,
            reserve: def.reserve,
            def,
        }
    }
}

#[derive(Resource, Default)]
pub struct PlayerLoadout {
    pub weapons: Vec<WeaponState>,
    pub active: usize,
}

impl PlayerLoadout {
    pub fn active_weapon(&self) -> Option<&WeaponState> {
        self.weapons.get(self.active)
    }
}
