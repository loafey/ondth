use bevy::{
    ecs::{
        component::Component,
        schedule::{IntoSystemConfigs, SystemConfigs},
        system::{Commands, Query},
    },
    math::Vec3,
    transform::components::Transform,
};
use bevy_hanabi::{ParticleEffect, ParticleEffectBundle};

use crate::{particles::ParticleMap, resources::projectiles::Projectile};

pub mod message;
pub mod pickup;
pub mod projectiles;

#[derive(Component)]
pub struct ProjectileEntity {
    pub dir: Vec3,
    pub data: Projectile,
}
impl ProjectileEntity {
    pub fn systems() -> SystemConfigs {
        (Self::update, Self::collision).into_configs()
    }

    pub fn update(mut query: Query<(&ProjectileEntity, &mut Transform)>) {
        for (ent, mut trans) in &mut query {
            trans.translation += ent.dir * ent.data.speed;
        }
    }

    pub fn collision(_query: Query<&ProjectileEntity>) {}
}

pub fn hitscan_hit_gfx(commands: &mut Commands, poss: &[Vec3], particles: &ParticleMap) {
    for pos in poss {
        commands.spawn(ParticleEffectBundle {
            effect: ParticleEffect::new(particles.demo.clone_weak()),
            transform: Transform::from_translation(*pos),
            ..Default::default()
        });
    }
}
