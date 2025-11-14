use crate::components::*;
use crate::scene::GameAssets;
use bevy::prelude::*;

// ===== UI MARKERS =====

#[derive(Component)]
pub struct GameOverUIRoot;

#[derive(Component)]
pub struct PausedUIRoot;

#[derive(Component)]
pub struct PausedCamera;

// ===== STARTUP SYSTEMS =====

pub fn setup_camera(mut commands: Commands) {
    // Main UI camera for menus
    commands.spawn((
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(Color::srgb(0.1, 0.1, 0.1)),
            order: -1,
            ..default()
        },
        crate::menu::MainCamera,
    ));
}

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    game_assets: Res<GameAssets>,
) {
    // 2D camera
    commands.spawn((
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(Color::srgb(0.1, 0.1, 0.1)),
            ..default()
        },
    ));

    // Use preloaded assets (texture & atlas layout)
    let texture = game_assets.player_texture.clone();
    let texture_atlas_layout = game_assets.player_layout.clone();

    // Animation indices for Gabe sprite
    let animation_indices = AnimationIndices::new();

    // Player with animated sprite (24x24 sprite, scaled up)
    commands.spawn((
        Sprite::from_atlas_image(
            texture,
            TextureAtlas {
                layout: texture_atlas_layout,
                index: animation_indices.idle_first,
            },
        ),
        Transform::from_xyz(0.0, 200.0, 0.0).with_scale(Vec3::splat(4.0)),
        GlobalTransform::default(),
        Player,
        LevelEntity,
        Velocity {
            lin: Vec2::ZERO,
            on_ground: false,
            coyote: 0.0,
            jumps_left: MAX_JUMPS,
        },
        ColliderSize(Vec2::new(96.0, 96.0)), // 24 * 4 scale
        Health::new(100.0),
        animation_indices,
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        AnimationState::Idle,
    ));

    // Ground platform (800x30)
    let gray = materials.add(Color::srgb(0.4, 0.4, 0.45));
    let ground_mesh = meshes.add(Rectangle::new(800.0, 30.0));
    commands.spawn((
        Mesh2d(ground_mesh),
        MeshMaterial2d(gray),
        Transform::from_xyz(0.0, -150.0, 0.0),
        GlobalTransform::default(),
        Platform,
        LevelEntity,
        ColliderSize(Vec2::new(800.0, 30.0)),
    ));

    // Moving platform
    let p_mesh = meshes.add(Rectangle::new(200.0, 20.0));
    commands.spawn((
        Mesh2d(p_mesh.clone()),
        MeshMaterial2d(materials.add(Color::srgb(0.35, 0.6, 0.8))),
        Transform::from_xyz(-250.0, -20.0, 0.0),
        GlobalTransform::default(),
        Platform,
        LevelEntity,
        ColliderSize(Vec2::new(200.0, 20.0)),
        MovingPlatform {
            range_min: -350.0,
            range_max: -150.0,
            speed: 60.0,
            dir: 1.0,
        },
    ));

    // Static platforms
    for (x, y) in [(150.0, 40.0), (350.0, 120.0)] {
        commands.spawn((
            Mesh2d(p_mesh.clone()),
            MeshMaterial2d(materials.add(Color::srgb(0.35, 0.6, 0.8))),
            Transform::from_xyz(x, y, 0.0),
            GlobalTransform::default(),
            Platform,
            LevelEntity,
            ColliderSize(Vec2::new(200.0, 20.0)),
        ));
    }

    // Print controls to console as a simple HUD substitute
    println!("Left/Right: Arrow keys  •  Jump: Space  •  Double jump enabled");

    // HUD timer resource (for periodic console updates)
    commands.insert_resource(HudTimer(Timer::from_seconds(0.5, TimerMode::Repeating)));
}

pub fn cleanup_playing(
    mut commands: Commands,
    players: Query<Entity, With<Player>>,
    platforms: Query<Entity, With<Platform>>,
    cameras: Query<Entity, With<Camera>>,
) {
    // Remove all players
    for entity in players.iter() {
        commands.entity(entity).despawn();
    }
    // Remove all platforms
    for entity in platforms.iter() {
        commands.entity(entity).despawn();
    }
    // Remove all cameras
    for entity in cameras.iter() {
        commands.entity(entity).despawn();
    }
}

pub fn move_platforms(mut platforms: Query<(&mut Transform, &mut MovingPlatform), With<Platform>>, time: Res<Time>) {
    let dt = time.delta_secs();
    for (mut tf, mut mv) in platforms.iter_mut() {
        tf.translation.x += mv.dir * mv.speed * dt;
        if tf.translation.x < mv.range_min {
            tf.translation.x = mv.range_min;
            mv.dir = 1.0;
        } else if tf.translation.x > mv.range_max {
            tf.translation.x = mv.range_max;
            mv.dir = -1.0;
        }
    }
}

pub fn player_input(
    input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Velocity, With<Player>>,
) {
    if let Ok(mut vel) = query.single_mut() {
        // Horizontal movement using arrow keys
        let mut dir_x = 0.0;
        if input.pressed(KeyCode::ArrowLeft) {
            dir_x -= 1.0;
        }
        if input.pressed(KeyCode::ArrowRight) {
            dir_x += 1.0;
        }
        vel.lin.x = dir_x * PLAYER_SPEED;

        // Jump with coyote time and double jump
        if input.just_pressed(KeyCode::Space) {
            let can_jump = vel.on_ground || vel.coyote > 0.0 || vel.jumps_left > 0;
            if can_jump {
                vel.lin.y = JUMP_VELOCITY;
                if !vel.on_ground && vel.coyote <= 0.0 {
                    // mid-air jump consumes one extra
                    if vel.jumps_left > 0 {
                        vel.jumps_left -= 1;
                    }
                }
                vel.on_ground = false;
                vel.coyote = 0.0;
            }
        }
    }
}

use bevy::ecs::system::ParamSet;

pub fn physics(
    time: Res<Time>,
    mut params: ParamSet<(
        Query<(&mut Transform, &mut Velocity, &ColliderSize), With<Player>>,
        Query<(&Transform, Option<&MovingPlatform>, &ColliderSize), With<Platform>>,
    )>,
) {
    let dt = time.delta_secs();

    // snapshot platforms first to avoid double-borrowing ParamSet while iterating players
    let mut platform_snapshots: Vec<(Vec2, Option<(f32, f32)>, Vec2)> = Vec::new();
    for (p_transform, moving, p_size) in params.p1().iter() {
        platform_snapshots.push((
            Vec2::new(p_transform.translation.x, p_transform.translation.y),
            moving.map(|m| (m.dir, m.speed)),
            p_size.0,
        ));
    }

    // iterate players (mutable borrow)
    for (mut transform, mut vel, size) in params.p0().iter_mut() {
        // coyote timer
        if !vel.on_ground {
            vel.coyote -= dt;
        } else {
            vel.coyote = COYOTE_TIME;
        }

        // apply gravity
        vel.lin.y -= GRAVITY * dt;

        // integrate desired position
        let mut pos = transform.translation;
        pos.x += vel.lin.x * dt;
        pos.y += vel.lin.y * dt;

        // reset grounded state; will set true if collision from above
        vel.on_ground = false;

        // AABB collision resolution with platform snapshots
        let a_pos = Vec2::new(pos.x, pos.y);
        let a_half = size.0 / 2.0;
        for (b_pos, moving_info, b_size) in platform_snapshots.iter() {
            let b_half = *b_size / 2.0;

            let delta = *b_pos - a_pos;
            let overlap = Vec2::new(a_half.x + b_half.x - delta.x.abs(), a_half.y + b_half.y - delta.y.abs());

            if overlap.x > 0.0 && overlap.y > 0.0 {
                // collision detected; resolve along smaller overlap
                if overlap.x < overlap.y {
                    // push in X
                    let sign = delta.x.signum();
                    pos.x -= sign * overlap.x;
                    vel.lin.x = 0.0;
                } else {
                    // push in Y
                    let sign = delta.y.signum();
                    pos.y -= sign * overlap.y;
                    vel.lin.y = 0.0;

                    // if sign < 0, player was above platform and moved up onto it => landed
                    if sign < 0.0 {
                        vel.on_ground = true;
                        vel.coyote = COYOTE_TIME;
                        vel.jumps_left = MAX_JUMPS;
                        // if platform is moving, inherit its horizontal velocity
                        if let Some((dir, speed)) = moving_info {
                            vel.lin.x += dir * speed;
                        }
                    }
                }
            }
        }

        transform.translation = Vec3::new(pos.x, pos.y, transform.translation.z);
    }
}

pub fn camera_follow(mut cams: Query<&mut Transform, (With<Camera>, Without<Player>)>, player_q: Query<&Transform, With<Player>>) {
    if let Ok(player_tf) = player_q.single() {
        for mut cam_tf in cams.iter_mut() {
            let target = Vec3::new(player_tf.translation.x, player_tf.translation.y, cam_tf.translation.z);
            // lerp
            cam_tf.translation = cam_tf.translation.lerp(target, CAMERA_LERP);
            // clamp
            cam_tf.translation.x = cam_tf.translation.x.clamp(LEVEL_MIN.x, LEVEL_MAX.x);
            cam_tf.translation.y = cam_tf.translation.y.clamp(LEVEL_MIN.y, LEVEL_MAX.y);
        }
    }
}

pub fn hud_health(mut timer: ResMut<HudTimer>, time: Res<Time>, player_q: Query<(&Health, &Velocity), With<Player>>) {
    timer.0.tick(time.delta());
    if timer.0.just_finished() {
        if let Ok((health, vel)) = player_q.single() {
            let health_bar = if health.health_percent() > 0.0 {
                let filled = (health.health_percent() * 20.0) as usize;
                let empty = 20 - filled;
                format!("[{}{}]", "=".repeat(filled), " ".repeat(empty))
            } else {
                "[DEAD]".to_string()
            };
            println!("Health: {} ({:.1}/{:.1}) | Jumps: {} | Status: {}",
                health_bar,
                health.current,
                health.max,
                vel.jumps_left,
                if health.is_alive() { "ALIVE" } else { "DEAD" }
            );
        }
    }
}

// ===== PAUSE SYSTEMS =====

pub fn pause_input(
    input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if input.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::Paused);
    }
}

pub fn setup_paused_menu(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Load font
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");

    // Title - centered with background block
    let title_mesh = meshes.add(Rectangle::new(350.0, 80.0));
    let pause_bg = materials.add(Color::srgb(0.15, 0.15, 0.15));
    
    commands.spawn((
        Mesh2d(title_mesh),
        MeshMaterial2d(pause_bg.clone()),
        Transform::from_xyz(0.0, 100.0, 0.0),
        PausedUIRoot,
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
        PausedUIRoot,
    ));

    // Instructions - centered with background block
    let instr_mesh = meshes.add(Rectangle::new(400.0, 50.0));
    
    commands.spawn((
        Mesh2d(instr_mesh),
        MeshMaterial2d(pause_bg),
        Transform::from_xyz(0.0, 0.0, 0.0),
        PausedUIRoot,
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
        PausedUIRoot,
    ));
}

pub fn cleanup_paused_ui(mut commands: Commands, query: Query<Entity, With<PausedUIRoot>>, camera_query: Query<Entity, With<PausedCamera>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in camera_query.iter() {
        commands.entity(entity).despawn();
    }
}

pub fn paused_input(
    input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if input.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::Playing);
    }

    if input.just_pressed(KeyCode::KeyM) {
        next_state.set(GameState::Menu);
    }
}

// ===== GAME OVER SYSTEMS =====

pub fn check_game_over(
    player_q: Query<&Health, With<Player>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if let Ok(health) = player_q.single() {
        if !health.is_alive() {
            next_state.set(GameState::GameOver);
        }
    }
}

pub fn setup_game_over_menu(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Load font
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");

    // Title - centered
    commands.spawn((
        Text2d::new("GAME OVER"),
        TextFont {
            font: font.clone(),
            font_size: 60.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.0, 0.0)),
        Transform::from_xyz(0.0, 100.0, 0.0),
        GameOverUIRoot,
    ));

    // Subtitle - centered with background block
    let bg_red = materials.add(Color::srgb(0.4, 0.0, 0.0));
    let subtitle_mesh = meshes.add(Rectangle::new(300.0, 50.0));
    
    commands.spawn((
        Mesh2d(subtitle_mesh.clone()),
        MeshMaterial2d(bg_red.clone()),
        Transform::from_xyz(0.0, 20.0, 0.0),
        GameOverUIRoot,
    ));
    
    commands.spawn((
        Text2d::new("You Died!"),
        TextFont {
            font: font.clone(),
            font_size: 40.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.5, 0.5)),
        Transform::from_xyz(0.0, 20.0 - 10.0, 1.0),
        GameOverUIRoot,
    ));

    // Instructions - centered with background block
    let instructions_mesh = meshes.add(Rectangle::new(400.0, 50.0));
    
    commands.spawn((
        Mesh2d(instructions_mesh),
        MeshMaterial2d(bg_red),
        Transform::from_xyz(0.0, -80.0, 0.0),
        GameOverUIRoot,
    ));
    
    commands.spawn((
        Text2d::new("R to Restart | M for Menu"),
        TextFont {
            font: font.clone(),
            font_size: 24.0,
            ..default()
        },
        TextColor(Color::srgb(0.9, 0.9, 0.9)),
        Transform::from_xyz(0.0, -80.0 - 8.0, 1.0),
        GameOverUIRoot,
    ));
}

pub fn cleanup_game_over_ui(mut commands: Commands, query: Query<Entity, With<GameOverUIRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

pub fn game_over_input(
    input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if input.just_pressed(KeyCode::KeyR) {
        next_state.set(GameState::Playing);
    }

    if input.just_pressed(KeyCode::KeyM) {
        next_state.set(GameState::Menu);
    }
}
