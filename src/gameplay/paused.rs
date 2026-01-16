use crate::common::{DespawnOnExit, despawn_with};
use crate::states::{GameState, GameplayState};
use bevy::prelude::*;

pub struct PausedPlugin;

impl Plugin for PausedPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameplayState::Paused), setup_paused_screen)
            .add_systems(Update, paused_input.run_if(in_state(GameplayState::Paused)))
            .add_systems(
                OnExit(GameplayState::Paused),
                despawn_with(GameplayState::Paused),
            );
    }
}

fn setup_paused_screen(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Load font
    let font = asset_server.load("fonts/fusion-pixel-12px-proportional-zh_hans.ttf");

    // Title - centered with background block
    let title_mesh = meshes.add(Rectangle::new(350.0, 80.0));
    let pause_bg = materials.add(Color::srgb(0.15, 0.15, 0.15));

    commands.spawn((
        Mesh2d(title_mesh),
        MeshMaterial2d(pause_bg.clone()),
        Transform::from_xyz(0.0, 100.0, 0.0),
        DespawnOnExit(GameplayState::Paused),
    ));

    commands.spawn((
        Text2d::new("PAUSED"),
        TextFont {
            font: font.clone(),
            font_size: 60.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(0.0, 100.0, 1.0),
        DespawnOnExit(GameplayState::Paused),
    ));

    // Instructions - centered with background block
    let instr_mesh = meshes.add(Rectangle::new(400.0, 50.0));

    commands.spawn((
        Mesh2d(instr_mesh),
        MeshMaterial2d(pause_bg),
        Transform::from_xyz(0.0, 0.0, 0.0),
        DespawnOnExit(GameplayState::Paused),
    ));

    commands.spawn((
        Text2d::new("ESC to Resume | M for Menu"),
        TextFont {
            font: font.clone(),
            font_size: 24.0,
            ..default()
        },
        TextColor(Color::srgb(0.9, 0.9, 0.9)),
        Transform::from_xyz(0.0, 0.0 - 8.0, 1.0),
        DespawnOnExit(GameplayState::Paused),
    ));
}

fn paused_input(
    input: Res<ButtonInput<KeyCode>>,
    mut gameplay_state: ResMut<NextState<GameplayState>>,
    mut screen_state: ResMut<NextState<GameState>>,
) {
    if input.just_pressed(KeyCode::Escape) {
        // Resume gameplay
        gameplay_state.set(GameplayState::Playing);
    }

    if input.just_pressed(KeyCode::KeyM) {
        // Return to menu (this will also cleanup gameplay entities)
        screen_state.set(GameState::Menu);
    }
}
