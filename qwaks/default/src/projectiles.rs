use faststr::FastStr;
use qwak_helper_types::Projectile;
use std::collections::HashMap;

pub fn get_projectiles() -> HashMap<FastStr, Projectile> {
    [
        ("nuke".into(), Projectile {
            id: "nuke".into(),
            model_file: "models/Projectile/Nuke.obj".into(),
            texture_file: "textures/weapons/WeaponMegaTexture.png".into(),
            scale: -0.01,
            rotation: [180.0, -180.0, 180.0],
            speed: 0.01,
        }),
        ("rocket".into(), Projectile {
            id: "rocket".into(),
            model_file: "models/Projectile/Rocket.obj".into(),
            texture_file: "textures/weapons/WeaponMegaTexture.png".into(),
            scale: -0.01,
            rotation: [180.0, -180.0, 180.0],
            speed: 0.1,
        }),
    ]
    .into()
}
