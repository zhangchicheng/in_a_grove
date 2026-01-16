use crate::common::*;
use crate::gameplay::GameplaySet;
use crate::states::GameState;
use bevy::prelude::*;

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (flip_sprite_direction,)
                .run_if(in_state(GameState::Gameplay))
                .in_set(GameplaySet::Animation),
        );
    }
}

use crate::gameplay::mechanics::{CharacterState, CharacterStatus};

// System 3: Generic Sprite Flipping (Shared by Player and NPCs)
fn flip_sprite_direction(mut query: Query<(&Velocity, &mut Sprite, &CharacterState)>) {
    for (velocity, mut sprite, state) in &mut query {
        // Lock direction if attacking or dead or hurt
        if matches!(
            state.status,
            CharacterStatus::Attack(_)
                | CharacterStatus::Throw
                | CharacterStatus::Parry
                | CharacterStatus::Hurt
                | CharacterStatus::Dead
        ) {
            continue;
        }

        if velocity.lin.x > 0.0 {
            sprite.flip_x = true;
        } else if velocity.lin.x < 0.0 {
            sprite.flip_x = false;
        }
    }
}
