use super::{
    ARMOR_GLYPH, GameButtonEvents, HEALTH_GLYPH, Player, PlayerController, PlayerFpsMaterial,
    PlayerFpsModel, PlayerMpModel,
};
use crate::{
    map_gen::GameObject,
    net::{
        PlayerInfo,
        steam::{CurrentAvatar, SteamClient},
    },
    queries::NetWorld,
    ui::menu_button::MenuButton,
};
use bevy::{
    pbr::NotShadowCaster,
    prelude::*,
    render::view::{NoFrustumCulling, RenderLayers},
    window::{CursorGrabMode, PrimaryWindow},
};
use bevy_rapier3d::prelude::*;
use bevy_scene_hook::reload::{Hook, SceneBundle as HookedSceneBundle};
use faststr::FastStr;
use resources::{Paused, PlayerSpawned, PlayerSpawnpoint};

impl Player {
    pub fn spawn_own_player(
        mut nw: NetWorld,
        player_spawn: Res<PlayerSpawnpoint>,
        avatar: Option<Res<CurrentAvatar>>,
        steam: Option<Res<SteamClient>>,
        mut q_windows: Query<&mut Window, With<PrimaryWindow>>,
        paused: ResMut<Paused>,
        mut player_spawned: ResMut<PlayerSpawned>,
    ) {
        let mut primary_window = q_windows.single_mut();
        if paused.0 {
            //rapier_context.
            primary_window.cursor_options.grab_mode = CursorGrabMode::None;
            primary_window.cursor_options.visible = true;
            //time.pause();
        } else {
            primary_window.cursor_options.grab_mode = CursorGrabMode::Locked;
            primary_window.cursor_options.visible = false;
            //time.unpause();
        }

        let id = nw.current_id.0;
        let entity = Self::spawn(
            &mut nw,
            true,
            player_spawn.0,
            id,
            Vec::new(),
            avatar.as_ref(),
        );

        nw.lobby.insert(
            nw.current_id.0,
            PlayerInfo::new(
                entity,
                FastStr::from(steam.map(|s| s.friends().name()).unwrap_or(format!("{id}"))),
            ),
        );

        player_spawned.0 = true
    }

    pub fn spawn(
        nw: &mut NetWorld,
        is_own: bool,
        player_spawn: Vec3,
        current_id: u64,
        weapons: Vec<Vec<FastStr>>,
        avatar: Option<&Res<CurrentAvatar>>,
    ) -> Entity {
        let mut camera = None;
        let mut fps_model = None;
        let mut ammo_hud = None;
        let mut armour_hud = None;
        let mut health_hud = None;
        let mut debug_hud = None;
        let mut message_holder = None;
        let mut hurt_flash = None;
        let mut shoot_sound_holder = None;
        let mut lobby_hud = None;
        let mut entity = nw.commands.spawn(Collider::cylinder(0.5, 0.15));
        let mut pause_screen = None;
        let mut death_splash = None;

        let player_commands = entity
            .insert(ActiveEvents::COLLISION_EVENTS)
            .insert(GameObject)
            .insert(Name::new("player"))
            .insert(Transform::from_translation(player_spawn))
            .insert(match is_own {
                true => RigidBody::Dynamic,
                false => RigidBody::Fixed,
            })
            .insert(Velocity::zero())
            .insert(GravityScale(0.0))
            .insert(Friction::new(0.0))
            .insert(Restitution::coefficient(0.0))
            .insert(LockedAxes::ROTATION_LOCKED)
            .insert(GlobalTransform::default())
            .insert(Ccd::enabled())
            .insert(InheritedVisibility::VISIBLE)
            .with_children(|c| {
                let new_camera_id = c
                    .spawn((
                        Camera3d::default(),
                        Projection::Perspective(PerspectiveProjection {
                            fov: 80.0f32.to_radians(),
                            ..default()
                        }),
                        Transform::from_translation(Vec3::new(0.0, 0.25, 0.0)),
                        Camera {
                            is_active: is_own,
                            order: 1,
                            ..default()
                        },
                        RenderLayers::layer(1),
                    ))
                    // .insert(ScreenSpaceAmbientOcclusion::default())
                    // .insert(Msaa::Off)
                    // .insert((DepthPrepass, MotionVectorPrepass, TemporalJitter::default()))
                    // .insert(TemporalAntiAliasing::default())
                    .insert(Name::new("player camera"))
                    .with_children(|c| {
                        let new_fps_model = c
                            .spawn(PlayerFpsModel)
                            .insert(HookedSceneBundle {
                                scene: SceneRoot::default(),
                                reload: Hook::new(move |entity, commands, world, root| {
                                    if entity.get::<Mesh3d>().is_some() {
                                        let cc = commands
                                            .insert(NoFrustumCulling)
                                            .insert(NotShadowCaster);
                                        if is_own {
                                            cc.insert(RenderLayers::layer(1));
                                        }
                                    }
                                    if entity.get::<MeshMaterial3d<StandardMaterial>>().is_some() {
                                        if let Some(material) =
                                            world.entity(root).get::<PlayerFpsMaterial>()
                                        {
                                            commands.insert(MeshMaterial3d(material.0.clone()));
                                        }
                                    }
                                }),
                            })
                            .insert(PlayerFpsMaterial::default())
                            .insert(Name::new("fps model holder"))
                            .id();
                        fps_model = Some(new_fps_model);

                        if is_own {
                            c.spawn((Transform::IDENTITY, SpatialListener::new(2.0)));
                        }

                        c.spawn((
                            Camera3d::default(),
                            Projection::Perspective(PerspectiveProjection {
                                fov: 80.0f32.to_radians(),
                                ..default()
                            }),
                            Transform::default(),
                            Camera {
                                is_active: is_own,
                                ..default()
                            },
                            RenderLayers::layer(0),
                            Msaa::Off,
                        ));

                        shoot_sound_holder = Some(c.spawn(Transform::IDENTITY).id());
                    })
                    .id();

                camera = Some(new_camera_id);
                if is_own {
                    c.spawn((Camera2d, Camera {
                        order: 2,
                        clear_color: ClearColorConfig::None,
                        is_active: is_own,
                        ..default()
                    }))
                    .insert(IsDefaultUiCamera);

                    c.spawn(Sprite {
                        image: nw.asset_server.load("crosshair.png"),
                        ..default()
                    });
                }
            });

        if is_own {
            player_commands.insert(PlayerController);
        } else {
            player_commands.with_children(|c| {
                let mut trans = Transform::from_translation(Vec3::new(0.0, -0.5, 0.0));
                trans.scale = Vec3::splat(0.5);
                trans.rotate_y(180f32.to_radians());
                c.spawn((
                    Mesh3d(nw.asset_server.load("models/Player/MP/Temp.obj")),
                    MeshMaterial3d(nw.materials.add(StandardMaterial {
                        base_color_texture: Some(
                            nw.asset_server.load("models/Enemies/DeadMan/deadman.png"),
                        ),
                        perceptual_roughness: 1.0,
                        reflectance: 0.0,
                        ..default()
                    })),
                    trans,
                ))
                .insert(Name::new("player mp model"))
                .insert(PlayerMpModel);
            });
        }
        let id = player_commands.id();
        if is_own {
            nw.commands
                .spawn(Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                })
                .with_children(|c| {
                    hurt_flash = Some(
                        c.spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                width: Val::Vw(100.0),
                                height: Val::Vh(100.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(1.0, 0.0, 0.0, 0.0)),
                        ))
                        .id(),
                    );

                    c.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            width: Val::Px(128.0 * 3.0),
                            height: Val::Px(32.0 * 3.0),
                            left: Val::Px(0.0),
                            bottom: Val::Px(0.0),
                            // a `NodeBundle` is transparent by default, so to see the image we have to its color to `WHITE`
                            // background_color: Color::WHITE.into(),
                            ..default()
                        },
                        ImageNode {
                            image: nw.asset_server.load("ui/PlayerHud.png"),
                            ..default()
                        },
                    ));

                    message_holder = Some(
                        c.spawn(Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(10.0),
                            top: Val::Px(10.0),
                            flex_direction: FlexDirection::Column,
                            ..default()
                        })
                        .id(),
                    );

                    c.spawn(Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(0.0),
                        top: Val::Px(0.0),
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        align_content: AlignContent::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    })
                    .with_children(|c| {
                        lobby_hud = Some(
                            c.spawn((
                                Text::new("LOBBY DATA"),
                                TextFont {
                                    font: nw.asset_server.load("ui/Color Basic.otf"),
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ))
                            .insert(Visibility::Hidden)
                            .id(),
                        );
                    });

                    let text_color = Color::srgb(0.921, 0.682, 0.203);

                    c.spawn(Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(34.0 * 3.0),
                        bottom: Val::Px(18.0 * 2.0),
                        ..default()
                    })
                    .with_children(|c| {
                        health_hud = Some(
                            c.spawn((
                                Text::new(format!("{HEALTH_GLYPH}100")),
                                TextFont {
                                    font: nw.asset_server.load("ui/Color Basic.otf"),
                                    ..default()
                                },
                                TextColor(text_color),
                            ))
                            .id(),
                        );
                    });

                    c.spawn(Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(34.0 * 3.0),
                        bottom: Val::Px(4.0),
                        ..default()
                    })
                    .with_children(|c| {
                        armour_hud = Some(
                            c.spawn((
                                Text::new(format!("{ARMOR_GLYPH}100")),
                                TextFont {
                                    font: nw.asset_server.load("ui/Color Basic.otf"),
                                    ..default()
                                },
                                TextColor(text_color),
                            ))
                            .id(),
                        );
                    });

                    c.spawn(Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(255.0),
                        bottom: Val::Px(10.0),
                        ..default()
                    })
                    .with_children(|c| {
                        ammo_hud = Some(
                            c.spawn((
                                Text::new("100\nCRUTONS"),
                                TextFont {
                                    font: nw.asset_server.load("ui/Color Basic.otf"),
                                    ..default()
                                },
                                TextColor(text_color),
                                TextLayout::new_with_justify(JustifyText::Center),
                            ))
                            .id(),
                        );
                    });

                    c.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            width: Val::Px(26.0 * 3.0),
                            height: Val::Px(28.0 * 3.0),
                            left: Val::Px(2.0 * 3.0),
                            bottom: Val::Px(2.0 * 3.0),
                            // a `NodeBundle` is transparent by default, so to see the image we have to its color to `WHITE`
                            //background_color: Color::WHITE.into(),
                            ..default()
                        },
                        ImageNode {
                            image: avatar
                                .map(|c| c.0.clone())
                                .unwrap_or_else(|| nw.asset_server.load("ui/PlayerIcon.png")),
                            ..default()
                        },
                    ));

                    c.spawn(Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(0.0),
                        top: Val::Px(0.0),
                        ..default()
                    })
                    .with_children(|c| {
                        debug_hud = Some(
                            c.spawn((Visibility::Hidden, Text::new("debug"), TextFont {
                                font_size: 16.0,
                                ..default()
                            }))
                            .id(),
                        );
                    });

                    death_splash = Some(
                        c.spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                width: Val::Vw(100.0),
                                height: Val::Vh(100.0),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgba(1.0, 0.0, 0.0, 0.4)),
                        ))
                        .insert(Visibility::Hidden)
                        .with_children(|c| {
                            c.spawn(Node {
                                padding: UiRect::all(Val::Px(10.0)),
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                ..default()
                            })
                            .insert(BackgroundColor(Color::BLACK))
                            .with_children(|c| {
                                c.spawn(Text("- You are dead -".to_string()));
                                c.spawn(MenuButton::new(
                                    "Respawn",
                                    None,
                                    None,
                                    None,
                                    GameButtonEvents::Respawn,
                                ));
                            });
                        })
                        .id(),
                    );
                })
                .insert(Name::new("player gui holder"))
                .insert(GameObject);

            pause_screen = Some(
                nw.commands
                    .spawn(Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(10.0),
                        ..default()
                    })
                    .insert(Name::new("pause screen"))
                    .insert(GameObject)
                    .insert(Visibility::Hidden)
                    .insert(BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)))
                    .with_children(|c| {
                        c.spawn(Node {
                            width: Val::Px(230.0),
                            align_items: AlignItems::Stretch,
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(10.0),
                            padding: UiRect {
                                left: Val::ZERO,
                                right: Val::ZERO,
                                top: Val::Px(10.0),
                                bottom: Val::ZERO,
                            },
                            ..default()
                        })
                        .insert(Name::new("pause gui"))
                        .insert(BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 1.0)))
                        .with_children(|c| {
                            c.spawn(Node {
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                width: Val::Percent(100.0),
                                ..default()
                            })
                            .with_child(Text("- PAUSED -".to_string()));
                            c.spawn(MenuButton::new(
                                "Options",
                                None,
                                None,
                                None,
                                GameButtonEvents::Options,
                            ));
                            c.spawn(MenuButton::new(
                                "Leave",
                                None,
                                None,
                                None,
                                GameButtonEvents::Leave,
                            ));
                        });
                    })
                    .id(),
            );
        }

        let mut player_commands = nw.commands.get_entity(id).unwrap();

        let mut player_data = Player {
            id: current_id,
            children: super::PlayerChildren {
                camera,
                fps_model,
                ammo_hud,
                armour_hud,
                health_hud,
                debug_hud,
                message_holder,
                shoot_sound_holder,
                lobby_hud,
                hurt_flash,
                pause_screen,
                death_splash,
            },
            ..default()
        };

        for (slot, list) in weapons.into_iter().enumerate() {
            for weapon in list {
                if let Some(weapon_data) = nw.weapon_map.0.get(&weapon) {
                    let handle = nw
                        .asset_server
                        .load(format!("{}#Scene0", weapon_data.model_file));
                    player_data.add_weapon(weapon_data.clone(), slot, handle);
                }
            }
        }
        player_commands.insert(player_data);
        id
    }
}
