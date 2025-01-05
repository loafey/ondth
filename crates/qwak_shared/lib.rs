#![allow(clippy::unused_unit)]
#[qwak_macro::plugin]
pub trait QwakPlugin {
    fn plugin_init() -> ();
    fn plugin_name() -> String;
    fn plugin_version() -> [i32; 3];
    fn map_interact(args: qwak_helper_types::MapInteraction) -> ();
}

#[qwak_macro::host]
pub trait QwakHostFunctions {
    fn debug_log(val: String);
    fn broadcast_message(val: String);
    fn get_player_name(id: u64) -> String;
    fn target_translate(target: String, x: f32, y: f32, z: f32);
}
