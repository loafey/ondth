//! Side game void

use std::collections::HashMap;

use faststr::FastStr;
use qwak_helper_types::{
    MapInteraction, PickupData, PlayerKilled, PlayerLeave, Projectile, WeaponData,
};
use qwak_shared::QwakPlugin;
qwak_shared::plugin_gen!(Plugin);
qwak_shared::host_calls!();
use host::*;

struct Plugin;
impl QwakPlugin for Plugin {
    fn plugin_init() {}

    fn plugin_name() -> String {
        "Void".to_string()
    }

    fn plugin_version() -> [i32; 3] {
        [0, 0, 1]
    }

    fn map_init() {}

    fn map_interact(args: MapInteraction) {}

    fn map_player_killed(args: PlayerKilled) {}

    fn map_player_respawn(args: PlayerKilled) {}

    fn map_get_lobby_info() -> String {
        "Empty".to_string()
    }

    fn map_player_join(id: u64) {}

    fn map_player_leave(args: PlayerLeave) {}

    fn plugin_get_projectiles() -> HashMap<FastStr, Projectile> {
        HashMap::new()
    }

    fn plugin_get_pickups() -> HashMap<FastStr, PickupData> {
        HashMap::new()
    }

    fn plugin_get_weapons() -> HashMap<FastStr, WeaponData> {
        HashMap::new()
    }
}
