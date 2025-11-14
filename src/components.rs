use bevy::prelude::*;

// Game states
#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    Menu,
    LevelSelection,
    Loading,
    Playing,
    Paused,
    GameOver,
    Settings,
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuSelection {
    StartGame,
    Settings,
    Exit,
}

impl Default for MenuSelection {
    fn default() -> Self {
        MenuSelection::StartGame
    }
}

#[derive(Resource, Debug, Clone)]
pub struct GameSettings {
    pub volume: f32,
    pub difficulty: Difficulty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    Easy,
    Normal,
    Hard,
}

impl Default for GameSettings {
    fn default() -> Self {
        GameSettings {
            volume: 1.0,
            difficulty: Difficulty::Normal,
        }
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsSelection {
    Volume,
    Difficulty,
    Back,
}

impl Default for SettingsSelection {
    fn default() -> Self {
        SettingsSelection::Volume
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurrentLevel {
    Level1,
    Level2,
    Level3,
}

impl Default for CurrentLevel {
    fn default() -> Self {
        CurrentLevel::Level1
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub enum LevelSelection {
    Level1,
    Level2,
    Level3,
    Back,
}

impl Default for LevelSelection {
    fn default() -> Self {
        LevelSelection::Level1
    }
}

// Game constants
pub const PLAYER_SPEED: f32 = 220.0;
pub const GRAVITY: f32 = 900.0;
pub const JUMP_VELOCITY: f32 = 420.0;
pub const COYOTE_TIME: f32 = 0.12;
pub const MAX_JUMPS: u8 = 2;

pub const CAMERA_LERP: f32 = 0.12; // how fast camera follows
pub const LEVEL_MIN: Vec2 = Vec2::new(-600.0, -400.0);
pub const LEVEL_MAX: Vec2 = Vec2::new(1200.0, 800.0);

// Components
#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Health { current: max, max }
    }

    pub fn take_damage(&mut self, amount: f32) {
        self.current = (self.current - amount).max(0.0);
    }

    pub fn heal(&mut self, amount: f32) {
        self.current = (self.current + amount).min(self.max);
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

// Marker for entities that belong to a loaded level so we can easily cleanup
#[derive(Component)]
pub struct LevelEntity;

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
    pub coyote: f32,
    pub jumps_left: u8,
}

#[derive(Resource)]
pub struct HudTimer(pub Timer);

// Animation components
#[derive(Component)]
pub struct AnimationIndices {
    pub idle_first: usize,
    pub idle_last: usize,
    pub run_first: usize,
    pub run_last: usize,
    pub current_first: usize,
    pub current_last: usize,
}

impl AnimationIndices {
    pub fn new() -> Self {
        // Gabe sprite sheet: 7 frames total
        // Frames 0: idle, Frames 1-6: run
        AnimationIndices {
            idle_first: 0,
            idle_last: 0,
            run_first: 1,
            run_last: 6,
            current_first: 0,
            current_last: 0,
        }
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct AnimationTimer(pub Timer);

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum AnimationState {
    Idle,
    Running,
}

impl Default for AnimationState {
    fn default() -> Self {
        AnimationState::Idle
    }
}

