#![feature(downcast_unchecked)]
//! Contains multiple types which help are used when calling `qwak` plugin
//! methods.
use std::{
    any::{Any, TypeId},
    collections::{HashMap, hash_map::Entry},
    mem::transmute,
};

use extism_pdk::{FromBytes, Msgpack, ToBytes};
use faststr::FastStr;
use serde::{Deserialize, Serialize};

/// A heterogenus HashMap that uses values type's as keys.
#[derive(Debug, Default)]
pub struct TypeMap {
    inner: HashMap<TypeId, Box<dyn Any>>,
}
#[allow(unsafe_code)]
impl TypeMap {
    /// Insert a value into the map.
    pub fn put<T: 'static>(&mut self, value: T) {
        self.inner.insert(value.type_id(), Box::new(value));
    }
    /// Get a reference to a value from the map.
    pub fn get<T: 'static>(&self) -> Option<&T> {
        self.inner
            .get(&TypeId::of::<T>())
            .map(|b| unsafe { b.downcast_ref_unchecked() })
    }
    /// Get a copy of a value from the map.
    pub fn copy<T: 'static + Copy>(&self) -> Option<T> {
        self.inner
            .get(&TypeId::of::<T>())
            .map(|b| *unsafe { b.downcast_ref_unchecked() })
    }
    /// Get a mutable reference to a value from the map.
    pub fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.inner
            .get_mut(&TypeId::of::<T>())
            .map(|b| unsafe { b.downcast_mut_unchecked() })
    }
    /// Get a potential [Entry] to a value in the map.
    pub fn entry<T: 'static>(&mut self) -> Entry<TypeId, Box<T>> {
        unsafe { transmute(self.inner.entry(TypeId::of::<T>())) }
    }
    /// Clear the map.
    pub fn clear(&mut self) {
        self.inner.clear();
    }
}
/// Create thread_local storage variable.
#[macro_export]
macro_rules! storage {
    () => {
        #[thread_local]
        static STORAGE: LazyCell<RefCell<TypeMap>> =
            LazyCell::new(|| RefCell::new(TypeMap::default()));
    };
}
/// Get a value from the storage.
#[macro_export]
macro_rules! storage_get {
    ($ty:ty) => {
        STORAGE.borrow().copy::<$ty>()
    };
}
/// Put a value in storage.
#[macro_export]
macro_rules! storage_put {
    ($ty:expr) => {
        STORAGE.borrow_mut().put($ty)
    };
}
/// Clear the storage.
#[macro_export]
macro_rules! storage_clear {
    () => {
        STORAGE.borrow_mut().clear()
    };
}

/// The argument to [`map_interact`](../qwak_shared/trait.QwakPlugin.html#tymethod.map_interact).
#[derive(Debug, Clone, FromBytes, ToBytes, Deserialize, Serialize)]
#[encoding(Msgpack)]
pub struct MapInteraction {
    /// The script to run
    pub script: String,
    /// The optional target entity
    pub target: Option<String>,
    /// Argument to be passed to the function (in JSon).
    pub argument: Option<String>,
    /// The id of the player activating the interaction
    pub player_id: u64,
}

/// The data for a projectile
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Projectile {
    /// The projectile id
    pub id: FastStr,
    /// The projectile model. Should be OBJ.
    pub model_file: FastStr,
    /// The texture location. Should be PNG.
    pub texture_file: FastStr,
    /// The scale of the model.
    pub scale: f32,
    /// The rotation of the model.
    pub rotation: [f32; 3],
    /// The flying speed of the projectile.
    pub speed: f32,
}

/// The type of a pickup.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PickupType {
    /// This will result in a weapon.
    Weapon,
}

/// The data for a pickup item.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PickupData {
    /// The type of the pickup.
    pub pickup_type: PickupType,
    /// The map classname.
    pub classname: FastStr,
    /// The item you will get by picking this up.
    pub gives: FastStr,
    /// The model for this pickup, should be OBJ.
    pub pickup_model: FastStr,
    /// The material for this pickup, should be MTL.
    pub pickup_material: FastStr,
    /// The texture for this pickup, should be PNG.
    pub texture_file: FastStr,
    /// The scale of this texture.
    pub scale: f32,
}
