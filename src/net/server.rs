use super::{
    ClientChannel, ClientMessage, Connections, NetState, PROTOCOL_ID, SimulationEvent, SteamClient,
    connection_config, update_world,
};
use crate::{
    net::{CurrentClientId, IsSteam, Lobby, PlayerInfo, ServerChannel, ServerMessage},
    player::Player,
    queries::NetWorld,
};
use bevy::{
    ecs::{
        event::EventReader,
        schedule::{IntoSystemConfigs, SystemConfigs, common_conditions::resource_exists},
        system::{Res, ResMut},
        world::World,
    },
    hierarchy::DespawnRecursiveExt,
    log::{error, info},
    prelude::{EventWriter, NextState},
};
use bevy_renet::{
    netcode::{NetcodeServerTransport, NetcodeTransportError, ServerAuthentication, ServerConfig},
    renet::{RenetServer, ServerEvent},
    steam::SteamTransportError,
};
use faststr::FastStr;
use macros::{error_continue, error_return, option_return};
use qwak_helper_types::{Attack, MapInteraction, PlayerKilled, PlayerLeave};
use renet_steam::{AccessPermission, SteamServerConfig, SteamServerTransport};
use resources::{CurrentMap, MapFirstRun};
use std::{net::UdpSocket, time::SystemTime};
use steamworks::SteamId;

// Yea this is cursed, I know, but somehow these references need to be passed to
// host functions, and I don't feel like creating an EDSL, or passing them through
// the WASM plugin. I promise that this should never segfault :)
pub static mut NW_PTR: Option<(
    &mut NetWorld,
    &mut RenetServer,
    &mut EventWriter<ServerMessage>,
)> = None;
/// Returns the content of [NW_PTR]
#[macro_export]
macro_rules! get_nw {
    () => {{
        // println!("maybe here: {}:{}:{}", file!(), line!(), column!());
        #[allow(unsafe_code)]
        unsafe {
            #[allow(unsafe_code)]
            let nw = NW_PTR.as_ref();
            let r: *const _ = nw.unwrap_unchecked();
            std::ptr::read(r)
        }
    }};
}
/// Setsthe content of [NW_PTR]
#[macro_export]
macro_rules! set_nw {
    ($nw:expr,$server:expr, $server_events:expr) => {
        #[allow(
            clippy::missing_transmute_annotations,
            unsafe_code,
            clippy::macro_metavars_in_unsafe
        )]
        unsafe {
            NW_PTR = Some(std::mem::transmute::<
                (&NetWorld, &RenetServer, &EventWriter<ServerMessage>),
                _,
            >((&*$nw, &*$server, &*$server_events)))
        };
    };
}

pub fn transmit_message(server: &mut RenetServer, nw: &mut NetWorld, text: String) {
    for (_, player, _) in &nw.players {
        if player.id == nw.current_id.0 {
            player.display_message(&mut nw.commands, &nw.asset_server, text.clone());
            break;
        }
    }
    server.broadcast_message(
        ServerChannel::ServerMessages as u8,
        error_return!(ServerMessage::Message { text }.bytes()),
    );
}

fn frag_checker(
    mut server: ResMut<RenetServer>,
    mut nw: NetWorld,
    mut event_writer: EventWriter<ServerMessage>,
) {
    let mut frags = Vec::new();
    for (_, mut player, trans) in &mut nw.players {
        if (player.health <= 0.0 || trans.translation.y < -10000.0) && !player.dead {
            player.dead = true;
            let event = ServerMessage::MarkPlayerAsDead { id: player.id };
            event_writer.send(event.clone());
            server.broadcast_message(
                ServerChannel::ServerMessages as u8,
                error_continue!(event.bytes()),
            );

            // if player.id != nw.current_id.0 {
            // server.send_message(
            // player.id,
            // ServerChannel::ServerMessages as u8,
            // error_continue!(ServerMessage::Reset.bytes()),
            // );
            // }

            frags.push((player.id, player.last_hurter));
            player.last_hurter = 0;
        }
    }

    for (id, hurter) in frags {
        server.broadcast_message(
            ServerChannel::ServerMessages as u8,
            error_continue!(
                ServerMessage::KillStat {
                    death: id,
                    hurter: (hurter != 0).then_some(hurter)
                }
                .bytes()
            ),
        );

        set_nw!(&nw, &server, &event_writer);
        let info = PlayerKilled {
            player_id: id,
            by_id: Some(hurter),
        };
        error_continue!(nw.plugins.default.map_player_killed(info));
        // error_continue!(nw.plugins.default.map_player_respawn(info));
    }
}

#[allow(clippy::type_complexity)]
pub fn server_events(
    mut events: EventReader<ServerEvent>,
    mut connections: EventWriter<Connections>,
    mut sim_events: EventReader<SimulationEvent>,
    mut server: ResMut<RenetServer>,

    steam: Option<Res<SteamClient>>,
    map: Res<CurrentMap>,
    mut nw: NetWorld,
) {
    // Handle connection details
    for event in events.read() {
        match event {
            ServerEvent::ClientConnected { client_id } => {
                connections.send(Connections::Join(*client_id));

                server.send_message(
                    *client_id,
                    ServerChannel::ServerMessages as u8,
                    error_return!(ServerMessage::SetMap(map.0.clone()).bytes()),
                );

                for (pickup, trans) in &nw.pickups_query {
                    server.send_message(
                        *client_id,
                        ServerChannel::ServerMessages as u8,
                        error_continue!(
                            ServerMessage::SpawnPickup {
                                id: pickup.id,
                                translation: trans.translation,
                                data: pickup.data.clone()
                            }
                            .bytes()
                        ),
                    )
                }

                // Spawn players for newly joined client
                for (other_id, info) in &nw.lobby {
                    let (_, pl, trans) = error_continue!(nw.players.get(info.entity));
                    server.send_message(
                        *client_id,
                        ServerChannel::ServerMessages as u8,
                        error_continue!(
                            ServerMessage::SpawnPlayer {
                                name: info.name.clone(),
                                id: *other_id,
                                translation: trans.translation,
                                weapons: pl
                                    .weapons
                                    .iter()
                                    .map(|v| v.iter().map(|w| w.data.id.clone()).collect())
                                    .collect()
                            }
                            .bytes()
                        ),
                    );
                }

                let spawn_point = nw.player_spawn.0;
                let entity =
                    Player::spawn(&mut nw, false, spawn_point, *client_id, Vec::new(), None);
                let name = FastStr::from(
                    steam
                        .as_ref()
                        .map(|s| s.friends().get_friend(SteamId::from_raw(*client_id)))
                        .map(|f| f.name())
                        .unwrap_or(format!("{client_id}")),
                );
                nw.lobby
                    .insert(*client_id, PlayerInfo::new(entity, name.clone()));

                server.broadcast_message(
                    ServerChannel::ServerMessages as u8,
                    error_continue!(
                        ServerMessage::SpawnPlayer {
                            id: *client_id,
                            translation: spawn_point,
                            weapons: Vec::new(),
                            name
                        }
                        .bytes()
                    ),
                )
            }
            ServerEvent::ClientDisconnected { client_id, reason } => {
                if let Some(player_info) = nw.lobby.remove(client_id) {
                    connections.send(Connections::Leave(*client_id, format!("{reason}")));
                    nw.commands.entity(player_info.entity).despawn_recursive();
                }

                server.broadcast_message(
                    ServerChannel::ServerMessages as u8,
                    error_continue!(ServerMessage::DespawnPlayer { id: *client_id }.bytes()),
                )
            }
        }
    }

    for message in sim_events.read() {
        match message {
            SimulationEvent::PlayerPicksUpPickup { id, player, pickup } => {
                let remove_message = ServerMessage::DespawnPickup { id: *id };
                server.broadcast_message(
                    ServerChannel::NetworkedEntities as u8,
                    error_continue!(remove_message.bytes()),
                );
                let pickup_message = ClientMessage::PickupWeapon {
                    weapon: pickup.clone(),
                };

                update_world(*player, &pickup_message, &mut nw);

                let pickup_message_wrapped = ServerMessage::PlayerUpdate {
                    id: *player,
                    message: pickup_message,
                };

                let bytes = error_continue!(pickup_message_wrapped.bytes());

                server.broadcast_message(ServerChannel::NetworkedEntities as u8, bytes);
            }
        }
    }
}

pub fn client_events(
    mut server: ResMut<RenetServer>,
    mut nw: NetWorld,
    mut server_events: EventWriter<ServerMessage>,
    mut connections: EventReader<Connections>,
    mut first_time: ResMut<MapFirstRun>,
) {
    set_nw!(&nw, &server, &server_events);
    if first_time.0 {
        first_time.0 = false;
        nw.plugins
            .default
            .map_init()
            .expect("failed running `map_init`");
    }
    for message in connections.read() {
        match message {
            Connections::Join(id) => error_continue!(nw.plugins.default.map_player_join(*id)),
            Connections::Leave(id, reason) => {
                error_continue!(nw.plugins.default.map_player_leave(PlayerLeave {
                    id: *id,
                    reason: reason.clone()
                }));
            }
        }
    }

    for client_id in server.clients_id() {
        while let Some(message) = server.receive_message(client_id, ClientChannel::Input as u8) {
            let message = error_continue!(ClientMessage::from_bytes(&message));
            handle_client_message(&mut server, client_id, message, &mut nw, &mut server_events);
        }

        while let Some(message) = server.receive_message(client_id, ClientChannel::Command as u8) {
            let message = error_continue!(ClientMessage::from_bytes(&message));
            handle_client_message(&mut server, client_id, message, &mut nw, &mut server_events);
        }
    }
}

#[allow(mutable_transmutes)]
pub fn handle_client_message(
    server: &mut RenetServer,
    client_id: u64,
    message: ClientMessage,
    nw: &mut NetWorld,
    server_events: &mut EventWriter<ServerMessage>,
) {
    let rapier_context = nw.rapier_context.single();
    match message {
        ClientMessage::Interact => {
            let player = option_return!(nw.lobby.get(&client_id)).entity;
            let (player_entity, mut player, trans) = error_return!(nw.players.get_mut(player));

            let cam = option_return!(player.children.camera);
            let (_, cam_trans) = error_return!(nw.cameras.get(cam));

            let (int, _) =
                option_return!(player.interact(player_entity, rapier_context, cam_trans, &trans));
            let (_e, int) = option_return!(nw.interactables.get(int).ok());
            set_nw!(nw, server, server_events);
            error_return!(nw.plugins.default.map_interact(MapInteraction {
                script: int.script.to_string(),
                target: int.target.as_ref().map(|s| s.to_string()),
                argument: int.argument.as_ref().map(|s| s.to_string()),
                player_id: client_id
            }));
        }
        ClientMessage::Fire { attack } => {
            let mut hit_pos = Vec::new();
            let mut hit_ents = Vec::new();

            let player = option_return!(nw.lobby.get(&client_id)).entity;
            let (player_entity, mut player, trans) = error_return!(nw.players.get_mut(player));

            let cam = option_return!(player.children.camera);
            let (_, cam_trans) = error_return!(nw.cameras.get(cam));

            let (slot, row) = option_return!(player.current_weapon);
            let attack_weapon = Some(player.weapons[slot][row].data.id.clone());
            let hits = player.attack(
                attack,
                &mut nw.materials,
                player_entity,
                &mut nw.commands,
                rapier_context,
                cam_trans,
                &trans,
                &mut nw.game_entropy,
                &nw.projectile_map,
                &nw.asset_server,
            );
            for (hit, pos) in hits {
                hit_pos.push(pos);
                hit_ents.push(hit);
            }

            let attack_weapon = error_return!(
                attack_weapon
                    .ok_or_else(|| format!("player {} attacked without holding weapon", client_id))
            );
            let attack_weapon = error_return!(
                nw.weapon_map
                    .0
                    .get(&attack_weapon)
                    .ok_or_else(|| format!("failed to find weapon {attack_weapon}"))
            );
            for ent in hit_ents {
                if let Ok((_, mut hit_player, _)) = nw.players.get_mut(ent) {
                    hit_player.last_hurter = client_id;
                    let damage = if attack == 1 {
                        if let Attack::RayCast {
                            damage, damage_mod, ..
                        } = &attack_weapon.attack1
                        {
                            damage + (damage_mod * (nw.game_entropy.get_f32() * 2.0 - 1.0))
                        } else {
                            error!("weird attack 1");
                            0.0
                        }
                    } else if let Attack::RayCast {
                        damage, damage_mod, ..
                    } = &attack_weapon.attack2
                    {
                        damage + (damage_mod * (nw.game_entropy.get_f32() * 2.0 - 1.0))
                    } else {
                        error!("weird attack 2");
                        0.0
                    };
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

            let msg = ServerMessage::HitscanHits { hits: hit_pos };
            server_events.send(msg.clone());
            // hitscan_hit_gfx(&nw.asset_server, &mut nw.commands, &hit_pos, &nw.particles);
            server.broadcast_message(
                ServerChannel::NetworkedEntities as u8,
                error_return!(msg.bytes()),
            );
        }
        ClientMessage::RequestLobbyInfo => {
            set_nw!(nw, server, server_events);
            let msg = ServerMessage::LobbyInfo(
                error_return!(nw.plugins.default.map_get_lobby_info()).into(),
            );
            if client_id == nw.current_id.0 {
                server_events.send(msg);
            } else {
                let bytes = error_return!(msg.bytes());
                server.send_message(client_id, ServerChannel::ServerMessages as u8, bytes);
            }
        }
        ClientMessage::RequestRespawn => {
            set_nw!(nw, server, server_events);
            error_return!(nw.plugins.default.map_player_respawn(PlayerKilled {
                player_id: client_id,
                by_id: None
            }));
        }
        message => {
            update_world(client_id, &message, nw);
            server.broadcast_message(
                ServerChannel::NetworkedEntities as u8,
                error_return!(
                    ServerMessage::PlayerUpdate {
                        id: client_id,
                        message,
                    }
                    .bytes()
                ),
            )
        }
    }
}

pub fn init_server(
    world: &mut World,
    next_state: &mut NextState<NetState>,
    steam_client: &Option<Res<SteamClient>>,
) -> bool {
    let server = RenetServer::new(connection_config());

    if let Some(sc) = steam_client {
        let steam_transport_config = SteamServerConfig {
            max_clients: 64,
            access_permission: AccessPermission::Public,
        };

        let transport = error_return!(SteamServerTransport::new(sc, steam_transport_config));

        world.insert_resource(IsSteam);
        world.insert_non_send_resource(transport);
        world.insert_resource(CurrentClientId(sc.user().steam_id().raw()))
    } else {
        let current_time = error_return!(SystemTime::now().duration_since(SystemTime::UNIX_EPOCH));
        let public_addr = error_return!("0.0.0.0:8000".parse());
        let socket = error_return!(UdpSocket::bind(public_addr));

        let server_config = ServerConfig {
            current_time,
            max_clients: 64,
            protocol_id: PROTOCOL_ID,
            public_addresses: vec![public_addr],
            authentication: ServerAuthentication::Unsecure,
        };

        let transport = error_return!(NetcodeServerTransport::new(server_config, socket));

        world.insert_resource(transport);
        world.insert_resource(CurrentClientId(current_time.as_millis() as u64));
    }
    world.insert_resource(server);
    world.insert_resource(Lobby::default());
    next_state.set(NetState::Server);
    info!("started server...");
    true
}

fn clean_up(commands: &mut World) {
    commands.remove_resource::<NetcodeServerTransport>();
    commands.remove_non_send_resource::<SteamServerTransport>();
    commands.remove_resource::<RenetServer>();
    commands.remove_resource::<Lobby>();
}

pub fn system_cleanup() -> SystemConfigs {
    (clean_up,).into_configs()
}

pub fn systems() -> SystemConfigs {
    (server_events, client_events, frag_checker).into_configs()
}

pub fn errors() -> SystemConfigs {
    (error_on_error_system,)
        .into_configs()
        .run_if(resource_exists::<NetcodeServerTransport>)
}

pub fn errors_steam() -> SystemConfigs {
    (error_on_error_system_steam,)
        .into_configs()
        .run_if(resource_exists::<IsSteam>)
}

pub fn error_on_error_system_steam(mut renet_error: EventReader<SteamTransportError>) {
    #[allow(clippy::never_loop)]
    for e in renet_error.read() {
        error!("{}", e);
    }
}

pub fn error_on_error_system(mut renet_error: EventReader<NetcodeTransportError>) {
    #[allow(clippy::never_loop)]
    for e in renet_error.read() {
        error!("{}", e);
    }
}
