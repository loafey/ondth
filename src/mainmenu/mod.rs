use crate::{
    APP_ID,
    net::{self, NetState, steam::SteamClient},
    plugins::Qwaks,
    ui::menu_button::MenuButton,
};
use bevy::{ecs::system::SystemState, prelude::*};
use bevy_simple_text_input::{TextInput, TextInputSettings, TextInputTextFont, TextInputValue};
use macros::{error_continue, error_return};
use resources::{CurrentMap, CurrentStage};
use std::{
    fs, io,
    path::{Path, PathBuf},
};
use steamworks::FriendFlags;

#[derive(Debug, Component)]
pub struct MainMenuEnt;

#[derive(Debug, Resource)]
struct MainMenuState {
    main: Entity,
    join: Entity,
    host: Entity,
}

#[derive(Debug, Component)]
pub enum ButtonEvent {
    #[allow(unused)]
    Solo,
    HostScreen,
    JoinMp,
    Back,
    IpJoin,
    FriendJoin(u64),
}

#[derive(Debug, Component)]
pub struct LevelButton(PathBuf);

#[derive(Debug, Component)]
pub struct FriendButton(u64);

fn get_mapfiles<P: AsRef<Path>>(dir: P) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    let dir = fs::read_dir(dir)?;
    for f in dir {
        let f = f?.path();

        if f.is_dir() {
            files.append(&mut get_mapfiles(f)?);
        } else {
            files.push(f);
        }
    }

    Ok(files)
}

#[allow(clippy::type_complexity)]
pub fn buttons(world: &mut World) {
    let mut state: SystemState<(
        Query<(&Interaction, &ButtonEvent), (Changed<Interaction>, With<Button>)>,
        Query<&TextInputValue>,
        ResMut<NextState<NetState>>,
        Option<Res<SteamClient>>,
        Res<MainMenuState>,
        Query<&mut Visibility>,
    )> = SystemState::new(world);
    // yea this is cursed, but i am lazy, bypassing the borrow checker like a baus
    #[allow(unsafe_code)]
    let world_copy = unsafe { &mut *(world as *mut World) };

    let (query, text_inputs, mut next_net_state, steam_client, state, mut vis) =
        state.get_mut(world);

    for (interaction, event) in &query {
        if !matches!(interaction, Interaction::Pressed) {
            continue;
        }

        match event {
            ButtonEvent::Solo => {
                error!("solo games are currently disabled");
            }
            ButtonEvent::HostScreen => {
                *error_continue!(vis.get_mut(state.host)) = Visibility::Visible;
                *error_continue!(vis.get_mut(state.main)) = Visibility::Hidden;
                *error_continue!(vis.get_mut(state.join)) = Visibility::Hidden;
            }
            ButtonEvent::JoinMp => {
                *error_continue!(vis.get_mut(state.host)) = Visibility::Hidden;
                *error_continue!(vis.get_mut(state.main)) = Visibility::Hidden;
                *error_continue!(vis.get_mut(state.join)) = Visibility::Visible;
            }
            ButtonEvent::Back => {
                *error_continue!(vis.get_mut(state.host)) = Visibility::Hidden;
                *error_continue!(vis.get_mut(state.main)) = Visibility::Visible;
                *error_continue!(vis.get_mut(state.join)) = Visibility::Hidden;
            }
            ButtonEvent::IpJoin => {
                if steam_client.is_none() {
                    let input = &error_return!(text_inputs.get_single()).0;
                    net::client::init_client(world_copy, &mut next_net_state, input, &steam_client);
                }
            }
            ButtonEvent::FriendJoin(id) => {
                net::client::init_client(
                    world_copy,
                    &mut next_net_state,
                    &format!("{id}"),
                    &steam_client,
                );
            }
        }
    }
}

pub fn clear(query: Query<(Entity, &MainMenuEnt)>, mut commands: Commands) {
    for (ent, _) in &query {
        commands.entity(ent).despawn_recursive();
    }
    commands.insert_resource(AmbientLight::default());
}

#[allow(clippy::type_complexity)]
pub fn update_level_buttons(world: &mut World) {
    #[allow(unsafe_code)]
    // yea this is cursed, but i am lazy, bypassing the borrow checker like a baus
    let world_copy = unsafe { &mut *(world as *mut World) };
    let mut state: SystemState<(
        Query<(&Interaction, &LevelButton), (Changed<Interaction>, With<Button>)>,
        ResMut<NextState<CurrentStage>>,
        ResMut<NextState<NetState>>,
        Option<Res<SteamClient>>,
        ResMut<CurrentMap>,
    )> = SystemState::new(world);
    let (query, mut next_state, mut next_net_state, steam_client, mut cur_level) =
        state.get_mut(world);

    for (interaction, button) in &query {
        if matches!(interaction, Interaction::Pressed) {
            cur_level.0.clone_from(&button.0);
            info!("starting multiplayer game");
            if net::server::init_server(world_copy, &mut next_net_state, &steam_client) {
                next_state.set(CurrentStage::InGame);
            }
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn update_id_buttons(
    query: Query<(&Interaction, &FriendButton), (Changed<Interaction>, With<Button>)>,
    mut text_input: Query<&mut TextInputValue>,
) {
    for (interaction, button) in &query {
        if matches!(interaction, Interaction::Pressed) {
            let mut inp = error_continue!(text_input.get_single_mut());
            inp.0 = format!("{}", button.0);
            info!("set join id to: {:?}", button.0);
        }
    }
}

pub fn update_point_light(mut query: Query<&mut PointLight>) {
    for mut light in query.iter_mut() {
        light.intensity += 0.1;
        light.intensity *= 1.02;
        light.intensity = light.intensity.min(65000.0);
    }
}

pub fn setup(
    mut commands: Commands,
    steam_client: Option<Res<SteamClient>>,

    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    qwaks: Res<Qwaks>,
) {
    commands.insert_resource(AmbientLight {
        brightness: 0.0,
        ..default()
    });

    let map_files = error_return!(get_mapfiles("assets/maps"));
    let friends = steam_client
        .as_ref()
        .map(|sc| sc.friends().get_friends(FriendFlags::ALL))
        .map(|friends| {
            friends
                .into_iter()
                .filter(|f| {
                    f.game_played()
                        .map(|f| f.game.app_id() == APP_ID)
                        .unwrap_or_default()
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    commands
        .spawn((Camera2d, Msaa::Off))
        .insert(Camera {
            order: 2,
            clear_color: ClearColorConfig::None,
            is_active: true,
            ..default()
        })
        .insert(MainMenuEnt);
    let material = materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load("ui/main_menu.png")),
        alpha_mode: AlphaMode::Add,
        ..default()
    });
    // cube
    let scale = 0.5;
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(16.0 * scale, 9.0 * scale, 1.0))),
        MeshMaterial3d(material),
        Transform::from_xyz(0.0, 0.6, 0.0),
        MainMenuEnt,
    ));
    // light
    for x in -1..=1 {
        commands.spawn((
            PointLight {
                color: Color::srgb(1.0, 0.6, 0.6),
                shadows_enabled: false,
                intensity: 1000.0,
                range: 1000.0,
                ..default()
            },
            Transform::from_xyz(x as f32 * 2.0, 0.0, -9.0),
            MainMenuEnt,
        ));
    }

    // camera
    commands.spawn((
        Camera3d::default(),
        Camera {
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, -9.0).looking_at(Vec3::ZERO, Vec3::Y),
        MainMenuEnt,
        Msaa::Off,
    ));

    let mut main = None;
    let mut join = None;
    let mut host = None;

    const FONT_SIZE: Option<f32> = Some(32.0);
    const PADDING: Option<f32> = Some(5.0);
    const BORDER: Option<f32> = Some(10.0);

    #[cfg(not(feature = "production"))]
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            right: Val::Px(-250.0),
            top: Val::Px(-270.0),
            padding: UiRect {
                left: Val::Px(100.0),
                right: Val::Px(100.0),
                top: Val::Px(400.0),
                bottom: Val::Px(10.0),
            },
            ..default()
        })
        .with_child((Text::new("DEVELOPMENT BUILD"), TextColor(Color::WHITE)))
        .insert(Transform::from_rotation(Quat::from_axis_angle(
            Vec3::Z,
            45f32.to_radians(),
        )))
        .insert(BackgroundColor(Color::srgb_u8(255, 0, 0)))
        .insert(MainMenuEnt);

    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        })
        .with_children(|c| {
            main = Some(
                c.spawn(Node {
                    position_type: PositionType::Absolute,
                    width: Val::Px(400.0),
                    border: UiRect::all(Val::Px(2.0)),
                    left: Val::Px(76.0),
                    bottom: Val::Px(76.0),
                    flex_direction: FlexDirection::Column,
                    ..default()
                })
                .insert(Visibility::Visible)
                .with_children(|c| {
                    c.spawn(Node {
                        flex_direction: FlexDirection::Column,
                        ..default()
                    })
                    .with_children(|c| {
                        // c.spawn(MenuButton::new(
                        //     "Solo",
                        //     Some(32.0),
                        //     Some(5.0),
                        //     Some(10.0),
                        //     ButtonEvent::Solo,
                        // ));
                        c.spawn(MenuButton::new(
                            "Host Game",
                            FONT_SIZE,
                            PADDING,
                            BORDER,
                            ButtonEvent::HostScreen,
                        ));
                        c.spawn(MenuButton::new(
                            "Join Game",
                            FONT_SIZE,
                            PADDING,
                            BORDER,
                            ButtonEvent::JoinMp,
                        ));
                    });
                })
                .id(),
            );

            c.spawn(Node {
                position_type: PositionType::Absolute,
                right: Val::Px(0.0),
                bottom: Val::Px(0.0),
                // background_color: Color::rgb(0.65, 0.65, 0.65).into(),
                ..default()
            })
            .insert((
                Text::new(format!(
                    "Running \"{}\" ({})",
                    qwaks.default.plugin_name().unwrap(),
                    {
                        let version = qwaks.default.plugin_version().unwrap();
                        format!("{}.{}.{}", version[0], version[1], version[2])
                    }
                )),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
            ));

            host = Some(
                c.spawn(Node {
                    position_type: PositionType::Absolute,
                    width: Val::Px(400.0),
                    border: UiRect::all(Val::Px(2.0)),
                    left: Val::Px(76.0),
                    bottom: Val::Px(76.0),
                    flex_direction: FlexDirection::Column,
                    ..default()
                })
                .insert(Visibility::Hidden)
                .with_children(|c| {
                    c.spawn(Node {
                        padding: UiRect::all(Val::Px(PADDING.unwrap() * 2.0)),
                        ..default()
                    })
                    .with_child((
                        Text::new("Maps:".to_string()),
                        TextFont {
                            font_size: FONT_SIZE.unwrap(),
                            ..default()
                        },
                    ));

                    for map in map_files {
                        let s = format!("{map:?}");
                        if s.contains("/autosave/") {
                            continue;
                        }
                        c.spawn(MenuButton::new(
                            s[13..s.len() - 5].to_string(),
                            Some(16.0),
                            Some(4.0),
                            Some(2.0),
                            LevelButton(map.clone()),
                        ));
                    }
                    c.spawn(MenuButton::new(
                        "Back",
                        FONT_SIZE,
                        PADDING,
                        BORDER,
                        ButtonEvent::Back,
                    ));
                })
                .id(),
            );
            join = Some(
                c.spawn(Node {
                    position_type: PositionType::Absolute,
                    width: Val::Px(400.0),
                    border: UiRect::all(Val::Px(2.0)),
                    left: Val::Px(76.0),
                    bottom: Val::Px(76.0),
                    flex_direction: FlexDirection::Column,
                    ..default()
                })
                .insert(Visibility::Hidden)
                .with_children(|c| {
                    if steam_client.is_some() {
                        c.spawn(Node {
                            padding: UiRect::all(Val::Px(PADDING.unwrap() * 2.0)),
                            ..default()
                        })
                        .with_child((
                            Text::new("Friends:".to_string()),
                            TextFont {
                                font_size: FONT_SIZE.unwrap(),
                                ..default()
                            },
                        ));
                        for friend in friends {
                            c.spawn(MenuButton::new(
                                friend.name(),
                                Some(16.0),
                                Some(4.0),
                                Some(2.0),
                                ButtonEvent::FriendJoin(friend.id().raw()),
                            ));
                        }
                    } else {
                        c.spawn((
                            Text::new("Enter IP:".to_string()),
                            TextFont {
                                font_size: 32.0,
                                ..default()
                            },
                        ));

                        c.spawn(Node::default()).insert((
                            TextInput,
                            TextInputValue("127.0.0.1:8000".to_string()),
                            TextInputSettings {
                                retain_on_submit: true,
                                ..default()
                            },
                            TextInputTextFont(TextFont {
                                font_size: 32.0,
                                ..default()
                            }),
                        ));
                        c.spawn(MenuButton::new(
                            "Join",
                            FONT_SIZE,
                            PADDING,
                            BORDER,
                            ButtonEvent::IpJoin,
                        ));
                    }
                    c.spawn(MenuButton::new(
                        "Back",
                        FONT_SIZE,
                        PADDING,
                        BORDER,
                        ButtonEvent::Back,
                    ));
                })
                .id(),
            );
        })
        .insert(MainMenuEnt);

    commands.insert_resource(MainMenuState {
        main: main.unwrap(),
        join: join.unwrap(),
        host: host.unwrap(),
    });
}
