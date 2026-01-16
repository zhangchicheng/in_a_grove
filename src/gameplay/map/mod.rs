mod audio;
mod loader;
mod physics;

pub use loader::load_map;

use bevy::prelude::*;
use bevy_ecs_tiled::physics::TiledPhysicsPlugin;
use bevy_ecs_tiled::prelude::*;

use crate::GameState;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TiledPlugin::default())
            .add_plugins(TiledPhysicsPlugin::<physics::GamePhysicsBackend>::default())
            .add_systems(OnEnter(GameState::Gameplay), load_map)
            .add_systems(
                Update,
                audio::spawn_map_audio.run_if(in_state(GameState::Gameplay)),
            );
    }
}
