use super::BrushEntity;
use bevy::{ecs::schedule::SystemConfigs, math::Vec3, prelude::*, time::Time};

#[derive(Debug, Component, Clone, Copy)]
pub struct RotateBrush {
    goal: Vec3,
    current_rot: Vec3,
    time: f32,
    cur_time: f32,
}
impl RotateBrush {
    pub fn new(goal: Vec3, current_rot: Vec3, time: f32) -> Self {
        Self {
            goal,
            current_rot,
            time,
            cur_time: 0.0,
        }
    }
    pub fn update(
        time: Res<Time>,
        mut commands: Commands,
        mut query: Query<(Entity, &mut RotateBrush, &mut Transform), With<BrushEntity>>,
    ) {
        for (ent, mut tb, mut t) in &mut query {
            if tb.time == 0.0 {
                let (x, y, z) = t.rotation.to_euler(EulerRot::XYZ);
                t.rotation =
                    Quat::from_euler(EulerRot::XYZ, x + tb.goal.x, y + tb.goal.y, z + tb.goal.z);
                commands.entity(ent).remove::<RotateBrush>();
            } else {
                tb.current_rot = tb.current_rot.lerp(tb.goal, tb.cur_time / tb.time);
                t.rotation = Quat::from_euler(
                    EulerRot::XYZ,
                    tb.current_rot.x,
                    tb.current_rot.y,
                    tb.current_rot.z,
                );
                tb.cur_time += time.delta_secs();
                tb.cur_time = tb.cur_time.min(tb.time);
                if tb.cur_time >= tb.time {
                    commands.entity(ent).remove::<RotateBrush>();
                }
            }
        }
    }
}

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
    (TranslateBrush::update, RotateBrush::update).into_configs()
}
