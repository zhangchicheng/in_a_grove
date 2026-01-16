use crate::common::input::PlayerAction;
use crate::common::{Health, Velocity};
use crate::gameplay::mechanics::*;
use crate::gameplay::ui::is_cursor_on_ui;
use bevy::prelude::*;
use bevy_aseprite_ultra::prelude::*;
use leafwing_input_manager::prelude::*;

type CharacterQuery<'a> = (
    &'a mut CharacterState,
    &'a Health,
    &'a mut Velocity,
    Option<&'a ActionState<PlayerAction>>,
    Option<&'a mut JumpBuffer>,
    Option<&'a mut CoyoteTime>,
);

pub fn character_state_decision_system(
    mut query: Query<CharacterQuery>,
    time: Res<Time>,
    ui_interaction_query: Query<&Interaction, With<Node>>,
) {
    let dt = time.delta();
    let cursor_on_ui = is_cursor_on_ui(ui_interaction_query);

    for (mut state, health, mut velocity, input, mut jump_buffer, mut coyote_time) in &mut query {
        let now = time.elapsed_secs_f64();

        // Update Timers
        let mut wants_to_jump = false;

        if let Some(ref mut buffer) = jump_buffer {
            buffer.timer.tick(dt);
            if let Some(action_state) = input
                && action_state.just_pressed(&PlayerAction::Jump)
            {
                buffer.timer.reset();
            }
            if !buffer.timer.is_finished() {
                wants_to_jump = true;
            }
        }

        if let Some(ref mut coyote) = coyote_time {
            if velocity.on_ground {
                coyote.timer.reset();
            } else {
                coyote.timer.tick(dt);
            }
        }

        // 1. Highest Priority: Death
        if health.current <= 0.0 {
            if state.status != CharacterStatus::Dead {
                state.status = CharacterStatus::Dead;
            }
            continue;
        }

        // 2. State Lock Check
        if now < state.locked_until {
            continue;
        }

        // 3. Attack/Skill Decision (Only for Player for now)
        if let Some(action_state) = input {
            // UI Blocking Check
            if !cursor_on_ui {
                if action_state.just_pressed(&PlayerAction::Attack1) {
                    state.status = CharacterStatus::Attack(1);
                    state.locked_until = now + 0.5; // TODO: Get from MoveLibrary
                    continue;
                }
                if action_state.just_pressed(&PlayerAction::Attack2) {
                    state.status = CharacterStatus::Attack(2);
                    state.locked_until = now + 0.5;
                    continue;
                }
                if action_state.just_pressed(&PlayerAction::Attack3) {
                    state.status = CharacterStatus::Attack(3);
                    state.locked_until = now + 0.5;
                    continue;
                }
                if action_state.just_pressed(&PlayerAction::ThrowSpear) {
                    state.status = CharacterStatus::Throw;
                    state.locked_until = now + 0.8;
                    continue;
                }
                if action_state.just_pressed(&PlayerAction::Parry) {
                    state.status = CharacterStatus::Parry;
                    state.locked_until = now + 0.5;
                    continue;
                }
            }
        }

        // Jump Decision
        if wants_to_jump {
            let can_jump = velocity.on_ground
                || (coyote_time
                    .as_ref()
                    .map(|c| !c.timer.is_finished())
                    .unwrap_or(false))
                || velocity.jumps_left > 0;

            if can_jump {
                state.status = CharacterStatus::Jump;
                velocity.lin.y = 400.0; // JUMP_VELOCITY
                if !velocity.on_ground {
                    // mid-air jump consumes one extra
                    if velocity.jumps_left > 0 {
                        velocity.jumps_left -= 1;
                    }
                }
                // Reset coyote time to prevent double jumping from coyote
                if let Some(ref mut coyote) = coyote_time {
                    coyote.timer.set_elapsed(std::time::Duration::from_secs(10));
                }
                continue;
            }
        }

        // 4. Movement Decision (Lowest Priority)
        if !velocity.on_ground {
            if velocity.lin.y < 0.0 && state.status != CharacterStatus::Fall {
                state.status = CharacterStatus::Fall;
            }
        } else if velocity.lin.x.abs() > 10.0 {
            if state.status != CharacterStatus::Walk {
                state.status = CharacterStatus::Walk;
            }
        } else if state.status != CharacterStatus::Idle {
            state.status = CharacterStatus::Idle;
        }
    }
}

pub fn sync_animation_system(
    mut query: Query<(&CharacterState, &mut AseAnimation), Changed<CharacterState>>,
    aseprites: Res<Assets<Aseprite>>,
) {
    for (state, mut anim) in &mut query {
        match state.status {
            CharacterStatus::Idle => {
                anim.animation.tag = Some("idle".into());
                anim.animation.repeat = AnimationRepeat::Loop;
            }
            CharacterStatus::Walk => {
                anim.animation.tag = Some("walk".into());
                anim.animation.repeat = AnimationRepeat::Loop;
            }
            CharacterStatus::Attack(n) => {
                anim.animation.tag = Some(format!("attack_{}", n));
                anim.animation.repeat = AnimationRepeat::Count(0);
            }
            CharacterStatus::Throw => {
                anim.animation.tag = Some("throw_spear".into());
                anim.animation.repeat = AnimationRepeat::Count(0);
            }
            CharacterStatus::Parry => {
                anim.animation.tag = Some("parry".into());
                anim.animation.repeat = AnimationRepeat::Count(0);
            }
            CharacterStatus::Hurt => {
                let tag = if let Some(aseprite) = aseprites.get(&anim.aseprite) {
                    if aseprite.tags.contains_key("hurt") {
                        "hurt"
                    } else {
                        "hurt_1"
                    }
                } else {
                    "hurt"
                };
                anim.animation.tag = Some(tag.into());
                anim.animation.repeat = AnimationRepeat::Count(0);
            }
            CharacterStatus::Dead => {
                anim.animation.tag = Some("death".into());
                anim.animation.repeat = AnimationRepeat::Count(0);
            }
            CharacterStatus::Jump => {
                let tag = if let Some(aseprite) = aseprites.get(&anim.aseprite) {
                    if aseprite.tags.contains_key("jump") {
                        "jump"
                    } else {
                        "idle"
                    }
                } else {
                    "idle"
                };
                anim.animation.tag = Some(tag.into());
                anim.animation.repeat = AnimationRepeat::Loop;
            }
            CharacterStatus::Fall => {
                let tag = if let Some(aseprite) = aseprites.get(&anim.aseprite) {
                    if aseprite.tags.contains_key("fall") {
                        "fall"
                    } else if aseprite.tags.contains_key("jump") {
                        "jump"
                    } else {
                        "idle"
                    }
                } else {
                    "idle"
                };
                anim.animation.tag = Some(tag.into());
                anim.animation.repeat = AnimationRepeat::Loop;
            }
        }

        anim.animation.playing = true;
        anim.animation.queue.clear();

        // Queue idle after non-looping animations
        if !matches!(
            state.status,
            CharacterStatus::Idle
                | CharacterStatus::Walk
                | CharacterStatus::Dead
                | CharacterStatus::Jump
                | CharacterStatus::Fall
        ) {
            anim.animation
                .queue
                .push_back(("idle".into(), AnimationRepeat::Loop));
        }
    }
}
