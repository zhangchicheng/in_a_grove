use bevy::prelude::*;
use bevy_aseprite_ultra::prelude::*;
use leafwing_input_manager::prelude::*;
use crate::assets::scene::GameAssets;
use crate::common::{DespawnOnExit, Health, Player, Velocity, ColliderSize};
use crate::common::input::{GameInputMap, PlayerAction};
use crate::states::{GameState, GameplayState};
use crate::gameplay::combat::HitTracker;
use crate::gameplay::mechanics::{CharacterState, CoyoteTime, JumpBuffer};

const MAX_JUMPS: u8 = 2;

pub fn spawn_player(
    commands: &mut Commands,
    game_assets: &GameAssets,
    input_map: &GameInputMap,
) {
    commands
        .spawn((
            AseAnimation {
                aseprite: game_assets.player.clone(),
                animation: Animation {
                    tag: Some("idle".into()),
                    speed: 0.5,
                    ..default()
                },
            },
            Transform::from_xyz(0.0, 200.0, 10.0),
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
            ViewVisibility::default(),
            Sprite::default(),
            Player,
            DespawnOnExit(GameState::Gameplay),
            Velocity {
                lin: Vec2::ZERO,
                on_ground: false,
                jumps_left: MAX_JUMPS,
            },
            ColliderSize(Vec2::splat(48.0)),
            Health::new(100.0),
            input_map.0.clone(),
            ActionState::<PlayerAction>::default(),
        ))
        .insert((
            HitTracker::default(),
            SpatialListener::new(48.0), // Match collider size roughly
            CharacterState::default(),
            JumpBuffer::default(),
            CoyoteTime::default(),
        ));
}

pub fn check_game_over(
    player_q: Query<&Health, With<Player>>,
    mut next_state: ResMut<NextState<GameplayState>>,
) {
    if let Ok(health) = player_q.single()
        && !health.is_alive()
    {
        next_state.set(GameplayState::GameOver);
    }
}
