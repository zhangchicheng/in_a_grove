use bevy::prelude::*;

/// Top-level game states.
/// Each state corresponds to a different screen with different systems running.
#[derive(States, Debug, Hash, PartialEq, Eq, Clone, Copy, Default)]
pub enum GameState {
    /// Loading assets
    #[default]
    Loading,
    /// Main menu - first screen shown
    Menu,
    /// Settings screen - configure game options
    Settings,
    /// Active gameplay - player controls character
    Gameplay,
    /// Game over screen - shown when player dies
    GameOver,
}

/// Sub-state used while GameState::Gameplay is active.
/// Controls whether gameplay is running or paused.
#[derive(States, Clone, Copy, Default, Eq, PartialEq, Debug, Hash)]
pub enum GameplayState {
    /// Default state when not in gameplay
    #[default]
    Disabled,
    /// Actively playing the game
    Playing,
    /// Game is paused, showing pause overlay
    Paused,
    /// Player has died, showing game over overlay
    GameOver,
}
