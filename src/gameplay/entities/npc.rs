use bevy::prelude::*;
use bevy_aseprite_ultra::prelude::*;
use crate::assets::scene::GameAssets;
use crate::common::{DespawnOnExit, Health, Npc, Velocity, ColliderSize, Player};
use crate::states::GameState;
use crate::gameplay::combat::HitTracker;
use crate::gameplay::mechanics::{CharacterState, CharacterStatus};

#[derive(Component)]
pub struct NpcAi {
    pub state: AiState,
    pub timer: Timer,          // Used for idle, reaction delay, or patrol pause
    pub target: Option<Entity>, // Locked target (player)
    pub attack_cooldown: Timer, // Attack cooldown
    pub patrol_center: Vec2,    // Patrol center
    pub patrol_range: f32,      // Patrol range
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiState {
    Patrol,     // Patrol: Walk left and right within range
    Chase,      // Chase: Move towards player
    ReadyToAttack, // Pre-attack: Stop within range, pause (simulate reaction)
    Attacking,  // Attacking: Wait for attack to finish
    #[allow(dead_code)]
    Flee,       // Flee: Retreat when low health
    Hurt,       // Hurt stun (handled by physics/mechanics, AI paused)
}

impl Default for NpcAi {
    fn default() -> Self {
        let mut attack_cooldown = Timer::from_seconds(2.0, TimerMode::Once);
        attack_cooldown.set_elapsed(std::time::Duration::from_secs(2)); // Start finished

        Self {
            state: AiState::Patrol,
            timer: Timer::from_seconds(1.0, TimerMode::Once),
            target: None,
            attack_cooldown,
            patrol_center: Vec2::ZERO,
            patrol_range: 150.0,
        }
    }
}

pub fn spawn_npc(
    commands: &mut Commands,
    game_assets: &GameAssets,
) {
    let spawn_pos = Vec2::new(100.0, 200.0);
    // Spawn NPC (Sohei)
    commands.spawn((
        AseAnimation {
            aseprite: game_assets.sohei.clone(),
            animation: Animation {
                tag: Some("idle".into()),
                speed: 0.5,
                ..default()
            },
        },
        Transform::from_xyz(spawn_pos.x, spawn_pos.y, 10.0),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        ViewVisibility::default(),
        Sprite::default(),
        Npc,
        DespawnOnExit(GameState::Gameplay),
        Velocity {
            lin: Vec2::ZERO,
            on_ground: false,
            jumps_left: 0,
        },
        ColliderSize(Vec2::new(30.0, 48.0)),
        Health::new(50.0),
        HitTracker::default(),
        CharacterState::default(),
        NpcAi {
            patrol_center: spawn_pos,
            patrol_range: 150.0,
            ..default()
        },
    ));
}

#[allow(clippy::type_complexity)]
pub fn npc_behavior(
    time: Res<Time>,
    mut npc_query: Query<(Entity, &mut NpcAi, &mut CharacterState, &Transform, &mut Velocity, &mut Sprite)>,
    player_query: Query<(Entity, &Transform, &Health), With<Player>>,
) {
    // Get player info (assuming single player)
    let (player_entity, player_tf, player_health) = if let Some(p) = player_query.iter().next() {
        p
    } else {
        return; // No player, AI rests
    };

    if !player_health.is_alive() {
        return; // Player dead, idle
    }

    for (_npc_entity, mut ai, mut char_state, npc_tf, mut velocity, mut sprite) in npc_query.iter_mut() {
        // 1. State blocking
        if matches!(char_state.status, CharacterStatus::Dead) {
            velocity.lin.x = 0.0;
            continue;
        }
        
        if matches!(char_state.status, CharacterStatus::Hurt) {
            ai.state = AiState::Hurt;
            velocity.lin.x = 0.0;
            continue;
        }

        // Recover from hurt
        if ai.state == AiState::Hurt {
            ai.state = AiState::Chase;
        }

        // Update timers
        ai.timer.tick(time.delta());
        ai.attack_cooldown.tick(time.delta());

        // --- Core Fix 1: Separate Axis Calculation ---
        let dx = player_tf.translation.x - npc_tf.translation.x;
        let dy = player_tf.translation.y - npc_tf.translation.y;
        
        let dist_x = dx.abs(); // Horizontal distance
        let dist_y = dy.abs(); // Vertical distance (check if on same platform)
        
        // --- Core Fix 2: Check Asset Orientation ---
        // If your asset is originally drawn facing LEFT, change this to false
        // If your asset is originally drawn facing RIGHT, keep true
        let asset_faces_right = false; 
        
        // Calculate flip state
        let should_flip = if asset_faces_right {
            dx < 0.0 // Target on left, asset faces right -> flip
        } else {
            dx > 0.0 // Target on right, asset faces left -> flip
        };
        
        // --- FSM Core Logic ---
        match ai.state {
            AiState::Patrol => {
                if dist_x < 300.0 && dist_y < 100.0 {
                    ai.state = AiState::Chase;
                    ai.target = Some(player_entity);
                } else {
                    let speed = 50.0;
                    let right_bound = ai.patrol_center.x + ai.patrol_range;
                    let left_bound = ai.patrol_center.x - ai.patrol_range;

                    if velocity.lin.x >= 0.0 && npc_tf.translation.x > right_bound {
                        velocity.lin.x = -speed;
                    } else if velocity.lin.x <= 0.0 && npc_tf.translation.x < left_bound {
                        velocity.lin.x = speed;
                    } else if velocity.lin.x == 0.0 {
                         velocity.lin.x = speed;
                    }
                    
                    char_state.status = CharacterStatus::Walk;
                    // For patrol, we flip based on velocity
                    if asset_faces_right {
                        sprite.flip_x = velocity.lin.x < 0.0;
                    } else {
                        sprite.flip_x = velocity.lin.x > 0.0;
                    }
                }
            },
            
            AiState::Chase => {
                // Behavior: Move towards player
                let speed = 100.0;
                velocity.lin.x = dx.signum() * speed;
                
                // Face player
                sprite.flip_x = should_flip; 
                char_state.status = CharacterStatus::Walk;

                // Transition 1: Enter attack range
                // Note: Since there is no physical collision between Player and NPC,
                // this value determines exactly how close they stand (Center-to-Center).
                // If the attack misses, reduce this value. If they overlap too much, increase it.
                let attack_range_x = 30.0; 
                let attack_range_y = 20.0; // Allow 20px height difference

                if dist_x < attack_range_x && dist_y < attack_range_y {
                    velocity.lin.x = 0.0; // Stop
                    ai.state = AiState::ReadyToAttack;
                    ai.timer.set_duration(std::time::Duration::from_secs_f32(0.3)); // Reaction time 0.3s
                    ai.timer.reset();
                    char_state.status = CharacterStatus::Idle;
                }
                // Transition 2: Player too far or on different platform
                else if dist_x > 500.0 || dist_y > 150.0 {
                    ai.state = AiState::Patrol;
                    char_state.status = CharacterStatus::Idle;
                }
            },
            
            AiState::ReadyToAttack => {
                // Behavior: Stare at player, prepare to attack
                // Simulate human reaction delay
                
                // Face player even while waiting
                sprite.flip_x = should_flip; 

                if ai.timer.is_finished() {
                    if ai.attack_cooldown.is_finished() {
                        ai.state = AiState::Attacking;
                        // Trigger attack state in mechanics
                        char_state.status = CharacterStatus::Attack(1); 
                        char_state.locked_until = time.elapsed_secs_f64() + 0.8; // Lock character
                        
                        ai.attack_cooldown.reset(); // Reset cooldown
                    } else {
                        // Wait or chase if too far
                        if dist_x > 80.0 {
                            ai.state = AiState::Chase;
                        }
                    }
                }
            },
            
            AiState::Attacking => {
                // Behavior: Wait for attack animation to finish
                // Check if CharacterState has returned to Idle/Walk
                if !matches!(char_state.status, CharacterStatus::Attack(_)) {
                    // Attack finished
                    ai.state = AiState::Chase; // Continue chase or retreat
                }
            },
            
            _ => {}
        }
    }
}
