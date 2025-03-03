use crate::{
    entities::{ProjectileEntity, message::Message, pickup::PickupEntity},
    mainmenu,
    map_gen::{clean_up_map, load_map, texture_systems::*, world_entites},
    net::{self, NetState},
    player::Player,
    qwak_host_functions::qwak_functions,
    startup,
};
use bevy::prelude::*;
use qwak::*;
use resources::{
    entropy::{entropy_game, entropy_misc},
    inputs::PlayerInput,
    *,
};
use std::{collections::HashMap, path::PathBuf};

#[derive(Debug, Resource)]
pub struct Qwaks {
    pub default: QwakPlugin,
}
impl Qwaks {
    fn new(functions: impl IntoIterator<Item = qwak::Function>) -> Self {
        info!("Loading qwaks...");
        let default = match QwakPlugin::new("assets/qwaks/default.wasm", functions) {
            Ok(o) => o,
            Err(e) => panic!("failed loading default qwak: {e}"),
        };
        default.plugin_init().unwrap();

        info!("Done loading qwaks...");
        Self { default }
    }
}

pub struct Resources;
impl Resources {
    fn get_map() -> PathBuf {
        if let Some(map) = std::env::args().nth(1) {
            if std::fs::File::open(&map).is_ok() {
                return map.into();
            } else {
                error!("Can't find map: \"{map}\"")
            }
        }

        "assets/maps/Test.map".into()
    }
}
impl Plugin for Resources {
    fn build(&self, app: &mut App) {
        let qwaks = Qwaks::new(qwak_functions());
        app.init_state::<CurrentStage>()
            .init_state::<NetState>()
            .insert_resource(CurrentMap(Self::get_map()))
            .insert_resource(TextureLoadingState::NotLoaded)
            .insert_resource(PlayerSpawned(false))
            .insert_resource(TexturesLoading::default())
            .insert_resource(TextureMap::default())
            .insert_resource(PlayerSpawnpoint(Vec3::ZERO))
            .insert_resource(MapDoneLoading(false))
            .insert_resource(Paused(false))
            .insert_resource(MapFirstRun(true))
            .insert_resource(PickupMap(qwaks.default.plugin_get_pickups().unwrap()))
            .insert_resource(WeaponMap(qwaks.default.plugin_get_weapons().unwrap()))
            .insert_resource(PlayerInput::default())
            .insert_resource(entropy_game())
            .insert_resource(entropy_misc())
            .insert_resource(Projectiles(qwaks.default.plugin_get_projectiles().unwrap()))
            .insert_resource(TargetMap(HashMap::default()))
            .insert_resource(qwaks);
    }
}

pub struct ServerPlugin;
impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            (
                net::server::systems(),
                net::server::errors(),
                net::server::errors_steam(),
                net::client::all_cons(),
            )
                .run_if(in_state(NetState::Server)),
        )
        .add_systems(OnExit(CurrentStage::InGame), net::server::system_cleanup());
    }
}

pub struct ClientPlugin;
impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            (
                net::client::systems(),
                net::client::errors(),
                net::client::errors_steam(),
                net::client::all_cons(),
            )
                .run_if(in_state(NetState::Client)),
        )
        .add_systems(
            PreUpdate,
            net::send_messages.run_if(in_state(NetState::Server).or(in_state(NetState::Client))),
        )
        .add_systems(OnExit(CurrentStage::InGame), net::client::system_cleanup());
    }
}

pub struct StartupStage;
impl Plugin for StartupStage {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            startup::startup_setup.run_if(in_state(CurrentStage::Startup)),
        )
        .add_systems(
            Update,
            startup::startup_update.run_if(in_state(CurrentStage::Startup)),
        );
    }
}

pub struct MainMenuStage;
impl Plugin for MainMenuStage {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(CurrentStage::MainMenu),
            mainmenu::setup.run_if(in_state(CurrentStage::MainMenu)),
        )
        .add_systems(
            Update,
            (mainmenu::update_level_buttons, mainmenu::update_id_buttons)
                .run_if(in_state(CurrentStage::MainMenu)),
        )
        .add_systems(
            Update,
            mainmenu::buttons.run_if(in_state(CurrentStage::MainMenu)),
        )
        .add_systems(
            Update,
            mainmenu::update_point_light.run_if(in_state(CurrentStage::MainMenu)),
        )
        .add_systems(OnExit(CurrentStage::MainMenu), mainmenu::clear);
    }
}

pub struct GameStage;
impl Plugin for GameStage {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(OnEnter(CurrentStage::InGame), register_textures)
            .add_systems(
                Update,
                world_entites::systems().run_if(in_state(CurrentStage::InGame)),
            )
            .add_systems(
                Update,
                texture_waiter
                    .run_if(in_state(CurrentStage::InGame))
                    .run_if(if_texture_loading),
            )
            .add_systems(
                Update,
                load_map
                    .run_if(in_state(CurrentStage::InGame))
                    .run_if(if_texture_done_loading)
                    .run_if(not(if_map_done_loading)),
            )
            .add_systems(
                Update,
                Player::spawn_own_player
                    .run_if(in_state(CurrentStage::InGame))
                    .run_if(if_map_done_loading)
                    .run_if(not(if_player_spawned)),
            )
            .add_systems(
                PreUpdate,
                PlayerInput::update.run_if(in_state(CurrentStage::InGame)),
            )
            .add_systems(
                Update,
                (
                    Player::systems(),
                    PickupEntity::systems(),
                    ProjectileEntity::systems(),
                    Message::update_messages,
                )
                    .run_if(in_state(CurrentStage::InGame)), //.run_if(if_not_paused),
            )
            .add_systems(Update, Player::input_systems().run_if(if_not_paused))
            .add_systems(
                Update,
                (Player::pause_handler, Player::debug).run_if(in_state(CurrentStage::InGame)),
            )
            .add_systems(OnExit(CurrentStage::InGame), clean_up_map);
    }
}
