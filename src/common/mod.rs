use bevy::prelude::*;

pub mod components;
pub mod input;
pub use components::*;

/// Automatically despawn this entity when exiting the specified state
#[derive(Component)]
pub struct DespawnOnExit<T: Send + Sync + 'static>(pub T);

pub struct DespawnPlugin;

impl Plugin for DespawnPlugin {
    fn build(&self, app: &mut App) {
        use crate::states::GameState;
        app.add_systems(OnExit(GameState::Menu), despawn_with(GameState::Menu))
            .add_systems(
                OnExit(GameState::Settings),
                despawn_with(GameState::Settings),
            )
            .add_systems(
                OnExit(GameState::Gameplay),
                despawn_with(GameState::Gameplay),
            )
            .add_systems(
                OnExit(GameState::GameOver),
                despawn_with(GameState::GameOver),
            );
    }
}

pub fn despawn_with<T: PartialEq + Send + Sync + 'static + Copy>(
    state: T,
) -> impl FnMut(Commands, Query<(Entity, &DespawnOnExit<T>)>) {
    move |mut commands: Commands, query: Query<(Entity, &DespawnOnExit<T>)>| {
        for (entity, marker) in &query {
            if marker.0 == state {
                commands.entity(entity).despawn();
            }
        }
    }
}
