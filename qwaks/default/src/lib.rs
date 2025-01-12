#![allow(missing_docs)]
#![feature(thread_local)]
use faststr::FastStr;
use qwak_helper_types::{
    MapInteraction, PickupData, PlayerKilled, Projectile, TypeMap, WeaponData, storage,
    storage_clear, storage_get, storage_put,
};
use qwak_shared::QwakPlugin;
use std::{
    cell::{LazyCell, RefCell},
    collections::HashMap,
};

mod pickups;
mod projectiles;
mod weapons;

qwak_shared::plugin_gen!(Plugin);
qwak_shared::host_calls!();
use host::*;

storage!();

// Simple QWAK plugin that contains the required functions.
// This is compiled to WASM.
struct Plugin;
impl QwakPlugin for Plugin {
    fn plugin_init() {}

    fn plugin_name() -> String {
        "Ondth".to_string()
    }

    fn plugin_version() -> [i32; 3] {
        [0, 0, 1]
    }

    fn plugin_get_projectiles() -> HashMap<FastStr, Projectile> {
        projectiles::get_projectiles()
    }

    fn plugin_get_pickups() -> HashMap<FastStr, PickupData> {
        pickups::get_pickups()
    }

    fn plugin_get_weapons() -> HashMap<FastStr, WeaponData> {
        weapons::get_weapons()
    }

    // The functions scriptable entities can call
    fn map_interact(
        MapInteraction {
            script,
            target,
            player_id,
            argument,
        }: MapInteraction,
    ) {
        match &*script {
            "debug_log" => {
                let name = game::get_player_name(player_id);
                game::broadcast_message(format!("{name}: script: {script:?}, target: {target:?}"))
            }
            "translate_brush" => {
                let Some(target) = target else { return };
                // Parse the argument, or return a default value
                let ([x, y, z], delay) = if let Some(arg) = argument {
                    match serde_json::from_str(&arg) {
                        Ok(o) => o,
                        Err(e) => {
                            log::error(format!("{}:{}:{}: {e}", file!(), line!(), column!()));
                            ([0.0, 0.1, 0.0], 100u32)
                        }
                    }
                } else {
                    ([0.0, 0.1, 0.0], 100)
                };
                game::brush_rotate(target, x, y, z, delay);
                // host::brush_translate(target, x, y, z, delay);
            }
            "open_big_doors" => {
                #[derive(Clone, Copy, Default)]
                struct BoolDoor(bool);
                if storage_get!(BoolDoor).unwrap_or_default().0 {
                    return;
                }
                game::brush_rotate("bigDoor1".to_string(), 0.0, 50.0, 0.0, 100000);
                game::brush_translate("bigDoor1".to_string(), 0.5, 0.0, -0.5, 100000);
                game::brush_rotate("bigDoor2".to_string(), 0.0, -50.0, 0.0, 100000);
                game::brush_translate("bigDoor2".to_string(), 0.5, 0.0, 0.5, 100000);
                storage_put!(BoolDoor(true));
                for i in 0..4 {
                    game::timeout(
                        MapInteraction {
                            script: "play_sound".to_string(),
                            target: None,
                            argument: Some("[\"sounds/World/Door/scrape-1.ogg\", 0.5]".to_string()),
                            player_id,
                        },
                        i * 750,
                    );
                }
                game::timeout(
                    MapInteraction {
                        script: "play_sound".to_string(),
                        target: None,
                        argument: Some("[\"sounds/World/Door/Slam.ogg\", 1.5]".to_string()),
                        player_id,
                    },
                    2750,
                );
            }
            "play_sound" => {
                let (sound, volume) = serde_json::from_str(&argument.unwrap()).unwrap();
                game::play_sound(sound, volume);
            }
            "elevator" => {
                #[derive(Clone, Copy, Default)]
                struct FlipFlop(bool);
                let k = storage_get!(FlipFlop).unwrap_or_default().0;
                let target = "elevator".to_string();
                if k {
                    game::brush_translate(target, 0.0, -2.0, 0.0, 60000);
                    game::broadcast_message("going down".to_string());
                } else {
                    game::brush_translate(target, 0.0, 2.0, 0.0, 60000);
                    game::broadcast_message("going up".to_string());
                }
                storage_put!(FlipFlop(!k))
            }
            "hurt_me" => {
                game::broadcast_message("OUCH!".to_string());
                game::hurt_player(player_id, 10.0);
            }
            _ => panic!("unknown interaction: {script}"),
        }
    }

    fn map_init() {
        log::debug("clearing map storage...".to_string());
        storage_clear!();
    }

    fn map_player_killed(PlayerKilled { player_id, by_id }: PlayerKilled) {
        let killed = game::get_player_name(player_id);
        let killer = game::get_player_name(by_id.unwrap_or_default());
        game::broadcast_message(format!(
            "{} GOT FRAGGED BY {}!",
            killed.to_lowercase(),
            killer.to_uppercase()
        ));
    }
}
