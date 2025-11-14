mod animation;
mod components;
mod menu;
mod systems;
mod scene;

use bevy::prelude::*;
use components::*;
use systems::*;
use scene::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .init_state::<GameState>()
        .init_resource::<MenuSelection>()
        .init_resource::<GameSettings>()
        .init_resource::<SettingsSelection>()
        .init_resource::<CurrentLevel>()
        .init_resource::<LevelSelection>()
    .add_systems(Startup, (setup_camera, preload_assets))
        // Animation plugin
        .add_plugins(animation::animation_plugin)
        // Menu state systems
        .add_plugins(menu::menu_plugin)
        // Playing state systems
        .add_systems(OnEnter(GameState::Playing), setup)
        .add_systems(OnExit(GameState::Playing), cleanup_playing)
        .add_systems(
            PreUpdate,
            move_platforms.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            (
                player_input,
                animation::update_animation_state,
                physics,
                camera_follow,
                hud_health,
                pause_input,
                check_game_over,
            )
                .run_if(in_state(GameState::Playing)),
        )
        // Paused state systems
        .add_systems(OnEnter(GameState::Paused), setup_paused_menu)
        .add_systems(Update, paused_input.run_if(in_state(GameState::Paused)))
        .add_systems(OnExit(GameState::Paused), cleanup_paused_ui)
        // Game Over state systems
        .add_systems(OnEnter(GameState::GameOver), setup_game_over_menu)
        .add_systems(
            Update,
            game_over_input.run_if(in_state(GameState::GameOver)),
        )
        .add_systems(OnExit(GameState::GameOver), cleanup_game_over_ui)
        .run();
}
