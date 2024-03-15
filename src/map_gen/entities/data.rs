use std::fs;

use bevy::{ecs::system::ResMut, log::warn};
use macros::error_return;
use serde::{Deserialize, Serialize};

use crate::{PickupMap, WeaponMap};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "pickup_type")]
pub enum PickupData {
    Weapon {
        classname: String,
        gives: String,
        pickup_model: String,
        pickup_material: String,
        texture_file: String,
        scale: f32,
    },
}
impl PickupData {
    pub fn classname(&self) -> &str {
        match self {
            PickupData::Weapon { classname, .. } => classname,
        }
    }
}

pub fn load_pickups(mut map: ResMut<PickupMap>) {
    warn!("Loading pickups...");
    let data = error_return!(fs::read_to_string("assets/pickups.json"));
    let parsed = error_return!(serde_json::from_str::<Vec<PickupData>>(&data));

    for item in parsed {
        map.0.insert(item.classname().to_string(), item);
    }

    warn!("Done loading pickups...");
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(untagged)]
pub enum SoundEffect {
    #[default]
    Silent,
    Single(String),
    Random(Vec<String>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WeaponData {
    #[serde(default)]
    pub shoot_sfx: SoundEffect,
    pub id: String,
    #[serde(default)]
    pub slot: usize,
    #[serde(default)]
    pub texture_file: String,
    #[serde(default)]
    pub model_file: String,
    pub scale: f32,
    #[serde(default)]
    pub animations: WeaponAnimations,
    #[serde(default)]
    pub offset: [f32; 3],
    #[serde(default)]
    pub rotation: [f32; 3],
    pub pickup_sound: Option<String>,
    #[serde(default)]
    pub attack1: Attack,
    #[serde(default)]
    pub attack2: Attack,
}
impl WeaponData {
    fn default_firetime() -> f32 {
        1.0
    }
}

pub fn load_weapons(mut map: ResMut<WeaponMap>) {
    warn!("Loading pickups...");
    let data = error_return!(fs::read_to_string("assets/weapons.json"));
    let parsed = error_return!(serde_json::from_str::<Vec<WeaponData>>(&data));

    for item in parsed {
        map.0.insert(item.id.clone(), item);
    }

    warn!("Done loading weapons...");
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(tag = "type")]
pub enum Attack {
    #[default]
    None,
    RayCast {
        amount: usize,
        angle_mod: usize,
        damage: f32,
        damage_mod: f32,
        range: f32,
    },
    Projectile {
        projectile: String,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct WeaponAnimations {
    pub idle: String,
    pub shoot1: String,
    pub shoot2: String,
    pub reload: Option<String>,

    #[serde(default = "WeaponData::default_firetime")]
    pub fire_time1: f32,
    #[serde(default = "WeaponData::default_firetime")]
    pub anim_time1: f32,
    #[serde(default = "WeaponData::default_firetime")]
    pub fire_time2: f32,
    #[serde(default = "WeaponData::default_firetime")]
    pub anim_time2: f32,

    #[serde(default = "WeaponData::default_firetime")]
    pub reload_time_skip: f32,
    #[serde(default = "WeaponData::default_firetime")]
    pub reload_time: f32,
}
