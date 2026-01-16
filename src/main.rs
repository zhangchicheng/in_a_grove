mod assets;
mod common;
mod gameplay;
mod loading;
mod menu;
mod settings;
mod states;

use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;
use bevy_modern_pixel_camera::prelude::*;

// use assets::scene::preload_assets;
use bevy_aseprite_ultra::AsepriteUltraPlugin;
use common::input::{GameInputMap, PlayerAction};
use common::{CurrentScene, DespawnPlugin};
use gameplay::GameplayPlugin;
use leafwing_input_manager::prelude::*;
use loading::LoadingPlugin;
use menu::MenuPlugin;
use settings::SettingsPlugin;
use states::GameState;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Bevy Game".into(),
                        resolution: bevy::window::WindowResolution::new(1280, 720),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(AsepriteUltraPlugin)
        .add_plugins(PixelCameraPlugin)
        // 1. Initialize state
        .init_state::<GameState>()
        // Resources
        .init_resource::<CurrentScene>()
        .init_resource::<GameInputMap>()
        // Startup systems
        // .add_systems(Startup, preload_assets)
        // 2. Add your module plugins
        .add_plugins((
            FrameTimeDiagnosticsPlugin::default(),
            InputManagerPlugin::<PlayerAction>::default(),
            DespawnPlugin,
            LoadingPlugin,
            MenuPlugin,
            GameplayPlugin,
            SettingsPlugin,
        ))
        .run();
}
