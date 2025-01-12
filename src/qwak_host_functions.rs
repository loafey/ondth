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
use qwak_helper_types::{MapInteraction, MsgVec3};
use qwak_shared::QwakHostFunctions;

qwak_shared::host_gen!(Host);
struct Host;
impl QwakHostFunctions for Host {
    fn game_get_player_name(id: u64) -> String {
        let (nw, _, _) = get_nw!();
        let name = nw
            .lobby
            .get(&id)
            .map(|pi| pi.name.to_string())
            .unwrap_or_else(|| "unknown player".to_string());
        name
    }

    fn log_error(message: String) {
        bevy::log::error!(target: "plugin", "{message}");
    }

    #[allow(clippy::print_stderr)]
    fn log_debug(value: String) {
        bevy::log::debug!(target: "plugin", "{value}");
    }

    fn game_broadcast_message(value: String) {
        let (nw, server, _) = get_nw!();
        transmit_message(server, nw, value);
    }

    fn game_brush_translate(target_name: String, x: f32, y: f32, z: f32, delay: u32) {
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

    fn game_brush_rotate(target_name: String, x: f32, y: f32, z: f32, delay: u32) {
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

    fn game_play_sound(path: String, volume: f32) {
        let (_, server, sw) = get_nw!();
        let translate = ServerMessage::PlaySoundGlobally {
            sound: path.into(),
            volume,
        };
        let bytes = error_return!(translate.bytes());
        sw.send(translate);
        server.broadcast_message(ServerChannel::NetworkedEntities as u8, bytes);
    }

    fn game_hurt_player(id: u64, damage: f32) {
        let (nw, server, sw) = get_nw!();
        for (_, mut hit_player, _) in &mut nw.players {
            if hit_player.id == id {
                let msg = ServerMessage::Hit { amount: damage };
                if hit_player.id != nw.current_id.0 {
                    hit_player.last_hurter = id;
                    hit_player.health -= damage;
                    server.send_message(
                        hit_player.id,
                        ServerChannel::NetworkedEntities as u8,
                        error_continue!(msg.bytes()),
                    )
                } else {
                    sw.send(ServerMessage::Hit { amount: damage });
                }
            }
        }
    }

    fn game_timeout(map_interaction: MapInteraction, delay: u32) {
        let (_, server, sw) = get_nw!();
        let translate = ServerMessage::CreateTimer {
            delay,
            map_interaction,
        };
        let bytes = error_return!(translate.bytes());
        sw.send(translate);
        server.broadcast_message(ServerChannel::NetworkedEntities as u8, bytes);
    }

    fn game_teleport_player(id: u64, x: f32, y: f32, z: f32) {
        let (nw, server, sw) = get_nw!();
        let msg = ServerMessage::TeleportPlayer {
            location: Vec3::new(x, y, z),
        };
        if id == nw.current_id.0 {
            sw.send(msg);
        } else {
            server.send_message(
                id,
                ServerChannel::NetworkedEntities as u8,
                error_return!(msg.bytes()),
            );
        }
    }

    fn game_get_spawn_point() -> MsgVec3 {
        let (nw, _, _) = get_nw!();
        let v = nw.player_spawn.0;
        MsgVec3 {
            x: v.x,
            y: v.y,
            z: v.z,
        }
    }
}
