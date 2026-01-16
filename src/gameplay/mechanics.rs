use crate::gameplay::GameplaySet;
use crate::gameplay::mechanics_systems::*;
use crate::states::GameState;
use bevy::prelude::*;
use std::collections::HashMap;

pub struct MechanicsPlugin;

impl Plugin for MechanicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<CommandEvent>()
            .add_message::<HitEvent>()
            .init_resource::<MoveLibrary>()
            .add_systems(OnEnter(GameState::Gameplay), setup_mechanics)
            .add_systems(
                Update,
                (character_state_decision_system, sync_animation_system)
                    .chain()
                    .in_set(GameplaySet::Input)
                    .run_if(in_state(GameState::Gameplay)),
            );
    }
}

fn setup_mechanics(mut move_library: ResMut<MoveLibrary>) {
    move_library.moves.insert(
        "attack_1".into(),
        MoveData {
            animation_tag: "attack_1".into(),
            damage: 10.0,
            knockback: 50.0,
            startup_frames: 6,
            active_frames: 12,
            recovery_frames: 10,
            on_hit_frame_advantage: 5,
        },
    );

    move_library.moves.insert(
        "attack_2".into(),
        MoveData {
            animation_tag: "attack_2".into(),
            damage: 20.0,
            knockback: 100.0,
            startup_frames: 10,
            active_frames: 4,
            recovery_frames: 15,
            on_hit_frame_advantage: 2,
        },
    );

    move_library.moves.insert(
        "throw_spear".into(),
        MoveData {
            animation_tag: "throw_spear".into(),
            damage: 15.0,
            knockback: 50.0,
            startup_frames: 18,
            active_frames: 7,
            recovery_frames: 20,
            on_hit_frame_advantage: 0,
        },
    );
}

#[derive(Component)]
pub struct JumpBuffer {
    pub timer: Timer,
}

impl Default for JumpBuffer {
    fn default() -> Self {
        let mut timer = Timer::from_seconds(0.1, TimerMode::Once);
        timer.set_elapsed(std::time::Duration::from_secs_f32(0.1));
        Self { timer }
    }
}

#[derive(Component)]
pub struct CoyoteTime {
    pub timer: Timer,
}

impl Default for CoyoteTime {
    fn default() -> Self {
        let mut timer = Timer::from_seconds(0.1, TimerMode::Once);
        timer.set_elapsed(std::time::Duration::from_secs_f32(0.1));
        Self { timer }
    }
}

// Core State Enum
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Reflect)]
pub enum CharacterStatus {
    #[default]
    Idle,
    Walk,
    Jump,
    Fall,
    Attack(u8), // Attack 1, 2, 3
    Throw,
    Parry,
    Hurt,
    Dead,
}

// Component attached to characters
#[derive(Component)]
pub struct CharacterState {
    pub status: CharacterStatus,
    pub locked_until: f64,  // Time until which the state is locked
    #[allow(dead_code)]
    pub facing_right: bool, // Orientation
}

impl Default for CharacterState {
    fn default() -> Self {
        Self {
            status: CharacterStatus::Idle,
            locked_until: 0.0,
            facing_right: true,
        }
    }
}

// Events
#[derive(Event, Message)]
#[allow(dead_code)]
pub struct CommandEvent {
    pub player_entity: Entity,
    pub command: String, // e.g., "attack_1", "jump"
}

#[derive(Event, Message)]
pub struct HitEvent {
    #[allow(dead_code)]
    pub attacker: Entity,
    #[allow(dead_code)]
    pub victim: Entity,
    #[allow(dead_code)]
    pub move_id: String,
    #[allow(dead_code)]
    pub damage: f32,
    #[allow(dead_code)]
    pub knockback: f32,
    #[allow(dead_code)]
    pub hit_pos: Vec2,
}

// Move Data Resource
#[derive(Resource, Default)]
pub struct MoveLibrary {
    pub moves: HashMap<String, MoveData>,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct MoveData {
    pub animation_tag: String, // Tag in Aseprite
    #[allow(dead_code)]
    pub damage: f32,
    #[allow(dead_code)]
    pub knockback: f32,
    pub startup_frames: usize, // Logic frames, or use time
    pub active_frames: usize,
    pub recovery_frames: usize,
    #[allow(dead_code)]
    pub on_hit_frame_advantage: i32,
}
