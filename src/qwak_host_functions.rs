#![allow(static_mut_refs)]
use crate::{
    get_nw,
    net::{
        ServerChannel, ServerMessage,
        server::{NW_PTR, transmit_message},
    },
};
use bevy::math::Vec3;
pub use inner::functions as qwak_functions;
use macros::{error_return, option_return};
use qwak_shared::QwakHostFunctions;

qwak_shared::host_gen!(Host);
struct Host;
impl QwakHostFunctions for Host {
    fn get_player_name(id: u64) -> String {
        let (nw, _, _) = get_nw!();
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
        let (nw, server, _) = get_nw!();
        transmit_message(server, nw, value);
    }

    fn target_translate(target_name: String, x: f32, y: f32, z: f32) {
        let (nw, server, _) = get_nw!();
        let target_name = target_name.into();
        let target = option_return!(nw.targets.get(&target_name));
        let (_, mut t) = option_return!(nw.target_brushes.get_mut(*target).ok());
        t.translation += Vec3::new(x, y, z);

        let pickup_message_wrapped = ServerMessage::TranslateBrush {
            target: target_name,
            translation: Vec3::new(x, y, z),
        };
        let bytes = error_return!(pickup_message_wrapped.bytes());
        server.broadcast_message(ServerChannel::NetworkedEntities as u8, bytes);
    }
}
