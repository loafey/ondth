//! Contains multiple types which help are used when calling `qwak` plugin
//! methods.
use extism_pdk::{FromBytes, Msgpack, ToBytes};
use serde::{Deserialize, Serialize};

/// The argument to [`map_interact`](../qwak_shared/trait.QwakPlugin.html#tymethod.map_interact).
#[derive(Debug, Clone, FromBytes, ToBytes, Deserialize, Serialize)]
#[encoding(Msgpack)]
pub struct MapInteraction {
    /// The script to run
    pub script: String,
    /// The optional target entity
    pub target: Option<String>,
    /// The id of the player activating the interaction
    pub player_id: u64,
}
