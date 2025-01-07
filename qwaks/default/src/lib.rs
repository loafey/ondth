#![allow(missing_docs)]
#![feature(thread_local)]
use std::cell::{LazyCell, RefCell};

use qwak_helper_types::{MapInteraction, TypeMap, storage, storage_get, storage_put};
use qwak_shared::QwakPlugin;

qwak_shared::plugin_gen!(Plugin);
qwak_shared::host_calls!();

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
                let name = host::get_player_name(player_id);
                host::broadcast_message(format!("{name}: script: {script:?}, target: {target:?}"))
            }
            "translate_brush" => {
                let Some(target) = target else { return };
                // Parse the argument, or return a default value
                let ([x, y, z], delay) = if let Some(arg) = argument {
                    match serde_json::from_str(&arg) {
                        Ok(o) => o,
                        Err(e) => {
                            host::print_error(format!(
                                "{}:{}:{}: {e}",
                                file!(),
                                line!(),
                                column!()
                            ));
                            ([0.0, 0.1, 0.0], 100u32)
                        }
                    }
                } else {
                    ([0.0, 0.1, 0.0], 100)
                };
                host::brush_rotate(target, x, y, z, delay);
                // host::brush_translate(target, x, y, z, delay);
            }
            "open_big_doors" => {
                #[derive(Clone, Copy, Default)]
                struct BoolDoor(bool);
                if storage_get!(BoolDoor).unwrap_or_default().0 {
                    return;
                }
                host::brush_rotate("bigDoor1".to_string(), 0.0, 50.0, 0.0, 100000);
                host::brush_translate("bigDoor1".to_string(), 0.5, 0.0, -0.5, 100000);
                host::brush_rotate("bigDoor2".to_string(), 0.0, -50.0, 0.0, 100000);
                host::brush_translate("bigDoor2".to_string(), 0.5, 0.0, 0.5, 100000);
                storage_put!(BoolDoor(true));
            }
            _ => panic!("unknown interaction: {script}"),
        }
    }
}
