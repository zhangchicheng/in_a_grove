use crate::common::{Health, Velocity};
use crate::gameplay::mechanics::{CharacterState, CharacterStatus, HitEvent};
use bevy::prelude::*;

pub fn apply_damage_system(
    mut hit_events: MessageReader<HitEvent>,
    mut query: Query<&mut Health>,
) {
    for event in hit_events.read() {
        if let Ok(mut health) = query.get_mut(event.victim) {
            health.current = (health.current - event.damage).max(0.0);
            info!(
                "Hit! Entity {:?} hit {:?} for {} damage. Health: {}",
                event.attacker, event.victim, event.damage, health.current
            );
        }
    }
}

pub fn apply_knockback_system(
    mut hit_events: MessageReader<HitEvent>,
    mut query: Query<(&mut Velocity, &GlobalTransform)>,
) {
    for event in hit_events.read() {
        if let Ok((mut velocity, victim_tf)) = query.get_mut(event.victim) {
            let direction = (victim_tf.translation().xy() - event.hit_pos).normalize_or_zero();
            velocity.lin += direction * event.knockback;
        }
    }
}

pub fn handle_hit_reaction_system(
    mut hit_events: MessageReader<HitEvent>,
    mut query: Query<(&mut CharacterState, &Health)>,
    time: Res<Time>,
) {
    for event in hit_events.read() {
        if let Ok((mut state, health)) = query.get_mut(event.victim) {
            if health.current <= 0.0 {
                state.status = CharacterStatus::Dead;
                state.locked_until = f64::MAX;
                info!("Entity {:?} died. State set to Dead.", event.victim);
            } else {
                state.status = CharacterStatus::Hurt;
                state.locked_until = time.elapsed_secs_f64() + 0.3;
            }
        }
    }
}
