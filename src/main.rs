#![doc = include_str!("../readme.md")]
#![feature(let_chains)]
extern crate macros;
use crate::net::{
    SimulationEvent,
    steam::{SteamClient, try_steam},
};
use bevy::{
    core_pipeline::experimental::taa::TemporalAntiAliasPlugin, image::ImageAddressMode,
    log::LogPlugin, prelude::*,
};
use bevy_hanabi::HanabiPlugin;
use bevy_obj::ObjPlugin;
use bevy_rapier3d::{
    plugin::{NoUserData, RapierPhysicsPlugin},
    render::RapierDebugRenderPlugin,
};
use bevy_renet::{
    RenetClientPlugin, RenetServerPlugin,
    netcode::{NetcodeClientPlugin, NetcodeServerPlugin},
    steam::{SteamClientPlugin, SteamServerPlugin},
};
use bevy_scene_hook::reload::Plugin as HookPlugin;
use bevy_simple_text_input::TextInputPlugin;
use net::{ClientMessage, Connections, ServerMessage};
use plugins::{ClientPlugin, GameStage, MainMenuStage, Resources, ServerPlugin, StartupStage};
use steamworks::{AppId, SingleClient};

mod entities;
mod mainmenu;
mod map_gen;
mod net;
mod particles;
mod player;
mod plugins;
mod queries;
mod qwak_host_functions;
mod startup;
mod ui;

const APP_ID: AppId = AppId(480);

fn steam_callbacks(client: NonSend<SingleClient>) {
    client.run_callbacks();
}

fn main() {
    info!("running with asset hash: {}", integrity::get_asset_hash());

    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set({
                let mut plug = ImagePlugin::default_nearest();
                plug.default_sampler.address_mode_u = ImageAddressMode::Repeat;
                plug.default_sampler.address_mode_v = ImageAddressMode::Repeat;
                plug.default_sampler.address_mode_w = ImageAddressMode::Repeat;
                plug
            })
            .set(LogPlugin {
                filter: "cranelift_codegen=warn,bevy_ecs=error,wgpu=error,naga=warn,present_frames=warn,cosmic_text=warn,bevy_render=warn,offset_allocator=warn,extism_plugin=warn,extism=warn,winit=warn,bevy_winit=warn,bevy_hanabi=warn,bevy_app=warn,wasmtime=warn,bevy_asset=warn,gilrs=warn,bevy_hierarchy=warn"
                    .into(),
                // filter: "offset_allocator=warn,extism_plugin=warn,extism=warn,winit=warn,bevy_winit=warn,bevy_hanabi=warn,bevy_app=warn,wasmtime=warn,bevy_asset=warn,gilrs=warn,bevy_hierarchy=warn"
                    // .into(),
                level: bevy::log::Level::DEBUG,
                ..default()
            }), // .set(WindowPlugin {
                //     primary_window: Some(Window {
                //         mode: bevy::window::WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
                //         ..default()
                //     }),
                //     ..default()
                // }),
    );

    // app.add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::new());

    app.add_event::<ClientMessage>()
        .add_event::<SimulationEvent>()
        .add_event::<ServerMessage>()
        .add_event::<Connections>();

    app.add_plugins(Resources);
    app.add_plugins(RapierPhysicsPlugin::<NoUserData>::default());
    app.add_plugins(RapierDebugRenderPlugin::default().disabled());
    app.add_plugins(TemporalAntiAliasPlugin);
    app.add_plugins(ObjPlugin);
    app.add_plugins(HookPlugin);
    app.add_plugins((StartupStage, MainMenuStage, GameStage));
    app.add_plugins(HanabiPlugin);
    app.add_plugins(TextInputPlugin);
    // MP
    app.add_plugins(RenetClientPlugin);
    app.add_plugins(RenetServerPlugin);
    app.add_plugins(ServerPlugin);
    app.add_plugins(ClientPlugin);

    app.add_systems(Startup, particles::register_particles);
    app.add_systems(Update, particles::ParticleLifetime::update);
    app.add_systems(Update, ui::ui_systems());

    if let Some((steam, single_client)) = try_steam() {
        app.insert_non_send_resource(single_client);
        app.insert_resource(SteamClient::new(steam));
        app.add_plugins((SteamServerPlugin, SteamClientPlugin));
        app.add_systems(PreUpdate, steam_callbacks);
        app.add_systems(Startup, net::steam::grab_avatar);
    } else {
        app.add_plugins((NetcodeServerPlugin, NetcodeClientPlugin));
    }

    app.run();
}
