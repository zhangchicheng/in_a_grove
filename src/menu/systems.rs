use super::components::*;
use crate::common::{
    CurrentScene, DespawnOnExit, KeyboardSelectable, MenuAudioAssets, SelectedButtonIndex,
};
use crate::states::GameState;
use bevy::prelude::*;

const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const HOVERED_PRESSED_BUTTON: Color = Color::srgb(0.25, 0.65, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

pub fn menu_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Camera for rendering the menu UI
    commands.spawn((
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(Color::srgb(0.1, 0.1, 0.1)),
            ..default()
        },
        DespawnOnExit(GameState::Menu),
    ));

    // Load menu audio assets
    commands.insert_resource(MenuAudioAssets {
        menu_blip: asset_server.load("sounds/menu_blip.wav"),
    });

    spawn_main_menu(&mut commands, &asset_server);
}

fn spawn_main_menu(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    let font = asset_server.load("fonts/fusion-pixel-12px-proportional-zh_hans.ttf");

    // [优化 1] 提取通用样式
    let button_node_style = Node {
        width: Val::Px(300.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };

    let button_text_font = TextFont {
        font: font.clone(),
        font_size: 33.0,
        ..default()
    };

    // [优化 2] 辅助闭包：减少重复代码
    // Inlined to avoid ChildBuilder type issues


    commands
        .spawn((
            DespawnOnExit(GameState::Menu),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            OnMainMenuScreen,
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
                    // Title
                    parent.spawn((
                        Text::new("我的游戏"),
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

                    // Buttons (Inlined)
                    // Start
                    parent
                        .spawn((
                            Button,
                            button_node_style.clone(),
                            BackgroundColor(NORMAL_BUTTON),
                            MenuButtonAction::Play,
                            KeyboardSelectable { index: 0 },
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("开始"),
                                button_text_font.clone(),
                                TextColor(TEXT_COLOR),
                            ));
                        });

                    // Settings
                    parent
                        .spawn((
                            Button,
                            button_node_style.clone(),
                            BackgroundColor(NORMAL_BUTTON),
                            MenuButtonAction::Settings,
                            KeyboardSelectable { index: 1 },
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("设置"),
                                button_text_font.clone(),
                                TextColor(TEXT_COLOR),
                            ));
                        });

                    // Quit
                    parent
                        .spawn((
                            Button,
                            button_node_style.clone(),
                            BackgroundColor(NORMAL_BUTTON),
                            MenuButtonAction::Quit,
                            KeyboardSelectable { index: 2 },
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("退出"),
                                button_text_font.clone(),
                                TextColor(TEXT_COLOR),
                            ));
                        });
                });
        });
}

/// [优化 3] 整合导航系统：同时支持 键盘 和 手柄 (Gamepad)
/// 原名 keyboard_navigation
pub fn input_navigation(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    gamepad_input: Option<Res<ButtonInput<GamepadButton>>>, // 新增手柄输入
    mut selected_index: ResMut<SelectedButtonIndex>,
    audio_assets: Res<MenuAudioAssets>,
    mut commands: Commands,
    button_query: Query<&KeyboardSelectable, With<Button>>,
) {
    let max = button_query.iter().map(|k| k.index).max().unwrap_or(0);
    selected_index.max_index = max;

    let mut changed = false;

    // Up
    let up_pressed = keyboard_input.just_pressed(KeyCode::ArrowUp)
        || keyboard_input.just_pressed(KeyCode::KeyW)
        || gamepad_input.as_ref().map_or(false, |input| input.get_just_pressed().any(|btn| *btn == GamepadButton::DPadUp));

    if up_pressed && selected_index.index > 0 {
        selected_index.index -= 1;
        changed = true;
    }

    // Down
    let down_pressed = keyboard_input.just_pressed(KeyCode::ArrowDown)
        || keyboard_input.just_pressed(KeyCode::KeyS)
        || gamepad_input.as_ref().map_or(false, |input| input.get_just_pressed().any(|btn| *btn == GamepadButton::DPadDown));

    if down_pressed && selected_index.index < selected_index.max_index {
        selected_index.index += 1;
        changed = true;
    }

    if changed {
        commands.spawn((
            AudioPlayer::new(audio_assets.menu_blip.clone()),
            PlaybackSettings::DESPAWN,
        ));
    }
}

/// [优化 4] 按钮交互系统：处理颜色变化 + 鼠标夺权
/// 原名 button_system
#[allow(clippy::type_complexity)]
pub fn button_interaction_system(
    mut selected_index: ResMut<SelectedButtonIndex>, // 改为 ResMut
    interaction_query: Query<
        (
            &Interaction,
            &KeyboardSelectable, // 移除 Option，必须有这个组件才参与逻辑
        ),
        (With<Button>, Changed<Interaction>), // 只有状态改变时才运行，优化性能
    >,
    // 为了非 Changed 的情况（键盘导航导致颜色变化），我们需要第二个 Query
    // 或者我们稍微改一下逻辑，每一帧都跑颜色设置，但只在 Changed<Interaction> 时更新 Index
    mut all_buttons_query: Query<(
        &Interaction,
        &mut BackgroundColor,
        Option<&SelectedOption>,
        &KeyboardSelectable,
    ), With<Button>>,
) {
    // 1. 鼠标悬停逻辑：如果鼠标指到了按钮，强制更新选中的 Index
    for (interaction, selectable) in &interaction_query {
        if *interaction == Interaction::Hovered {
            selected_index.index = selectable.index;
        }
    }

    // 2. 视觉更新逻辑：根据当前的 Index 设置颜色
    for (interaction, mut background_color, selected, selectable) in &mut all_buttons_query {
        let is_selected = selectable.index == selected_index.index;

        *background_color = match (*interaction, selected, is_selected) {
            (Interaction::Pressed, _, _) | (Interaction::None, Some(_), _) => PRESSED_BUTTON.into(),
            (Interaction::Hovered, Some(_), _) => HOVERED_PRESSED_BUTTON.into(),
            
            // 鼠标悬停 OR 键盘选中 都是 Hover 颜色
            (Interaction::Hovered, None, _) | (Interaction::None, None, true) => HOVERED_BUTTON.into(),
            
            (Interaction::None, None, false) => NORMAL_BUTTON.into(),
        };
    }
}

pub fn menu_action(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    gamepad_input: Option<Res<ButtonInput<GamepadButton>>>, // 新增手柄
    selected_index: Res<SelectedButtonIndex>,
    interaction_query: Query<
        (&Interaction, &MenuButtonAction, &KeyboardSelectable),
        With<Button>,
    >,
    mut screen_state: ResMut<NextState<GameState>>,
    mut current_scene: ResMut<CurrentScene>,
    mut exit: MessageWriter<bevy::app::AppExit>,
) {
    // Check for confirmation keys (Keyboard or Gamepad South/A)
    let confirm_pressed = keyboard_input.just_pressed(KeyCode::Enter)
        || keyboard_input.just_pressed(KeyCode::Space)
        || gamepad_input.as_ref().map_or(false, |input| input.get_just_pressed().any(|btn| *btn == GamepadButton::South));

    for (interaction, menu_button_action, selectable) in &interaction_query {
        let mouse_clicked = *interaction == Interaction::Pressed;
        let is_selected = selectable.index == selected_index.index;

        if mouse_clicked || (confirm_pressed && is_selected) {
            match menu_button_action {
                MenuButtonAction::Quit => {
                    exit.write(bevy::app::AppExit::Success);
                }
                MenuButtonAction::Play => {
                    *current_scene = CurrentScene::Prologue;
                    screen_state.set(GameState::Gameplay);
                }
                MenuButtonAction::Settings => {
                    screen_state.set(GameState::Settings);
                }
            }
        }
    }
}
