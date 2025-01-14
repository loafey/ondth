//! Contains all the resources used by the game.
use bevy::{
    asset::{Handle, UntypedHandle},
    ecs::system::Res,
    image::Image,
    math::Vec3,
    prelude::{Entity, Resource, States},
};
use faststr::FastStr;
use qwak_helper_types::{PickupData, Projectile, WeaponData};
use std::{collections::HashMap, ops::Deref, path::PathBuf};

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

/// Contains true if this is the first frame of a map
#[derive(Resource)]
pub struct MapFirstRun(pub bool);

/// True if the map is done and loaded
#[derive(Resource)]
pub struct MapDoneLoading(pub bool);

/// Returns if the map is done loading.
pub fn if_map_done_loading(val: Res<MapDoneLoading>) -> bool {
    val.0
}

/// True if the player has been spawned
#[derive(Resource)]
pub struct PlayerSpawned(pub bool);

/// Returns if the host player has been spawned.
pub fn if_player_spawned(val: Res<PlayerSpawned>) -> bool {
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

/// A [HashMap] of all projectile data.
#[derive(Debug, Resource)]
pub struct Projectiles(pub HashMap<FastStr, Projectile>);

/// A map with weapon data
#[derive(Debug, Resource, Default)]
pub struct WeaponMap(pub HashMap<FastStr, WeaponData>);

/// A struct containing a [HashMap] containing ids and their respective [Entities](bevy::prelude::Entity).
#[derive(Debug, Resource, Default)]
pub struct TargetMap(pub HashMap<FastStr, Vec<Entity>>);
impl Deref for TargetMap {
    type Target = HashMap<FastStr, Vec<Entity>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
