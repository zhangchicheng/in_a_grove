pub mod detection;
pub mod events;

use bevy::prelude::*;
use crate::states::GameState;

pub use detection::*;
pub use events::*;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AttackConfig>()
            .add_systems(OnEnter(GameState::Gameplay), setup_combat)
            .add_systems(
                Update,
                (
                    manage_hit_tracker,
                    spawn_projectiles,
                    projectile_movement,
                    attack_judgment,
                    apply_damage_system,
                    apply_knockback_system,
                    handle_hit_reaction_system,
                )
                    .run_if(in_state(GameState::Gameplay)),
            );
    }
}
