use bevy::prelude::{Component, Entity};
use faststr::FastStr;

#[derive(Debug, Component, Clone)]
pub struct Interactable {
    pub script: FastStr,
    pub target: Option<FastStr>,
}
