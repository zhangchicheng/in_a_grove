use crate::components::*;
use bevy::prelude::*;

pub fn animation_plugin(app: &mut App) {
    app.add_systems(Update, animate_sprite.run_if(in_state(GameState::Playing)));
}

pub fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(&AnimationIndices, &mut AnimationTimer, &mut Sprite)>,
) {
    for (indices, mut timer, mut sprite) in &mut query {
        timer.tick(time.delta());

        if timer.just_finished() {
            if let Some(atlas) = &mut sprite.texture_atlas {
                // Move to next frame, or loop back to first frame
                atlas.index = if atlas.index == indices.current_last {
                    indices.current_first
                } else {
                    atlas.index + 1
                };
            }
        }
    }
}

pub fn update_animation_state(
    mut query: Query<(&Velocity, &mut AnimationIndices, &mut AnimationState, &mut Sprite)>,
) {
    for (velocity, mut indices, mut state, mut sprite) in &mut query {
        let is_moving = velocity.lin.x.abs() > 1.0;
        let new_state = if is_moving {
            AnimationState::Running
        } else {
            AnimationState::Idle
        };

        // Handle direction flipping
        if velocity.lin.x < 0.0 {
            sprite.flip_x = true;
        } else if velocity.lin.x > 0.0 {
            sprite.flip_x = false;
        }

        // Update animation state and reset frame index if state changed
        if *state != new_state {
            *state = new_state;
            match new_state {
                AnimationState::Idle => {
                    indices.current_first = indices.idle_first;
                    indices.current_last = indices.idle_last;
                }
                AnimationState::Running => {
                    indices.current_first = indices.run_first;
                    indices.current_last = indices.run_last;
                }
            }
            // Reset animation frame to first frame of new state
            if let Some(atlas) = &mut sprite.texture_atlas {
                atlas.index = indices.current_first;
            }
        }
    }
}

