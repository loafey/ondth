use bevy::prelude::Component;
use faststr::FastStr;

#[derive(Debug, Component, Clone)]
pub struct Interactable {
    pub script: FastStr,
    pub target: Option<FastStr>,
    pub argument: Option<FastStr>,
}
