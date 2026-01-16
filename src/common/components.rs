use bevy::prelude::*;

// ============================================================================
// RESOURCES - Global state accessible by all systems
// ============================================================================

/// Tracks which scene/map is currently loaded
/// Each scene represents a part of the story-driven adventure
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(dead_code)]
pub enum CurrentScene {
    /// The beginning of the adventure
    #[default]
    Prologue,
    /// First major story area
    Chapter1,
    /// Second major story area
    Chapter2,
    /// Third major story area
    Chapter3,
    /// Final story area
    Epilogue,
}

/// Audio assets for menu interactions
#[derive(Resource)]
pub struct MenuAudioAssets {
    pub menu_blip: Handle<AudioSource>,
}

/// Tracks the currently selected button index for keyboard navigation
#[derive(Resource, Default)]
pub struct SelectedButtonIndex {
    pub index: usize,
    pub max_index: usize,
}

/// Marker for buttons that can be selected with keyboard navigation
#[derive(Component)]
pub struct KeyboardSelectable {
    pub index: usize,
}

/// Marker for the gameplay camera that tracks the player.
#[derive(Component)]
pub struct MainCamera;

/// Marker for the HUD root container
#[derive(Component)]
pub struct HudRoot;

/// Marker for the health bar fill element
#[derive(Component)]
pub struct HealthBarFill;

/// Marker for the health text display
#[derive(Component)]
pub struct HealthText;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Npc;

#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Health { current: max, max }
    }

    pub fn is_alive(&self) -> bool {
        self.current > 0.0
    }

    pub fn health_percent(&self) -> f32 {
        (self.current / self.max).clamp(0.0, 1.0)
    }
}

#[derive(Component)]
pub struct Platform;

#[derive(Component)]
pub struct MovingPlatform {
    pub range_min: f32,
    pub range_max: f32,
    pub speed: f32,
    pub dir: f32,
}

#[derive(Component)]
pub struct ColliderSize(pub Vec2);

#[derive(Component, Debug)]
pub struct Velocity {
    pub lin: Vec2,
    pub on_ground: bool,
    pub jumps_left: u8,
}
