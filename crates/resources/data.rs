use bevy::{ecs::system::Resource, utils::HashMap};
use faststr::FastStr;
use serde::{Deserialize, Serialize};
use std::fs::read_to_string;

/// The data for a projectile
#[derive(Debug, Deserialize, Clone)]
pub struct Projectile {
    /// The projectile id
    pub id: FastStr,
    /// The projectile model. Should be OBJ.
    pub model_file: FastStr,
    /// The texture location. Should be PNG.
    pub texture_file: FastStr,
    /// The scale of the model.
    pub scale: f32,
    /// The rotation of the model.
    pub rotation: [f32; 3],
    /// The flying speed of the projectile.
    pub speed: f32,
}

/// A [HashMap] of all projectile data.
#[derive(Debug, Resource)]
pub struct Projectiles(pub HashMap<FastStr, Projectile>);
impl Default for Projectiles {
    fn default() -> Self {
        let input = read_to_string("assets/projectiles.json").unwrap();
        let json = serde_json::from_str::<Vec<Projectile>>(&input).unwrap();
        Projectiles(json.into_iter().map(|p| (p.id.clone(), p)).collect())
    }
}

/// The type of a pickup.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PickupType {
    /// This will result in a weapon.
    Weapon,
}

/// The data for a pickup item.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PickupData {
    /// The type of the pickup.
    pub pickup_type: PickupType,
    /// The map classname.
    pub classname: FastStr,
    /// The item you will get by picking this up.
    pub gives: FastStr,
    /// The model for this pickup, should be OBJ.
    pub pickup_model: FastStr,
    /// The material for this pickup, should be MTL.
    pub pickup_material: FastStr,
    /// The texture for this pickup, should be PNG.
    pub texture_file: FastStr,
    /// The scale of this texture.
    pub scale: f32,
}

/// The sound effect to be played when using a weapon/or by an enemy.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(untagged)]
pub enum SoundEffect {
    /// No sound at all.
    #[default]
    Silent,
    /// Always play this.
    Single(FastStr),
    /// Play one of these random ones.
    Random(Vec<FastStr>),
}

/// The data of a weapon.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WeaponData {
    /// The sound effect to play when shooting.
    #[serde(default)]
    pub shoot_sfx: SoundEffect,
    /// The id of a weapon.
    pub id: FastStr,
    /// The inventory slot this weapon should land in.
    #[serde(default)]
    pub slot: usize,
    /// The texture of this weapon. Should be PNG.
    #[serde(default)]
    pub texture_file: FastStr,
    /// The model of this weapon. Should be GLTF.
    #[serde(default)]
    pub model_file: FastStr,
    /// The scale of the model.
    pub scale: f32,
    /// The animations of this weapon.
    #[serde(default)]
    pub animations: WeaponAnimations,
    /// How to offset the model in first person view.
    #[serde(default)]
    pub offset: [f32; 3],
    /// How to rotate the model in first person view.
    #[serde(default)]
    pub rotation: [f32; 3],
    /// The sound to be played when picking up the weapon.
    pub pickup_sound: Option<FastStr>,
    /// The attack data for the first attack (left click by default).
    #[serde(default)]
    pub attack1: Attack,
    #[serde(default)]
    /// The attack data for the second attack (right click by default).
    pub attack2: Attack,
    /// The start of a message when picking up a weapon.
    #[serde(default = "default_pickupmessage1")]
    pub pickup_message1: FastStr,
    /// The end of a message when picking up a weapon.
    #[serde(default = "default_pickupmessage2")]
    pub pickup_message2: FastStr,
    /// The name to be displayed when picking up a weapon.
    #[serde(default = "default_fancyname")]
    pub fancy_name: FastStr,
}
impl WeaponData {
    fn default_firetime() -> f32 {
        1.0
    }
}

fn default_fancyname() -> FastStr {
    FastStr::from("UNNAMNED_WEAPON")
}

fn default_pickupmessage1() -> FastStr {
    FastStr::from("PICKED UP: ")
}

fn default_pickupmessage2() -> FastStr {
    FastStr::from("!")
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(tag = "type")]
/// The attack data of a weapon.
pub enum Attack {
    /// This weapon can't attack, should most likely not be used.
    #[default]
    None,
    /// Attack using a RayCast.
    RayCast {
        /// The amount of rays to be used.
        amount: usize,
        /// The maximum angle of the spray of a weapon.
        angle_mod: f32,
        /// The base damage of a weapon.
        damage: f32,
        /// The modifier for weapon damage, used to randomize the damage.
        damage_mod: f32,
        /// The range of this weapon.
        range: f32,
    },
    /// Attack using a projectile.
    Projectile {
        /// The projectile to be used.
        projectile: FastStr,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
/// The weapon animation data.
pub struct WeaponAnimations {
    /// The index of the idle animation.
    pub idle: usize,
    /// The index of the first attack animation.
    pub shoot1: usize,
    /// The index of the second attack animation.
    pub shoot2: usize,
    /// The optional index of the reload animation.
    pub reload: Option<usize>,

    /// The long the first attack animation needs to be played before you can attack again.
    #[serde(default = "WeaponData::default_firetime")]
    pub fire_time1: f32,
    /// How long the first attack animation is.Needs to be exact, or the idle animation will
    /// be played to early.
    #[serde(default = "WeaponData::default_firetime")]
    pub anim_time1: f32,
    /// The long the second attack animation needs to be played before you can attack again.
    #[serde(default = "WeaponData::default_firetime")]
    pub fire_time2: f32,
    /// How long the second attack animation is. Needs to be exact, or the idle animation will
    /// be played to early.
    #[serde(default = "WeaponData::default_firetime")]
    pub anim_time2: f32,

    /// When you can attack again in the middle of the reload animation.
    #[serde(default = "WeaponData::default_firetime")]
    pub reload_time_skip: f32,
    /// How long the reload animation is. Needs to be exact, or the idle animation will
    /// be played to early.
    #[serde(default = "WeaponData::default_firetime")]
    pub reload_time: f32,
}
