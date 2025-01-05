#![allow(static_mut_refs)]
use crate::{
    get_nw,
    net::server::{NW_PTR, transmit_message},
};
pub use inner::functions as qwak_functions;
use qwak_shared::QwakHostFunctions;

qwak_shared::host_gen!(Host);
struct Host;
impl QwakHostFunctions for Host {
    fn get_player_name(id: u64) -> String {
        let (nw, _) = get_nw!();
        let name = nw
            .lobby
            .get(&id)
            .map(|pi| pi.name.to_string())
            .unwrap_or_else(|| "unknown player".to_string());
        name
    }

    fn debug_log(value: String) {
        println!("{value}");
    }

    fn broadcast_message(value: String) {
        let (nw, server) = get_nw!();
        transmit_message(server, nw, value);
    }

    fn target_translate(target: String, x: f32, y: f32, z: f32) {
        let (nw, server) = get_nw!();
        let Some(target) = nw.targets.get(&(target.into())) else {
            return;
        };
        println!("{target} move {x} {y} {z}");
    }
}
