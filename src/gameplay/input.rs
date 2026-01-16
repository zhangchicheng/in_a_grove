use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use crate::common::input::{GameInputMap, PlayerAction};
use crate::common::{Player, Velocity};
use crate::states::GameplayState;

const PLAYER_SPEED: f32 = 200.0;

pub fn sync_input_map(
    input_map: Res<GameInputMap>,
    mut query: Query<&mut InputMap<PlayerAction>, With<Player>>,
) {
    if input_map.is_changed() {
        for mut map in query.iter_mut() {
            *map = input_map.0.clone();
        }
    }
}

pub fn player_input(mut query: Query<(&mut Velocity, &ActionState<PlayerAction>), With<Player>>) {
    if let Ok((mut vel, action_state)) = query.single_mut() {
        // Horizontal movement
        let mut dir_x = 0.0;
        if action_state.pressed(&PlayerAction::MoveLeft) {
            dir_x -= 1.0;
        }
        if action_state.pressed(&PlayerAction::MoveRight) {
            dir_x += 1.0;
        }
        vel.lin.x = dir_x * PLAYER_SPEED;
    }
}

pub fn pause_input(
    query: Query<&ActionState<PlayerAction>, With<Player>>,
    mut next_state: ResMut<NextState<GameplayState>>,
) {
    if let Ok(action_state) = query.single()
        && action_state.just_pressed(&PlayerAction::Pause)
    {
        next_state.set(GameplayState::Paused);
    }
}
