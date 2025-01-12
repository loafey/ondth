//! Contains the traits which define the functions the game can call to interact
//! with a plugin, how the plugins can interact with the host.

#![allow(clippy::unused_unit)]

/// The functions a plugin needs to define.
#[qwak_macro::plugin]
pub trait QwakPlugin {
    #[doc = "Called when a plugin is loaded. Can be used to call functions which for example sets up your runtime etc."]
    fn plugin_init() -> ();
    #[doc = "Returns the name of a plugin."]
    fn plugin_name() -> String;
    #[doc = "Returns the version of a plugin."]
    fn plugin_version() -> [i32; 3];
    #[doc = "Function called by the game when a map is loaded."]
    fn map_init() -> ();
    #[doc = "The function which defines the scripts `interactable` entities can call in a map."]
    fn map_interact(args: qwak_helper_types::MapInteraction) -> ();
    #[doc = "Function called by the game when a player is killed."]
    fn map_player_killed(args: qwak_helper_types::PlayerKilled) -> ();

    #[doc = "The projectiles this plugin defines."]
    fn plugin_get_projectiles()
    -> std::collections::HashMap<faststr::FastStr, qwak_helper_types::Projectile>;
    #[doc = "The pickups this plugin defines."]
    fn plugin_get_pickups()
    -> std::collections::HashMap<faststr::FastStr, qwak_helper_types::PickupData>;
    #[doc = "The weapons this plugin defines."]
    fn plugin_get_weapons()
    -> std::collections::HashMap<faststr::FastStr, qwak_helper_types::WeaponData>;
}

#[allow(clippy::unnecessary_safety_doc)]
#[qwak_macro::host]
/// The functions a the game defines for plugin -> game interaction.
pub trait QwakHostFunctions {
    #[doc = "Log an error."]
    fn log_error(message: String);
    #[doc = "Prints to `stdout`."]
    fn log_debug(val: String);

    #[doc = "Sends a message to all players."]
    #[doc = "# Safety"]
    #[doc = "Will segfault if ran outside of the game."]
    fn game_broadcast_message(val: String);
    #[doc = "Returns the player name of a specified id."]
    #[doc = "# Safety"]
    #[doc = "Will segfault if ran outside of the game."]
    fn game_get_player_name(id: u64) -> String;
    #[doc = "Move a brush by the vector provided."]
    #[doc = "# Safety"]
    #[doc = "Will segfault if ran outside of the game."]
    fn game_brush_translate(target: String, x: f32, y: f32, z: f32, duration: u32);
    #[doc = "Rotate a brush by the vector provided."]
    #[doc = "# Safety"]
    #[doc = "Will segfault if ran outside of the game."]
    fn game_brush_rotate(target: String, x: f32, y: f32, z: f32, duration: u32);
    #[doc = "Plays a sound effect globaly."]
    #[doc = "# Safety"]
    #[doc = "Will segfault if ran outside of the game."]
    fn game_play_sound(path: String, volume: f32);
    #[doc = "Run a MapInteract after a set amount of time."]
    fn game_timeout(map_int: qwak_helper_types::MapInteraction, delay: u32);
    #[doc = "Hurt a specific player."]
    #[doc = "# Safety"]
    #[doc = "Will segfault if ran outside of the game."]
    fn game_hurt_player(id: u64, damage: f32);
    #[doc = "Teleport player to the specified location."]
    #[doc = "# Safety"]
    #[doc = "Will segfault if ran outside of the game."]
    fn game_teleport_player(id: u64, x: f32, y: f32, z: f32);
    #[doc = "Teleport player to the specified location."]
    #[doc = "# Safety"]
    #[doc = "Will segfault if ran outside of the game."]
    fn game_get_spawn_point() -> qwak_helper_types::MsgVec3;
}
