use bevy::prelude::*;
use crate::components::*;

/// Holds preloaded game-wide assets so level spawns can be fast.
#[derive(Resource)]
pub struct GameAssets {
    pub player_texture: Handle<Image>,
    pub player_layout: Handle<TextureAtlasLayout>,
}

/// Preload textures and atlas layouts used by player/levels.
/// Run at startup so later spawns are fast (no disk/GPU wait during scene switch).
pub fn preload_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture: Handle<Image> = asset_server.load("textures/gabe/gabe-idle-run.png");
    // 24x24 tiles, 7 columns, 1 row
    let layout = TextureAtlasLayout::from_grid(bevy::math::UVec2::splat(24), 7, 1, None, None);
    let layout_handle = layouts.add(layout);

    commands.insert_resource(GameAssets {
        player_texture: texture,
        player_layout: layout_handle,
    });

    info!("Preloaded GameAssets (player sprite atlas)" );
}

/// Helper to request a level load: set the current level and switch to Loading state.
pub fn request_load_level(mut current: ResMut<CurrentLevel>, mut next_state: ResMut<NextState<GameState>>, level: CurrentLevel) {
    *current = level;
    next_state.set(GameState::Loading);
}
