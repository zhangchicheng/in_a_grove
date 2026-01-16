use crate::states::{GameState, GameplayState};
use bevy::prelude::*;

pub mod animation;
pub mod camera;
pub mod combat;
mod debug;
pub mod entities;
mod game_over;
pub mod input;
pub mod map;
pub mod mechanics;
mod mechanics_systems;
mod paused;
pub mod physics;
mod systems;
mod ui;

use animation::AnimationPlugin;
pub use game_over::GameOverPlugin;
use map::MapPlugin;
pub use mechanics::MechanicsPlugin;
pub use paused::PausedPlugin;
use systems::*;
use ui::UiPlugin;

/// System sets for organizing and ordering game systems.
///
/// These sets provide coarse-grained ordering to ensure predictable behavior
/// between different parts of the game. Systems in earlier sets run before
/// systems in later sets.
///
/// Used in gameplay.rs with .chain() to enforce: Input → Physics → Camera → Animation → UI
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameplaySet {
    /// Player input handling (reads input, updates velocity)
    /// Must run first to capture intent before physics
    Input,
    /// Physics simulation (applies velocity, handles collisions)
    /// Depends on Input setting velocity
    Physics,
    /// Camera updates (follows player)
    /// Depends on Physics finalizing player position
    Camera,
    /// Animation state updates
    /// Depends on Physics to check velocity for run/idle states
    Animation,
    /// UI updates (health bars, etc.)
    /// Runs last to display final state
    UI,
}

/// Plugin that manages the Gameplay screen
pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        // Initialize GameplayState
        app.init_state::<GameplayState>();

        // Add sub-plugins
        app.add_plugins((
            GameOverPlugin,
            PausedPlugin,
            AnimationPlugin,
            MapPlugin,
            debug::DebugPlugin,
            combat::CombatPlugin,
            MechanicsPlugin,
            UiPlugin,
        ));

        // Configure system set ordering
        app.configure_sets(
            Update,
            (
                GameplaySet::Input,
                GameplaySet::Camera,
                GameplaySet::Animation,
                GameplaySet::UI,
            )
                .chain()
                .run_if(in_state(GameState::Gameplay))
                .run_if(in_state(GameplayState::Playing)),
        );

        app.configure_sets(
            FixedUpdate,
            GameplaySet::Physics
                .run_if(in_state(GameState::Gameplay))
                .run_if(in_state(GameplayState::Playing)),
        );

        // Start gameplay when entering the Gameplay screen
        app.add_systems(
            OnEnter(GameState::Gameplay),
            (setup_gameplay, ui::setup_hud, start_playing),
        )
        .add_systems(OnExit(GameState::Gameplay), stop_gameplay)
        .add_systems(FixedUpdate, physics::move_platforms.in_set(GameplaySet::Physics))
        .add_systems(
            Update,
            (
                input::player_input,
                input::pause_input,
                entities::player::check_game_over,
                input::sync_input_map,
                entities::npc::npc_behavior,
            )
                .in_set(GameplaySet::Input),
        )
        .add_systems(FixedUpdate, physics::physics.in_set(GameplaySet::Physics))
        .add_systems(
            Update,
            (camera::camera_follow, camera::handle_camera_zoom).in_set(GameplaySet::Camera),
        )
        .add_systems(Update, ui::hud_health.in_set(GameplaySet::UI));
    }
}
