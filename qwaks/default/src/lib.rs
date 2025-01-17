#![allow(missing_docs)]
#![feature(thread_local)]
use extism_pdk::Msgpack;
use faststr::FastStr;
use qwak_helper_types::{
    MapInteraction, PickupData, PlayerKilled, PlayerLeave, Projectile, WeaponData,
};
use qwak_shared::QwakPlugin;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{
    any::type_name,
    collections::{HashMap, HashSet},
};

mod pickups;
mod projectiles;
mod weapons;

qwak_shared::plugin_gen!(Plugin);
qwak_shared::host_calls!();
use host::*;

#[derive(Debug, Default, Deserialize, Serialize)]
struct PlayerStats {
    kills: usize,
    deaths: usize,
}

fn storage_init() {
    if let Ok(Some(keys)) = extism_pdk::var::get::<Msgpack<HashSet<String>>>("----Keys----") {
        for k in keys.0 {
            extism_pdk::var::remove(k).unwrap();
        }
    }

    extism_pdk::var::set("----Keys----", Msgpack(HashSet::<String>::new())).unwrap();
}
fn storage_set<T: Serialize>(val: T) {
    let mut keys: HashSet<String> = extism_pdk::var::get("----Keys----")
        .unwrap()
        .map(|p: Msgpack<HashSet<String>>| p.0)
        .unwrap_or_default();
    keys.insert(type_name::<T>().to_string());
    extism_pdk::var::set("----Keys----", Msgpack(keys)).unwrap();
    extism_pdk::var::set(type_name::<T>(), Msgpack(val)).unwrap();
}
fn storage_get<T: DeserializeOwned>() -> Option<T> {
    extism_pdk::var::get::<Msgpack<T>>(type_name::<T>())
        .ok()
        .and_then(|o| o)
        .map(|o| o.0)
}

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
                let name = game::player::get_name(player_id);
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
                game::brush::rotate(target, x, y, z, delay);
                // host::brush_translate(target, x, y, z, delay);
            }
            "open_big_doors" => {
                #[derive(Default, Deserialize, Serialize)]
                struct BoolDoor(bool);
                if storage_get::<BoolDoor>().unwrap_or_default().0 {
                    return;
                }
                game::brush::rotate("bigDoor1".to_string(), 0.0, 50.0, 0.0, 100000);
                game::brush::translate("bigDoor1".to_string(), 0.5, 0.0, -0.5, 100000);
                game::brush::rotate("bigDoor2".to_string(), 0.0, -50.0, 0.0, 100000);
                game::brush::translate("bigDoor2".to_string(), 0.5, 0.0, 0.5, 100000);
                storage_set(BoolDoor(true));
                for i in 0..4 {
                    game::map::timeout(
                        MapInteraction {
                            script: "play_sound".to_string(),
                            target: None,
                            argument: Some("[\"sounds/World/Door/scrape-1.ogg\", 0.5]".to_string()),
                            player_id,
                        },
                        i * 750,
                    );
                }
                game::map::timeout(
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
                let (sound, volume): (String, f32) =
                    serde_json::from_str(&argument.unwrap()).unwrap();
                game::audio::global::play(sound, volume);
            }
            "elevator" => {
                #[derive(Default, Deserialize, Serialize)]
                struct FlipFlop(bool);
                let k = storage_get::<FlipFlop>().unwrap_or_default().0;
                let target = "elevator".to_string();
                if k {
                    game::brush::translate(target, 0.0, -2.0, 0.0, 60000);
                    game::broadcast_message("going down".to_string());
                } else {
                    game::brush::translate(target, 0.0, 2.0, 0.0, 60000);
                    game::broadcast_message("going up".to_string());
                }
                storage_set(FlipFlop(!k));
            }
            "hurt_me" => {
                game::broadcast_message("OUCH!".to_string());
                game::player::hurt(player_id, 10.0);
            }
            "heal_me" => {
                game::broadcast_message("DE-OUCH!".to_string());
                game::player::heal(player_id, 10.0);
            }
            _ => panic!("unknown interaction: {script}"),
        }
    }

    fn map_get_lobby_info() -> String {
        let player_info = storage_get::<HashMap<u64, PlayerStats>>().unwrap_or_default();
        let mut s = "lobby info:".to_string();
        for (p, v) in player_info.iter() {
            s += &format!(
                "\n{}: d: {}, k: {}",
                game::player::get_name(*p).to_lowercase(),
                v.deaths,
                v.kills
            );
        }
        s
    }

    fn map_init() {
        log::debug("clearing map storage...".to_string());
        storage_init();
        let mut player_info = storage_get::<HashMap<u64, PlayerStats>>().unwrap_or_default();
        player_info.clear();
        player_info.insert(game::host_id(), PlayerStats::default());
        storage_set(player_info);
    }

    fn map_player_killed(PlayerKilled { player_id, by_id }: PlayerKilled) {
        let killed = game::player::get_name(player_id);
        let killer = game::player::get_name(by_id.unwrap_or_default());
        game::broadcast_message(format!(
            "{} GOT FRAGGED BY {}!",
            killed.to_lowercase(),
            killer.to_uppercase()
        ));

        let mut player_info = storage_get::<HashMap<u64, PlayerStats>>().unwrap_or_default();
        player_info.entry(player_id).or_default().deaths += 1;
        player_info
            .entry(by_id.unwrap_or_default())
            .or_default()
            .kills += 1;
        storage_set(player_info);
    }
    fn map_player_respawn(PlayerKilled { player_id, .. }: PlayerKilled) {
        let spawn = game::map::spawn_point();
        game::player::set_stats(player_id, 100.0, 0.0);
        game::player::teleport(player_id, spawn.x, spawn.y, spawn.z);
    }

    fn map_player_join(id: u64) {
        game::broadcast_message(format!(
            "{} JOINED",
            game::player::get_name(id).to_lowercase()
        ));

        let mut player_info = storage_get::<HashMap<u64, PlayerStats>>().unwrap_or_default();
        player_info.insert(id, PlayerStats::default());
        storage_set(player_info);
    }

    fn map_player_leave(PlayerLeave { id, reason }: PlayerLeave) {
        game::broadcast_message(format!(
            "{} LEFT ({reason})",
            game::player::get_name(id).to_lowercase()
        ));

        let mut player_info = storage_get::<HashMap<u64, PlayerStats>>().unwrap_or_default();
        player_info.remove(&id);
        storage_set(player_info);
    }
}
