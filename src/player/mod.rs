use std::collections::HashMap;

use bevy::prelude::*;
use faststr::FastStr;
use resources::data::WeaponData;

use crate::entities::message::Message;

mod debug;
mod spawn;
mod update;

#[derive(Component, Debug)]
pub struct PlayerFpsModel;

#[derive(Debug, Component)]
pub struct PlayerController;

#[derive(Debug, Component)]
pub struct PlayerMpModel;

#[derive(Debug)]
pub struct CameraMovement {
    backdrift: f32,
    backdrift_goal: f32,
    backdrift_max: f32,
    original_trans: Vec3,

    bob_goal: f32,
    bob_current: f32,

    cam_rot_max_goal: f32,
    cam_rot_goal: f32,
    cam_rot_current: f32,

    switch_offset: f32,
}

#[derive(Debug)]
pub struct WeaponState {
    mesh: Handle<Scene>,
    timer: f32,
    anim_time: f32,
    need_to_reload: bool,
    reload_timer: f32,
    pub data: WeaponData,
}

#[derive(Debug, Default)]
pub struct PlayerChildren {
    pub camera: Option<Entity>,
    pub fps_model: Option<Entity>,
    pub health_hud: Option<Entity>,
    pub armour_hud: Option<Entity>,
    pub ammo_hud: Option<Entity>,
    pub debug_hud: Option<Entity>,
    pub message_holder: Option<Entity>,
    pub shoot_sound_holder: Option<Entity>,
    pub lobby_hud: Option<Entity>,
}

#[derive(Debug, Default)]
pub struct DebugInfo {
    pub on_ground: bool,
    pub head_hit: bool,
    pub velocity: Vec3,
    pub current_speed: f32,
    pub current_falling: f32,
    pub last_airtime: f32,
}

#[derive(Component, Debug, Default)]
pub struct PlayerFpsMaterial(Handle<StandardMaterial>);

#[derive(Component, Debug)]
pub struct Player {
    // stupid hack needed because GLTF scences do not spawn with
    // animation graphs, which *are* needed to play animations.
    // basically; the first thing the animation system tries to do
    // is insert this next to the animation player in the entity tree.
    fps_anim_graph: Option<AnimationGraph>,
    // even stupider hack: when switching weapons the fps model entity
    // is invalidated, and because bevy has no way to check if an
    // insertion failed or not (outside of using the panicing method)
    // we simply have to try inserting the graph two times, because
    // the first one *might* fail
    fps_anim_graph_insert_count: u8,
    fps_anims: HashMap<FastStr, u32>,

    pub id: u64,
    pub last_hurter: u64,

    pub health: f32,
    pub armour: f32,

    velocity: Vec3,
    hort_speed: f32,
    hort_friction: f32,
    jump_height: f32,
    on_ground: bool,
    head_hit: bool,
    gravity: f32,

    camera_movement: CameraMovement,

    pub children: PlayerChildren,

    pub weapons: [Vec<WeaponState>; 10],
    pub current_weapon: Option<(usize, usize)>,
    current_weapon_old: Option<(usize, usize)>,
    pub current_weapon_anim: FastStr,
    current_weapon_anim_old: FastStr,
    pub restart_anim: bool,

    half_height: f32,
    radius: f32,
    air_time: Option<std::time::Instant>,

    pub debug_info: DebugInfo,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            id: 0,
            last_hurter: 0,
            health: 100.0,
            armour: 100.0,
            velocity: Vec3::ZERO,
            hort_friction: 8.0,
            hort_speed: 0.6,
            on_ground: false,
            head_hit: false,
            jump_height: 7.0,
            gravity: -15.0,
            half_height: 0.5,
            radius: 0.15,
            air_time: None,
            current_weapon: None,
            current_weapon_old: None,
            weapons: Default::default(),
            current_weapon_anim: Default::default(),
            current_weapon_anim_old: Default::default(),
            restart_anim: false,
            children: Default::default(),
            fps_anims: Default::default(),
            fps_anim_graph: None,
            fps_anim_graph_insert_count: 0,
            camera_movement: CameraMovement {
                backdrift: 0.0,
                backdrift_goal: 0.0,
                backdrift_max: 0.02,
                original_trans: Vec3::ZERO,
                cam_rot_max_goal: 0.03,
                cam_rot_goal: 0.03,
                cam_rot_current: 0.0,

                bob_current: 0.0,
                bob_goal: 0.0,

                switch_offset: 0.0,
            },
            debug_info: Default::default(),
        }
    }
}
impl Player {
    pub fn add_weapon(&mut self, data: WeaponData, slot: usize, mesh: Handle<Scene>) -> bool {
        if !self.weapons[slot].iter().any(|c| c.data.id == data.id) {
            self.weapons[slot].push(WeaponState {
                need_to_reload: false,
                data,
                mesh,
                reload_timer: 0.0,
                timer: 0.0,
                anim_time: 0.0,
            });

            if self.current_weapon.is_none() {
                self.current_weapon = Some((slot, 0))
            }
            true
        } else {
            error!("unhandled: picked up weapon when already had one");
            false
        }
    }

    pub fn display_message(
        &self,
        commands: &mut Commands,
        asset_server: &AssetServer,
        message: String,
    ) {
        if let Some(holder) = self.children.message_holder {
            let message_id = commands
                .spawn((
                    Text::new(message),
                    TextFont {
                        font: asset_server.load("ui/Color Basic.otf"),
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(Color::srgb(1.0, 0.0, 0.0)),
                ))
                .insert(Message::default())
                .id();
            commands.entity(holder).add_child(message_id);
        } else {
            info!("Got message: {message}")
        }
    }
}

const HEALTH_GLYPH: &str = "+";
const ARMOR_GLYPH: &str = "Δ";
