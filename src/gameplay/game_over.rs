use crate::states::{GameState, GameplayState};
use bevy::prelude::*;

pub struct GameOverPlugin;

impl Plugin for GameOverPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameplayState::GameOver), setup_game_over_overlay)
            .add_systems(OnExit(GameplayState::GameOver), cleanup_game_over_overlay)
            .add_systems(
                Update,
                game_over_input.run_if(in_state(GameplayState::GameOver)),
            );
    }
}

#[derive(Component)]
struct GameOverOverlay;

fn setup_game_over_overlay(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/fusion-pixel-12px-proportional-zh_hans.ttf");

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)), // Semi-transparent black
            GameOverOverlay,
            ZIndex(100), // Ensure it's on top
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("YOU DIED"),
                TextFont {
                    font: font.clone(),
                    font_size: 80.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.0, 0.0)),
                Node {
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
            ));

            // Menu Button (Instruction)
            parent.spawn((
                Text::new("Press M for Menu"),
                TextFont {
                    font: font.clone(),
                    font_size: 30.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn cleanup_game_over_overlay(mut commands: Commands, query: Query<Entity, With<GameOverOverlay>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn game_over_input(
    input: Res<ButtonInput<KeyCode>>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut next_gameplay_state: ResMut<NextState<GameplayState>>,
) {
    if input.just_pressed(KeyCode::KeyM) {
        next_game_state.set(GameState::Menu);
        next_gameplay_state.set(GameplayState::Disabled);
    }
}
