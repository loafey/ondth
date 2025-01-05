//! Contains the traits which define the functions the game can call to interact
//! with a plugin, how the plugins can interact with the host.

#![allow(clippy::unused_unit)]
/// The functions a plugin needs to define.
#[qwak_macro::plugin]
pub trait QwakPlugin {
    #[doc = "Called when a plugin is loaded. 
Can be used to call functions which for example sets up your runtime etc."]
    fn plugin_init() -> ();
    #[doc = "Returns the name of a plugin."]
    fn plugin_name() -> String;
    #[doc = "Returns the version of a plugin."]
    fn plugin_version() -> [i32; 3];
    #[doc = "The function which defines the scripts `interactable` entities can call in a map."]
    fn map_interact(args: qwak_helper_types::MapInteraction) -> ();
}

#[qwak_macro::host]
/// The functions a the game defines for plugin -> game interaction.
pub trait QwakHostFunctions {
    #[doc = "Prints to `stdout`."]
    fn debug_log(val: String);
    #[doc = "Sends a message to all players."]
    fn broadcast_message(val: String);
    #[doc = "Returns the player name of a specified id."]
    fn get_player_name(id: u64) -> String;
    #[doc = "Move a brush by the vector provided."]
    fn target_translate(target: String, x: f32, y: f32, z: f32);
}
