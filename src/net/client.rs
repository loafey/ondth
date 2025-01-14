use super::{
    CurrentClientId, IsSteam, NetState, PROTOCOL_ID, ServerChannel, ServerMessage, SteamClient,
    connection_config, update_world,
};
use crate::{
    entities::{hitscan_hit_gfx, pickup::PickupEntity},
    map_gen::{
        self,
        world_entites::{RotateBrush, Timer, TranslateBrush},
    },
    net::{Lobby, PlayerInfo},
    player::Player,
    queries::NetWorld,
};
use bevy::{
    audio::{AudioPlayer, PlaybackSettings, Volume},
    ecs::{
        entity::Entity,
        event::EventReader,
        schedule::{IntoSystemConfigs, SystemConfigs, common_conditions::resource_exists},
        system::{Query, Res, ResMut},
        world::World,
    },
    hierarchy::DespawnRecursiveExt,
    log::info,
    math::{EulerRot, Vec3},
    prelude::{Commands, EventWriter, NextState},
};
use bevy_renet::{
    netcode::{ClientAuthentication, NetcodeClientTransport, NetcodeTransportError},
    renet::RenetClient,
    steam::SteamTransportError,
};
use macros::{error_continue, error_return, option_continue};
use renet_steam::SteamClientTransport;
use resources::{CurrentMap, CurrentStage};
use std::{net::UdpSocket, time::SystemTime};
use steamworks::SteamId;

pub fn get_events(mut client: ResMut<RenetClient>, mut server_events: EventWriter<ServerMessage>) {
    while let Some(message) = client.receive_message(ServerChannel::ServerMessages as u8) {
        let message = error_continue!(ServerMessage::from_bytes(&message));
        server_events.send(message);
    }
    while let Some(message) = client.receive_message(ServerChannel::NetworkedEntities as u8) {
        let message = error_continue!(ServerMessage::from_bytes(&message));
        server_events.send(message);
    }
}

pub fn handle_messages(
    pickups: Query<(Entity, &PickupEntity)>,
    mut current_stage: ResMut<CurrentMap>,
    mut state: ResMut<NextState<CurrentStage>>,
    mut nw: NetWorld,
    mut server_events: EventReader<ServerMessage>,
) {
    for message in server_events.read() {
        match message.clone() {
            ServerMessage::PlaySoundGlobally { sound, volume } => {
                nw.commands.spawn((
                    AudioPlayer::new(nw.asset_server.load(sound.to_string())),
                    PlaybackSettings::DESPAWN.with_volume(Volume::new(volume)),
                ));
            }
            ServerMessage::SetMap(map) => {
                info!("setting map to: {map:?}");
                current_stage.0 = map;
                state.set(CurrentStage::InGame);
            }
            ServerMessage::SpawnPlayer {
                id,
                translation,
                weapons,
                name,
            } => {
                if id != nw.current_id.0 {
                    let entity = Player::spawn(&mut nw, false, translation, id, weapons, None);
                    nw.lobby.insert(id, PlayerInfo::new(entity, name));
                }
            }
            ServerMessage::DespawnPlayer { id } => {
                let player = option_continue!(nw.lobby.get(&id)).entity;
                nw.commands.entity(player).despawn_recursive();
                nw.lobby.remove(&id);
            }
            ServerMessage::Reset => {
                let player = option_continue!(nw.lobby.get(&nw.current_id.0)).entity;
                let (_, mut player, _) = error_continue!(nw.players.get_mut(player));
                player.health = 100.0;
                player.armour = 0.0;
                player.last_hurter = 0;
            }
            ServerMessage::SpawnPickup {
                id,
                translation,
                data,
            } => {
                map_gen::entities::spawn_pickup(
                    id,
                    false,
                    translation,
                    &nw.asset_server,
                    &data,
                    &mut nw.commands,
                    &mut nw.materials,
                );
            }
            ServerMessage::Message { text } => {
                let player = option_continue!(nw.lobby.get(&nw.current_id.0)).entity;
                let (_, player, _) = error_continue!(nw.players.get(player));
                player.display_message(&mut nw.commands, &nw.asset_server, text);
            }
            ServerMessage::KillStat { death, hurter } => {
                if let Some(info) = nw.lobby.get_mut(&death) {
                    info.deaths += 1;
                }
                if let Some(hurter) = hurter
                    && let Some(info) = nw.lobby.get_mut(&hurter)
                {
                    info.kills += 1;
                }
            }
            ServerMessage::PlayerUpdate { id, message } => {
                update_world(id, &message, &mut nw);
            }
            ServerMessage::TranslateBrush {
                target,
                translation,
                delay,
            } => {
                let target = option_continue!(nw.targets.get(&target));
                for target in target {
                    let (e, t) = option_continue!(nw.target_brushes.get(*target).ok());
                    let mut command = option_continue!(nw.commands.get_entity(e));
                    let time = match delay {
                        0 => 0.0,
                        _ => (delay as f32) / 1000.0,
                    };
                    command.insert(TranslateBrush::new(t.translation + translation, time));
                }
            }
            ServerMessage::RotateBrush {
                target,
                translation,
                delay,
            } => {
                let target = option_continue!(nw.targets.get(&target));
                for target in target {
                    let (e, t) = option_continue!(nw.target_brushes.get(*target).ok());
                    let mut command = option_continue!(nw.commands.get_entity(e));
                    let translation = Vec3::new(
                        translation.x.to_radians(),
                        translation.y.to_radians(),
                        translation.z.to_radians(),
                    );
                    let time = match delay {
                        0 => 0.0,
                        _ => (delay as f32) / 1000.0,
                    };
                    let (x, y, z) = t.rotation.to_euler(EulerRot::XYZ);
                    command.insert(RotateBrush::new(
                        Vec3::new(x, y, z) + translation,
                        Vec3::new(x, y, z),
                        time,
                    ));
                }
            }
            ServerMessage::DespawnPickup { id } => {
                // TODO: Improve this
                for (ent, pickup) in &pickups {
                    if pickup.id == id {
                        nw.commands.entity(ent).despawn_recursive();
                    }
                }
            }
            ServerMessage::HitscanHits { hits } => {
                hitscan_hit_gfx(&nw.asset_server, &mut nw.commands, &hits, &nw.particles)
            }
            ServerMessage::Hit { amount } => {
                let player = option_continue!(nw.lobby.get(&nw.current_id.0)).entity;
                let (_, mut player, _) = error_continue!(nw.players.get_mut(player));
                player.health -= amount;
                nw.commands.spawn((
                    AudioPlayer::new(nw.asset_server.load("sounds/BulletHit.ogg")),
                    PlaybackSettings::DESPAWN.with_volume(Volume::new(0.5)),
                ));
                player.hurt_flash += (amount / 2.0) / player.max_health;
                if amount >= 30.0 {
                    nw.commands.spawn((
                        AudioPlayer::new(nw.asset_server.load("sounds/Player/Hurt/hurt.ogg")),
                        PlaybackSettings::DESPAWN.with_volume(Volume::new(0.5)),
                    ));
                }
            }
            ServerMessage::CreateTimer {
                delay,
                map_interaction,
            } => {
                nw.commands
                    .spawn(Timer::new(delay as f32 / 1000.0, map_interaction));
            }
            ServerMessage::TeleportPlayer { location } => {
                for (_, player, mut trans) in &mut nw.players {
                    if player.id == nw.current_id.0 {
                        trans.translation = location;
                        break;
                    }
                }
            }
            ServerMessage::LobbyInfo(fast_str) => {
                for (_, mut player, _) in &mut nw.players {
                    if player.id == nw.current_id.0 {
                        player.lobby_info = fast_str;
                        break;
                    }
                }
            }
        }
    }
}

pub fn init_client(
    world: &mut World,
    next_state: &mut NextState<NetState>,
    ip: &String,
    steam_client: &Option<Res<SteamClient>>,
) -> bool {
    info!("joining: {ip}");
    let client = RenetClient::new(connection_config());

    if let Some(sc) = steam_client {
        let server_steam_id = SteamId::from_raw(error_return!(ip.parse()));

        sc.networking_utils().init_relay_network_access();

        let transport = error_return!(SteamClientTransport::new(sc, &server_steam_id));

        world.insert_resource(transport);
        world.insert_resource(CurrentClientId(sc.user().steam_id().raw()));
    } else {
        let current_time = error_return!(SystemTime::now().duration_since(SystemTime::UNIX_EPOCH));

        let server_addr = error_return!(ip.parse());
        let socket = error_return!(UdpSocket::bind("127.0.0.1:0"));

        let client_id = current_time.as_micros() as u64;

        let authentication = ClientAuthentication::Unsecure {
            client_id,
            protocol_id: PROTOCOL_ID,
            server_addr,
            user_data: None,
        };

        let transport = error_return!(NetcodeClientTransport::new(
            current_time,
            authentication,
            socket
        ));

        world.insert_resource(transport);
        world.insert_resource(CurrentClientId(client_id));
    }
    world.insert_resource(client);
    world.insert_resource(Lobby::default());
    next_state.set(NetState::Client);
    info!("started client");
    true
}

fn clean_up(mut commands: Commands) {
    commands.remove_resource::<RenetClient>();
    commands.remove_resource::<Lobby>();
}

pub fn system_cleanup() -> SystemConfigs {
    (clean_up,).into_configs()
}

pub fn systems() -> SystemConfigs {
    (get_events,).into_configs()
}
pub fn all_cons() -> SystemConfigs {
    (handle_messages,).into_configs()
}

pub fn errors() -> SystemConfigs {
    (panic_on_error_system.run_if(resource_exists::<NetcodeClientTransport>),).into_configs()
}

pub fn errors_steam() -> SystemConfigs {
    (panic_on_error_system_steam.run_if(resource_exists::<IsSteam>),).into_configs()
}

pub fn panic_on_error_system(mut renet_error: EventReader<NetcodeTransportError>) {
    #[allow(clippy::never_loop)]
    for e in renet_error.read() {
        panic!("{}", e);
    }
}

pub fn panic_on_error_system_steam(mut renet_error: EventReader<SteamTransportError>) {
    #[allow(clippy::never_loop)]
    for e in renet_error.read() {
        panic!("{}", e);
    }
}
