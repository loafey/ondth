//! Contains all the resources used by the game.
use bevy::{
    asset::{Handle, UntypedHandle},
    ecs::system::{Res, Resource},
    image::Image,
    log::info,
    math::Vec3,
    prelude::{Entity, States},
};
use data::{PickupData, WeaponData};
use faststr::FastStr;
use macros::error_return;
use std::{collections::HashMap, fs, ops::Deref, path::PathBuf};

/// Contains data definitions for weapons, enemies etc.
pub mod data;
/// Contains the structs for randomeness.
pub mod entropy;
/// Contains the struct for player input.
pub mod inputs;

/// Represents the current game stage
#[derive(Debug, Resource, PartialEq, Eq, States, Default, Hash, Clone, Copy)]
pub enum CurrentStage {
    /// The boot screen.
    #[default]
    Startup,
    /// The main menu screen.
    MainMenu,
    /// In game.
    InGame,
}

/// String to the current map
#[derive(Debug, Resource)]
pub struct CurrentMap(pub PathBuf);

/// Represents the pause state of the game
#[derive(Debug, Resource)]
pub struct Paused(pub bool);
/// Returns if the game paused or not.
#[allow(unused)]
pub fn if_not_paused(val: Res<Paused>) -> bool {
    !val.0
}

/// True if the map is done and loaded
#[derive(Resource)]
pub struct MapDoneLoading(pub bool);

/// Returns if the map is done loading.
pub fn if_map_done_loading(val: Res<MapDoneLoading>) -> bool {
    val.0
}

/// Represents where a player will spawn in the current level
#[derive(Resource)]
pub struct PlayerSpawnpoint(pub Vec3);

/// A list of which textures are currently being loaded
#[derive(Debug, Resource, Default)]
pub struct TexturesLoading(pub Vec<UntypedHandle>);

/// The different states for texture loading.
#[derive(Debug, Resource)]
pub enum TextureLoadingState {
    /// Textures are not loaded.
    NotLoaded,
    /// Textures are loading.
    Loading,
    /// Textures are loaded.
    Done,
}

/// Check if textures are not loaded.
#[allow(dead_code)]
pub fn if_textures_not_loaded(text: Res<TextureLoadingState>) -> bool {
    matches!(*text, TextureLoadingState::NotLoaded)
}
/// Check if textures are loading.
pub fn if_texture_loading(text: Res<TextureLoadingState>) -> bool {
    matches!(*text, TextureLoadingState::Loading)
}
/// Check if textures are loaded.
pub fn if_texture_done_loading(text: Res<TextureLoadingState>) -> bool {
    matches!(*text, TextureLoadingState::Done)
}

/// A map which provides Path -> Handle for textures
#[derive(Debug, Resource, Default)]
pub struct TextureMap(pub HashMap<FastStr, Handle<Image>>);

/// A map with pickup data
#[derive(Debug, Resource, Default)]
pub struct PickupMap(pub HashMap<FastStr, PickupData>);
impl PickupMap {
    /// Loads the pickup data from disc.
    pub fn new() -> Self {
        info!("Loading pickups...");
        let data = error_return!(fs::read_to_string("assets/pickups.json"));
        let parsed = error_return!(serde_json::from_str::<Vec<PickupData>>(&data));

        let mut map = HashMap::new();
        for item in parsed {
            map.insert(item.classname.clone(), item);
        }

        info!("Done loading pickups...");
        Self(map)
    }
}

/// A map with weapon data
#[derive(Debug, Resource, Default)]
pub struct WeaponMap(pub HashMap<FastStr, WeaponData>);
impl WeaponMap {
    /// Loads the weapon data from disc.
    pub fn new() -> Self {
        info!("Loading weapons...");
        let data = error_return!(fs::read_to_string("assets/weapons.json"));
        let parsed = error_return!(serde_json::from_str::<Vec<WeaponData>>(&data));

        let mut map = HashMap::new();
        for item in parsed {
            map.insert(item.id.clone(), item);
        }

        info!("Done loading weapons...");
        Self(map)
    }
}

/// A struct containing a [HashMap] containing ids and their respective [Entities](bevy::prelude::Entity).
#[derive(Debug, Resource, Default)]
pub struct TargetMap(pub HashMap<FastStr, Vec<Entity>>);
impl Deref for TargetMap {
    type Target = HashMap<FastStr, Vec<Entity>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
