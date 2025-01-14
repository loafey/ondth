use bevy::{ecs::schedule::SystemConfigs, prelude::IntoSystemConfigs};

pub mod menu_button;

pub fn ui_systems() -> SystemConfigs {
    (menu_button::update,).into_configs()
}
