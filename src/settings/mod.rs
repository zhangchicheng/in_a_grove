use crate::common::input::{GameInputMap, PlayerAction};
use crate::common::{DespawnOnExit, KeyboardSelectable, MenuAudioAssets, SelectedButtonIndex};
use crate::states::GameState;
use bevy::prelude::*;

const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const HOVERED_PRESSED_BUTTON: Color = Color::srgb(0.25, 0.65, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

#[derive(Component)]
struct OnSettingsScreen;

#[derive(Component)]
struct SelectedOption;

#[derive(Component)]
enum SettingsButtonAction {
    BackToMainMenu,
}

#[derive(Resource, Default)]
struct RemappingState {
    action: Option<PlayerAction>,
}

#[derive(Component)]
struct BindingButton {
    action: PlayerAction,
}

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RemappingState>()
            .add_systems(OnEnter(GameState::Settings), settings_setup)
            .add_systems(
                Update,
                (
                    keyboard_navigation,
                    button_system,
                    settings_action,
                    binding_action,
                    remapping_system,
                    update_binding_display,
                )
                    .run_if(in_state(GameState::Settings)),
            );
    }
}

fn settings_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut selected_index: ResMut<SelectedButtonIndex>,
    input_map: Res<GameInputMap>,
) {
    selected_index.index = 0;

    commands.spawn((
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(Color::srgb(0.1, 0.1, 0.1)),
            ..default()
        },
        DespawnOnExit(GameState::Settings),
    ));

    commands.insert_resource(MenuAudioAssets {
        menu_blip: asset_server.load("sounds/menu_blip.wav"),
    });

    let font = asset_server.load("fonts/fusion-pixel-12px-proportional-zh_hans.ttf");

    let button_node = Node {
        width: Val::Px(200.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };

    let button_text_style = (
        TextFont {
            font: font.clone(),
            font_size: 33.0,
            ..default()
        },
        TextColor(TEXT_COLOR),
    );

    commands
        .spawn((
            DespawnOnExit(GameState::Settings),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            OnSettingsScreen,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("SETTINGS"),
                        TextFont {
                            font: font.clone(),
                            font_size: 67.0,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                        Node {
                            margin: UiRect::all(Val::Px(50.0)),
                            ..default()
                        },
                    ));
                    parent.spawn((
                        Text::new("Volume: 100%"),
                        TextFont {
                            font: font.clone(),
                            font_size: 40.0,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                        Node {
                            margin: UiRect::all(Val::Px(20.0)),
                            ..default()
                        },
                    ));
                    parent.spawn((
                        Text::new("Difficulty: Normal"),
                        TextFont {
                            font: font.clone(),
                            font_size: 40.0,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                        Node {
                            margin: UiRect::all(Val::Px(20.0)),
                            ..default()
                        },
                    ));

                    parent.spawn((
                        Text::new("CONTROLS"),
                        TextFont {
                            font: font.clone(),
                            font_size: 40.0,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                        Node {
                            margin: UiRect::all(Val::Px(20.0)),
                            ..default()
                        },
                    ));

                    let actions = [
                        PlayerAction::MoveLeft,
                        PlayerAction::MoveRight,
                        PlayerAction::MoveUp,
                        PlayerAction::MoveDown,
                        PlayerAction::Jump,
                        PlayerAction::Attack1,
                        PlayerAction::Attack2,
                        PlayerAction::Attack3,
                        PlayerAction::ThrowSpear,
                        PlayerAction::Parry,
                        PlayerAction::Menu,
                        PlayerAction::Pause,
                    ];

                    let mut current_index = 0;
                    for action in actions {
                        let action_name = format!("{:?}", action);

                        parent
                            .spawn(Node {
                                width: Val::Px(500.0),
                                justify_content: JustifyContent::SpaceBetween,
                                align_items: AlignItems::Center,
                                margin: UiRect::all(Val::Px(5.0)),
                                ..default()
                            })
                            .with_children(|parent| {
                                parent.spawn((
                                    Text::new(action_name),
                                    TextFont {
                                        font: font.clone(),
                                        font_size: 20.0,
                                        ..default()
                                    },
                                    TextColor(TEXT_COLOR),
                                ));

                                parent
                                    .spawn((
                                        Button,
                                        BindingButton { action },
                                        KeyboardSelectable {
                                            index: current_index,
                                        },
                                        Node {
                                            width: Val::Px(150.0),
                                            justify_content: JustifyContent::Center,
                                            padding: UiRect::all(Val::Px(10.0)),
                                            ..default()
                                        },
                                        BackgroundColor(NORMAL_BUTTON),
                                    ))
                                    .with_children(|parent| {
                                        let key_name =
                                            input_map.0.get(&action).and_then(|inputs| {
                                                inputs.first().and_then(extract_key_name)
                                            });

                                        let mut spawned_icon = false;
                                        if let Some(ref k) = key_name
                                            && let Some(path) = get_key_icon_path(k)
                                        {
                                            parent.spawn((
                                                ImageNode::new(asset_server.load(path)),
                                                Node {
                                                    width: Val::Px(32.0),
                                                    height: Val::Px(32.0),
                                                    ..default()
                                                },
                                            ));
                                            spawned_icon = true;
                                        }

                                        if !spawned_icon {
                                            let text = if let Some(k) = key_name {
                                                k.strip_prefix("Key").unwrap_or(&k).to_string()
                                            } else {
                                                "None".to_string()
                                            };

                                            parent.spawn((
                                                Text::new(text),
                                                TextFont {
                                                    font: font.clone(),
                                                    font_size: 20.0,
                                                    ..default()
                                                },
                                                TextColor(TEXT_COLOR),
                                            ));
                                        }
                                    });
                            });
                        current_index += 1;
                    }
                    parent
                        .spawn((
                            Button,
                            button_node,
                            BackgroundColor(NORMAL_BUTTON),
                            SettingsButtonAction::BackToMainMenu,
                            KeyboardSelectable {
                                index: current_index,
                            },
                        ))
                        .with_children(|parent| {
                            parent.spawn((Text::new("Back"), button_text_style));
                        });
                });
        });
}

fn keyboard_navigation(
    input: Res<ButtonInput<KeyCode>>,
    mut selected_index: ResMut<SelectedButtonIndex>,
    audio_assets: Option<Res<MenuAudioAssets>>,
    mut commands: Commands,
    button_query: Query<&KeyboardSelectable, With<Button>>,
) {
    let max = button_query.iter().map(|k| k.index).max().unwrap_or(0);
    selected_index.max_index = max;

    if (input.just_pressed(KeyCode::ArrowUp) || input.just_pressed(KeyCode::KeyW))
        && selected_index.index > 0
    {
        selected_index.index -= 1;
        if let Some(ref audio) = audio_assets {
            commands.spawn((
                AudioPlayer::new(audio.menu_blip.clone()),
                PlaybackSettings::DESPAWN,
            ));
        }
    }

    if (input.just_pressed(KeyCode::ArrowDown) || input.just_pressed(KeyCode::KeyS))
        && selected_index.index < selected_index.max_index
    {
        selected_index.index += 1;
        if let Some(ref audio) = audio_assets {
            commands.spawn((
                AudioPlayer::new(audio.menu_blip.clone()),
                PlaybackSettings::DESPAWN,
            ));
        }
    }
}

#[allow(clippy::type_complexity)]
fn button_system(
    selected_index: Res<SelectedButtonIndex>,
    remapping_state: Res<RemappingState>,
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            Option<&SelectedOption>,
            Option<&KeyboardSelectable>,
            Option<&BindingButton>,
        ),
        With<Button>,
    >,
) {
    for (interaction, mut background_color, selected, keyboard_selectable, binding_button) in
        &mut interaction_query
    {
        let is_keyboard_selected = keyboard_selectable
            .map(|k| k.index == selected_index.index)
            .unwrap_or(false);

        let is_remapping = binding_button
            .map(|b| Some(b.action) == remapping_state.action)
            .unwrap_or(false);

        *background_color = if is_remapping {
            Color::srgb(0.8, 0.2, 0.2).into()
        } else {
            match (*interaction, selected, is_keyboard_selected) {
                (Interaction::Pressed, _, _) | (Interaction::None, Some(_), _) => {
                    PRESSED_BUTTON.into()
                }
                (Interaction::Hovered, Some(_), _) => HOVERED_PRESSED_BUTTON.into(),
                (Interaction::Hovered, None, _) => HOVERED_BUTTON.into(),
                (Interaction::None, None, true) => HOVERED_BUTTON.into(),
                (Interaction::None, None, false) => NORMAL_BUTTON.into(),
            }
        };
    }
}

fn settings_action(
    input: Res<ButtonInput<KeyCode>>,
    selected_index: Res<SelectedButtonIndex>,
    interaction_query: Query<
        (
            &Interaction,
            &SettingsButtonAction,
            Option<&KeyboardSelectable>,
        ),
        With<Button>,
    >,
    mut screen_state: ResMut<NextState<GameState>>,
) {
    let keyboard_activate =
        input.just_pressed(KeyCode::Enter) || input.just_pressed(KeyCode::Space);

    for (interaction, settings_button_action, keyboard_selectable) in &interaction_query {
        let mouse_activated = *interaction == Interaction::Pressed;
        let keyboard_activated = keyboard_activate
            && keyboard_selectable
                .map(|k| k.index == selected_index.index)
                .unwrap_or(false);

        if mouse_activated || keyboard_activated {
            match settings_button_action {
                SettingsButtonAction::BackToMainMenu => {
                    screen_state.set(GameState::Menu);
                }
            }
        }
    }
}

fn binding_action(
    mut input: ResMut<ButtonInput<KeyCode>>,
    selected_index: Res<SelectedButtonIndex>,
    interaction_query: Query<
        (&Interaction, &BindingButton, Option<&KeyboardSelectable>),
        With<Button>,
    >,
    mut remapping_state: ResMut<RemappingState>,
) {
    let enter_pressed = input.just_pressed(KeyCode::Enter);
    let space_pressed = input.just_pressed(KeyCode::Space);
    let keyboard_activate = enter_pressed || space_pressed;

    for (interaction, binding_button, keyboard_selectable) in &interaction_query {
        let mouse_activated = *interaction == Interaction::Pressed;
        let keyboard_activated = keyboard_activate
            && keyboard_selectable
                .map(|k| k.index == selected_index.index)
                .unwrap_or(false);

        if mouse_activated || keyboard_activated {
            remapping_state.action = Some(binding_button.action);
            if keyboard_activated {
                if enter_pressed {
                    input.reset(KeyCode::Enter);
                }
                if space_pressed {
                    input.reset(KeyCode::Space);
                }
            }
        }
    }
}

fn remapping_system(
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut input_map: ResMut<GameInputMap>,
    mut remapping_state: ResMut<RemappingState>,
) {
    if let Some(action) = remapping_state.action {
        let key_opt = keys.get_just_pressed().next().cloned();
        if let Some(key) = key_opt {
            input_map.0.clear_action(&action);
            input_map.0.insert(action, key);
            remapping_state.action = None;
            keys.reset(key);
        }
    }
}

fn update_binding_display(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    input_map: Res<GameInputMap>,
    button_query: Query<(Entity, &BindingButton)>,
    children_query: Query<&Children>,
) {
    if input_map.is_changed() {
        let font_handle = asset_server.load("fonts/fusion-pixel-12px-proportional-zh_hans.ttf");

        for (entity, button) in &button_query {
            if let Ok(children) = children_query.get(entity) {
                for &child in children {
                    commands.entity(child).despawn();
                }
            }

            let key_name = input_map
                .0
                .get(&button.action)
                .and_then(|inputs| inputs.first().and_then(extract_key_name));

            commands.entity(entity).with_children(|parent| {
                let mut spawned_icon = false;
                if let Some(ref k) = key_name
                    && let Some(path) = get_key_icon_path(k)
                {
                    parent.spawn((
                        ImageNode::new(asset_server.load(path)),
                        Node {
                            width: Val::Px(32.0),
                            height: Val::Px(32.0),
                            ..default()
                        },
                    ));
                    spawned_icon = true;
                }

                if !spawned_icon {
                    let text = if let Some(k) = key_name {
                        k.strip_prefix("Key").unwrap_or(&k).to_string()
                    } else {
                        "None".to_string()
                    };

                    parent.spawn((
                        Text::new(text),
                        TextFont {
                            font: font_handle.clone(),
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                    ));
                }
            });
        }
    }
}

fn extract_key_name(input: &impl std::fmt::Debug) -> Option<String> {
    let s = format!("{:?}", input);
    // Try to find "Keyboard(KeyName)"
    if let Some(start) = s.find("Keyboard(") {
        let rest = &s[start + 9..];
        if let Some(end) = rest.find(')') {
            return Some(rest[..end].to_string());
        }
    }
    // Try to find "Button(KeyName)" - leafwing 0.19 seems to use this for keys sometimes
    if let Some(start) = s.find("Button(") {
        let rest = &s[start + 7..];
        if let Some(end) = rest.find(')') {
            return Some(rest[..end].to_string());
        }
    }
    None
}

fn get_key_icon_path(key_name: &str) -> Option<String> {
    let id = match key_name {
        "Digit1" => 1,
        "Digit2" => 2,
        "Digit3" => 3,
        "Digit4" => 4,
        "Digit5" => 5,
        "Digit6" => 6,
        "Digit7" => 7,
        "Digit8" => 8,
        "Digit9" => 9,
        "Digit0" => 10,
        "Minus" => 11,
        "Equal" => 12,
        "KeyQ" => 13,
        "KeyW" => 14,
        "KeyE" => 15,
        "KeyR" => 16,
        "KeyT" => 17,
        "KeyY" => 18,
        "KeyU" => 19,
        "KeyI" => 20,
        "KeyO" => 21,
        "KeyP" => 22,
        "BracketLeft" => 23,
        "BracketRight" => 24,
        "KeyA" => 25,
        "KeyS" => 26,
        "KeyD" => 27,
        "KeyF" => 28,
        "KeyG" => 29,
        "KeyH" => 30,
        "KeyJ" => 31,
        "KeyK" => 32,
        "KeyL" => 33,
        "Semicolon" => 34,
        "Quote" => 35,
        "KeyZ" => 37,
        "KeyX" => 38,
        "KeyC" => 39,
        "KeyV" => 40,
        "KeyB" => 41,
        "KeyN" => 42,
        "KeyM" => 43,
        "Comma" => 44,
        "Period" => 45,
        "Slash" => 46,
        "Space" => 59,
        "Enter" => 48,
        "Escape" => 49,
        "ArrowUp" => 83,
        "ArrowDown" => 84,
        "ArrowLeft" => 85,
        "ArrowRight" => 86,
        _ => return None,
    };
    Some(format!("textures/keyboard/keyboard_{}.png", id))
}
