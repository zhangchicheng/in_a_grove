use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

use crate::common::DespawnOnExit;
use crate::states::GameState;

/// Spawns the TMX-backed bamboo forest map via `bevy_ecs_tiled`.
pub fn load_map(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        TiledMap(asset_server.load("maps/bamboo_forest.tmx")),
        TilemapAnchor::Center,
        Transform::default(),
        GlobalTransform::default(),
        Visibility::default(),
        DespawnOnExit(GameState::Gameplay),
        Name::new("BambooForestMap"),
    ));
}
