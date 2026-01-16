use crate::common::{Health, HudRoot, HealthBarFill, HealthText, DespawnOnExit, Player};
use crate::states::GameState;
use bevy::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (spawn_health_bars, update_health_bars).run_if(in_state(GameState::Gameplay)),
        );
    }
}

#[derive(Component)]
pub struct HealthBar;

#[derive(Component)]
pub struct HealthBarContainer;

fn spawn_health_bars(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    query: Query<(Entity, &Health), Without<HealthBarContainer>>,
) {
    for (entity, _health) in query.iter() {
        let bar_width = 30.0;
        let bar_height = 4.0;
        let y_offset = 25.0;

        // Create mesh and materials
        let mesh = meshes.add(Rectangle::new(bar_width, bar_height));
        let bg_material = materials.add(Color::srgb(0.2, 0.0, 0.0));
        let fg_material = materials.add(Color::srgb(0.0, 1.0, 0.0));

        commands.entity(entity).with_children(|parent| {
            // Background (Dark Red)
            parent.spawn((
                Mesh2d(mesh.clone()),
                MeshMaterial2d(bg_material),
                Transform::from_translation(Vec3::new(0.0, y_offset, 10.0)),
            ));

            // Foreground (Green)
            parent.spawn((
                Mesh2d(mesh),
                MeshMaterial2d(fg_material),
                Transform::from_translation(Vec3::new(0.0, y_offset, 11.0)),
                HealthBar,
            ));
        });

        commands.entity(entity).insert(HealthBarContainer);
    }
}

fn update_health_bars(
    mut query: Query<(&mut Transform, &MeshMaterial2d<ColorMaterial>, &ChildOf), With<HealthBar>>,
    health_query: Query<&Health>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (mut transform, handle, parent) in query.iter_mut() {
        if let Ok(health) = health_query.get(parent.0) {
            let percent = (health.current / health.max).clamp(0.0, 1.0);

            // Scale x
            transform.scale.x = percent;

            // Adjust position to keep left anchor
            let bar_width = 30.0;
            transform.translation.x = -bar_width / 2.0 + (bar_width * percent / 2.0);

            // Color
            if let Some(material) = materials.get_mut(handle) {
                if percent < 0.3 {
                    material.color = Color::srgb(1.0, 0.0, 0.0);
                } else if percent < 0.6 {
                    material.color = Color::srgb(1.0, 1.0, 0.0);
                } else {
                    material.color = Color::srgb(0.0, 1.0, 0.0);
                }
            }
        }
    }
}

/// 检查鼠标是否悬停在任何 UI 元素上
/// 用于在 GameplayPlugin 中阻断游戏输入
pub fn is_cursor_on_ui(
    // 查询所有带有 Interaction 组件（按钮等）的 UI 节点
    ui_interaction_query: Query<&Interaction, With<Node>>,
) -> bool {
    // 只要有一个 UI 处于 Hovered 或 Pressed 状态，就返回 true
    ui_interaction_query.iter().any(|i| *i != Interaction::None)
}

pub fn setup_hud(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/fusion-pixel-12px-proportional-zh_hans.ttf");

    // HUD root container - top-left positioned
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(20.0),
                top: Val::Px(20.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                ..default()
            },
            HudRoot,
            DespawnOnExit(GameState::Gameplay),
        ))
        .with_children(|parent| {
            // Health label
            parent.spawn((
                Text::new("HEALTH"),
                TextFont {
                    font: font.clone(),
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
            ));

            // Health bar container (background)
            parent
                .spawn(Node {
                    width: Val::Px(200.0),
                    height: Val::Px(24.0),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                })
                .insert(BackgroundColor(Color::srgb(0.2, 0.2, 0.2)))
                .with_children(|parent| {
                    // Health bar fill (will be updated based on health %)
                    parent.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.8, 0.2, 0.2)),
                        HealthBarFill,
                    ));
                });

            // Health text (numerical display)
            parent.spawn((
                Text::new("100 / 100"),
                TextFont {
                    font: font.clone(),
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                HealthText,
            ));
        });
}

pub fn hud_health(
    player_q: Query<&Health, With<Player>>,
    mut health_bar_q: Query<(&mut Node, &mut BackgroundColor), With<HealthBarFill>>,
    mut health_text_q: Query<&mut Text, With<HealthText>>,
) {
    if let Ok(health) = player_q.single() {
        // Update health bar width and color based on health percentage
        if let Ok((mut node, mut bg_color)) = health_bar_q.single_mut() {
            let health_percent = health.health_percent();
            node.width = Val::Percent(health_percent * 100.0);

            // Color interpolation: Green -> Yellow -> Red as health decreases
            let color = if health_percent > 0.5 {
                // Green to Yellow (100% -> 50%)
                let t = (1.0 - health_percent) * 2.0; // 0.0 -> 1.0
                Color::srgb(0.2 + t * 0.8, 0.8, 0.2)
            } else {
                // Yellow to Red (50% -> 0%)
                let t = health_percent * 2.0; // 1.0 -> 0.0
                Color::srgb(1.0, t * 0.8, 0.2)
            };
            *bg_color = BackgroundColor(color);
        }

        // Update health text
        if let Ok(mut text) = health_text_q.single_mut() {
            text.0 = format!("{:.0} / {:.0}", health.current, health.max);
        }
    }
}
