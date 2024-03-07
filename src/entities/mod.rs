use crate::{map_gen::SCALE_FIX, PickupMap, PlayerSpawnpoint};
use bevy::{
    ecs::system::{Commands, ResMut},
    log::error,
    math::Vec3,
    pbr::{PointLight, PointLightBundle},
    transform::components::Transform,
};
use std::collections::HashMap;

use self::data::PickupData;

pub mod data;

fn parse_vec(str: &str) -> Vec3 {
    let mut splat = str.split_whitespace();
    let x = splat
        .next()
        .unwrap_or_default()
        .parse::<f32>()
        .unwrap_or_default();
    let y = splat
        .next()
        .unwrap_or_default()
        .parse::<f32>()
        .unwrap_or_default();
    let z = splat
        .next()
        .unwrap_or_default()
        .parse::<f32>()
        .unwrap_or_default();

    Vec3::new(x, z, -y) / SCALE_FIX
}
pub fn spawn_entity(
    attributes: HashMap<String, String>,
    commands: &mut Commands,
    player_spawn: &mut ResMut<PlayerSpawnpoint>,
    pickup_map: &PickupMap,
) {
    match attributes.get("classname").as_ref().map(|s| &s[..]) {
        Some("light") => {
            let light_level = attributes
                .get("light")
                .and_then(|l| l.parse::<f32>().ok())
                .unwrap_or(150.0);

            let pos = attributes
                .get("origin")
                .map(|p| parse_vec(p))
                .unwrap_or_default();

            commands.spawn(PointLightBundle {
                transform: Transform::from_translation(pos),
                point_light: PointLight {
                    intensity: light_level * 100.0,
                    range: light_level * 100.0,
                    shadows_enabled: false,
                    ..Default::default()
                },
                ..Default::default()
            });
        }
        Some("info_player_start") => {
            let mut pos = attributes
                .get("origin")
                .map(|p| parse_vec(p))
                .unwrap_or_default();

            pos.y += 0.5;

            player_spawn.0 = pos;
        }
        Some(x) if pickup_map.0.contains_key(x) => {
            let data = pickup_map.0.get(x).unwrap();
            spawn_pickup(data, attributes, commands);
        }
        _ => error!("unhandled entity: {attributes:?}"),
    }
}

fn spawn_pickup(data: &PickupData, attributes: HashMap<String, String>, commands: &mut Commands) {
    let pos = attributes
        .get("origin")
        .map(|p| parse_vec(p))
        .unwrap_or_default();
}
