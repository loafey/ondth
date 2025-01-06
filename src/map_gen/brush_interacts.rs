use bevy::{ecs::schedule::SystemConfigs, math::Vec3, prelude::*, time::Time};

use super::BrushEntity;

#[derive(Debug, Component, Clone, Copy)]
pub struct TranslateBrush {
    goal: Vec3,
    time: f32,
    cur_time: f32,
}
impl TranslateBrush {
    pub fn new(goal: Vec3, time: f32) -> Self {
        Self {
            goal,
            time,
            cur_time: 0.0,
        }
    }
    pub fn update(
        time: Res<Time>,
        mut commands: Commands,
        mut query: Query<(Entity, &mut TranslateBrush, &mut Transform), With<BrushEntity>>,
    ) {
        for (ent, mut tb, mut t) in &mut query {
            if tb.time == 0.0 {
                t.translation = tb.goal;
                commands.entity(ent).remove::<TranslateBrush>();
            } else {
                t.translation = t.translation.lerp(tb.goal, tb.cur_time / tb.time);
                tb.cur_time += time.delta_secs();
                tb.cur_time = tb.cur_time.min(tb.time);
                if tb.cur_time >= tb.time {
                    commands.entity(ent).remove::<TranslateBrush>();
                }
            }
        }
    }
}

pub fn systems() -> SystemConfigs {
    (TranslateBrush::update,).into_configs()
}
