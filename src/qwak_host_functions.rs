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
use macros::{error_continue, error_return};
use qwak_helper_types::MapInteraction;
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

    fn print_error(message: String) {
        bevy::log::error!(target: "plugin", "{message}");
    }

    #[allow(clippy::print_stderr)]
    fn debug_log(value: String) {
        bevy::log::debug!(target: "plugin", "{value}");
    }

    fn broadcast_message(value: String) {
        let (nw, server, _) = get_nw!();
        transmit_message(server, nw, value);
    }

    fn brush_translate(target_name: String, x: f32, y: f32, z: f32, delay: u32) {
        let (_, server, sw) = get_nw!();
        let target_name = target_name.into();

        let translate = ServerMessage::TranslateBrush {
            target: target_name,
            translation: Vec3::new(x, y, z),
            delay,
        };
        let bytes = error_return!(translate.bytes());
        sw.send(translate);
        server.broadcast_message(ServerChannel::NetworkedEntities as u8, bytes);
    }

    fn brush_rotate(target_name: String, x: f32, y: f32, z: f32, delay: u32) {
        let (_, server, sw) = get_nw!();
        let target_name = target_name.into();

        let translate = ServerMessage::RotateBrush {
            target: target_name,
            translation: Vec3::new(x, y, z),
            delay,
        };
        let bytes = error_return!(translate.bytes());
        sw.send(translate);
        server.broadcast_message(ServerChannel::NetworkedEntities as u8, bytes);
    }

    fn play_sound(path: String, volume: f32) {
        let (_, server, sw) = get_nw!();
        let translate = ServerMessage::PlaySoundGlobally {
            sound: path.into(),
            volume,
        };
        let bytes = error_return!(translate.bytes());
        sw.send(translate);
        server.broadcast_message(ServerChannel::NetworkedEntities as u8, bytes);
    }

    fn hurt_player(id: u64, damage: f32) {
        let (nw, server, _) = get_nw!();
        for (_, mut hit_player, _) in &mut nw.players {
            hit_player.last_hurter = id;
            hit_player.health -= damage;
            if hit_player.id != nw.current_id.0 {
                server.send_message(
                    hit_player.id,
                    ServerChannel::NetworkedEntities as u8,
                    error_continue!(ServerMessage::Hit { amount: damage }.bytes()),
                )
            }
        }
    }

    fn timeout(map_interaction: MapInteraction, delay: u32) {
        let (_, server, sw) = get_nw!();
        let translate = ServerMessage::CreateTimer {
            delay,
            map_interaction,
        };
        let bytes = error_return!(translate.bytes());
        sw.send(translate);
        server.broadcast_message(ServerChannel::NetworkedEntities as u8, bytes);
    }
}
