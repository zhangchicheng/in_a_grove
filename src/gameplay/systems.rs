use bevy::prelude::*;

use crate::assets::scene::GameAssets;
use crate::common::input::GameInputMap;
use crate::states::GameplayState;

use crate::gameplay::camera::spawn_gameplay_camera;
use crate::gameplay::entities::player::spawn_player;
use crate::gameplay::entities::npc::spawn_npc;

pub fn start_playing(mut next_state: ResMut<NextState<GameplayState>>) {
    next_state.set(GameplayState::Playing);
}

pub fn stop_gameplay(mut next_state: ResMut<NextState<GameplayState>>) {
    next_state.set(GameplayState::Disabled);
}

pub fn setup_gameplay(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    input_map: Res<GameInputMap>,
) {
    spawn_gameplay_camera(&mut commands);
    spawn_player(&mut commands, &game_assets, &input_map);
    spawn_npc(&mut commands, &game_assets);
}
