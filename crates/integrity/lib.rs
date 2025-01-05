//! This crate exports a function that calculates the hash of the asset directory.
//!
//! ## Safety
//! This crate has not been proven cryptographically secure.

use sha256::digest;
use std::{fs::File, io::Read, path::Path};

/// Return the sha256 hash for the asset directory.
/// ## Safety
/// This crate has not been proven cryptographically secure.
#[allow(clippy::unnecessary_safety_doc)]
pub fn get_asset_hash() -> String {
    recurse("assets").unwrap()
}

fn recurse<P: AsRef<Path>>(p: P) -> std::io::Result<String> {
    let p = p.as_ref();
    let mut total = String::new();
    if p.is_dir() {
        for p in p.read_dir()? {
            let p = p?.path();
            total.push_str(&recurse(p)?);
        }
    } else {
        let mut f = File::open(p)?;
        let mut v = Vec::new();
        f.read_to_end(&mut v)?;
        total.push_str(&digest(v));
    }

    Ok(digest(total))
}
