use qwak_helper_types::MapInteraction;
use qwak_shared::QwakPlugin;
qwak_shared::plugin_gen!(Plugin);
qwak_shared::host_calls!();

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

    fn map_interact(
        MapInteraction {
            script,
            target,
            player_id,
        }: MapInteraction,
    ) {
        match &*script {
            "debug_log" => {
                let name = host::get_player_name(player_id);
                host::broadcast_message(format!("{name}: script: {script:?}, target: {target:?}"))
            }
            _ => panic!("unknown interaction: {script}"),
        }
    }
}
