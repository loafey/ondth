use std::collections::HashMap;

use faststr::FastStr;
use qwak_helper_types::{PickupData, PickupType};

pub fn get_pickups() -> HashMap<FastStr, PickupData> {
    [
        ("weapon_bayonet".into(), PickupData {
            pickup_type: PickupType::Weapon,
            classname: "weapon_bayonet".into(),
            gives: "weapon_bayonet".into(),
            pickup_model: "models/Pickups/Guns/Bayonet.obj".into(),
            pickup_material: "models/Pickups/Guns/Bayonet.mtl".into(),
            texture_file: "textures/weapons/WeaponMegaTexture.png".into(),
            scale: 0.01,
        }),
        ("weapon_dynamite".into(), PickupData {
            pickup_type: PickupType::Weapon,
            classname: "weapon_dynamite".into(),
            gives: "weapon_dynamite".into(),
            pickup_model: "models/Pickups/Guns/Dynamite.obj".into(),
            pickup_material: "models/Pickups/Guns/Dynamite.mtl".into(),
            texture_file: "textures/weapons/WeaponMegaTexture.png".into(),
            scale: 0.01,
        }),
        ("weapon_rpg".into(), PickupData {
            pickup_type: PickupType::Weapon,
            classname: "weapon_rpg".into(),
            gives: "weapon_rpg".into(),
            pickup_model: "models/Pickups/Guns/Rpg.obj".into(),
            pickup_material: "models/Pickups/Guns/Rpg.mtl".into(),
            texture_file: "textures/weapons/WeaponMegaTexture.png".into(),
            scale: 0.01,
        }),
        ("weapon_flamethrower".into(), PickupData {
            pickup_type: PickupType::Weapon,
            classname: "weapon_flamethrower".into(),
            gives: "weapon_flamethrower".into(),
            pickup_model: "models/Pickups/Guns/Flamethrower.obj".into(),
            pickup_material: "models/Pickups/Guns/Flamethrower.mtl".into(),
            texture_file: "textures/weapons/WeaponMegaTexture.png".into(),
            scale: 0.01,
        }),
        ("weapon_pumpshotgun".into(), PickupData {
            pickup_type: PickupType::Weapon,
            classname: "weapon_pumpshotgun".into(),
            gives: "weapon_pumpshotgun".into(),
            pickup_model: "models/Pickups/Guns/PumpShotgun.obj".into(),
            pickup_material: "models/Pickups/Guns/PumpShotgun.mtl".into(),
            texture_file: "textures/weapons/WeaponMegaTexture.png".into(),
            scale: 0.01,
        }),
        ("weapon_supershotgun".into(), PickupData {
            pickup_type: PickupType::Weapon,
            classname: "weapon_supershotgun".into(),
            gives: "weapon_supershotgun".into(),
            pickup_model: "models/Pickups/Guns/SuperShotgun.obj".into(),
            pickup_material: "models/Pickups/Guns/SuperShotgun.mtl".into(),
            texture_file: "textures/weapons/WeaponMegaTexture.png".into(),
            scale: 0.01,
        }),
        ("weapon_revolver".into(), PickupData {
            pickup_type: PickupType::Weapon,
            classname: "weapon_revolver".into(),
            gives: "weapon_revolver".into(),
            pickup_model: "models/Pickups/Guns/Revolver.obj".into(),
            pickup_material: "models/Pickups/Guns/Revolver.mtl".into(),
            texture_file: "textures/weapons/WeaponMegaTexture.png".into(),
            scale: 0.01,
        }),
        ("weapon_supercoolgun".into(), PickupData {
            pickup_type: PickupType::Weapon,
            classname: "weapon_supercoolgun".into(),
            gives: "weapon_supercoolgun".into(),
            pickup_model: "models/Pickups/Guns/SuperCoolGun.obj".into(),
            pickup_material: "models/Pickups/Guns/SuperCoolGun.mtl".into(),
            texture_file: "textures/weapons/WeaponMegaTexture.png".into(),
            scale: 0.01,
        }),
        ("weapon_smgbelter".into(), PickupData {
            pickup_type: PickupType::Weapon,
            classname: "weapon_smgbelter".into(),
            gives: "weapon_smgbelter".into(),
            pickup_model: "models/Pickups/Guns/SmgBelter.obj".into(),
            pickup_material: "models/Pickups/Guns/SmgBelter.mtl".into(),
            texture_file: "textures/weapons/WeaponMegaTexture.png".into(),
            scale: 0.01,
        }),
        ("weapon_nukegun".into(), PickupData {
            pickup_type: PickupType::Weapon,
            classname: "weapon_nukegun".into(),
            gives: "weapon_nukegun".into(),
            pickup_model: "models/Pickups/Guns/NukeGun.obj".into(),
            pickup_material: "models/Pickups/Guns/NukeGun.mtl".into(),
            texture_file: "textures/weapons/WeaponMegaTexture.png".into(),
            scale: 0.01,
        }),
    ]
    .into()
}
