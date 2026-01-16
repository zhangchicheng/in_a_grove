use bevy::prelude::*;
use crate::common::{ColliderSize, MovingPlatform, Npc, Platform, Player, Velocity};

const GRAVITY: f32 = 1000.0;
const MAX_JUMPS: u8 = 2;

#[allow(clippy::type_complexity)]
pub fn physics(
    time: Res<Time>,
    mut params: ParamSet<(
        Query<(&mut Transform, &mut Velocity, &ColliderSize), Or<(With<Player>, With<Npc>)>>,
        Query<(&GlobalTransform, Option<&MovingPlatform>, &ColliderSize), With<Platform>>,
    )>,
) {
    let dt = time.delta_secs();

    // snapshot platforms first to avoid double-borrowing ParamSet while iterating players
    let mut platform_snapshots: Vec<(Vec2, Option<(f32, f32)>, Vec2)> = Vec::new();
    for (p_transform, moving, p_size) in params.p1().iter() {
        let translation = p_transform.translation();
        platform_snapshots.push((
            Vec2::new(translation.x, translation.y),
            moving.map(|m| (m.dir, m.speed)),
            p_size.0,
        ));
    }

    // iterate players (mutable borrow)
    for (mut transform, mut vel, size) in params.p0().iter_mut() {
        // apply gravity
        vel.lin.y -= GRAVITY * dt;

        // integrate desired position
        let mut pos = transform.translation;
        pos.x += vel.lin.x * dt;
        pos.y += vel.lin.y * dt;

        // reset grounded state; will set true if collision from above
        vel.on_ground = false;

        // AABB collision resolution with platform snapshots
        let a_pos = Vec2::new(pos.x, pos.y);
        let a_half = size.0 / 2.0;
        for (b_pos, moving_info, b_size) in platform_snapshots.iter() {
            let b_half = *b_size / 2.0;

            let delta = *b_pos - a_pos;
            let overlap = Vec2::new(
                a_half.x + b_half.x - delta.x.abs(),
                a_half.y + b_half.y - delta.y.abs(),
            );

            if overlap.x > 0.0 && overlap.y > 0.0 {
                // collision detected; resolve along smaller overlap
                if overlap.x < overlap.y {
                    // push in X
                    let sign = delta.x.signum();
                    pos.x -= sign * overlap.x;
                    vel.lin.x = 0.0;
                } else {
                    // push in Y
                    let sign = delta.y.signum();
                    pos.y -= sign * overlap.y;
                    vel.lin.y = 0.0;

                    // if sign < 0, player was above platform and moved up onto it => landed
                    if sign < 0.0 {
                        vel.on_ground = true;
                        vel.jumps_left = MAX_JUMPS;
                        // if platform is moving, inherit its horizontal velocity
                        if let Some((dir, speed)) = moving_info {
                            vel.lin.x += dir * speed;
                        }
                    }
                }
            }
        }

        transform.translation = Vec3::new(pos.x, pos.y, transform.translation.z);
    }
}

pub fn move_platforms(
    mut platforms: Query<(&mut Transform, &mut MovingPlatform), With<Platform>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for (mut tf, mut mv) in platforms.iter_mut() {
        tf.translation.x += mv.dir * mv.speed * dt;
        if tf.translation.x < mv.range_min {
            tf.translation.x = mv.range_min;
            mv.dir = 1.0;
        } else if tf.translation.x > mv.range_max {
            tf.translation.x = mv.range_max;
            mv.dir = -1.0;
        }
    }
}
