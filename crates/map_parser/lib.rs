//! A parser for [Map](https://quakewiki.org/wiki/Quake_Map_Format) files.
#![feature(let_chains)]

mod tokenizer;
use std::io::Result;
use tokenizer::tokenizer;

/// Contains the defintion of the parser as well as the structs for all parsed data.
pub mod parser;
pub use parser::{Entity, parser};

/// Returns a [Vec<Entity>] representing all brushes and entities in a map.
///
/// # Errors
/// Will return `Err` if `str` is not a valid [Map](https://quakewiki.org/wiki/Quake_Map_Format) string.
pub fn parse(str: &str) -> Result<Vec<Entity>> {
    parser(tokenizer(str))
}
