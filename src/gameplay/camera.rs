use bevy::prelude::*;
use bevy_ecs_tiled::prelude::TiledParallaxCamera;
use bevy_modern_pixel_camera::prelude::*;
use crate::common::{DespawnOnExit, MainCamera, Player};
use crate::states::GameState;

const CAMERA_LERP: f32 = 0.1;
const SCENE_BOUNDS_MIN: Vec2 = Vec2::new(-1000.0, -1000.0);
const SCENE_BOUNDS_MAX: Vec2 = Vec2::new(1000.0, 1000.0);

pub fn spawn_gameplay_camera(commands: &mut Commands) {
    let start_pos = Vec3::new(0.0, 6.0, 50.0);
    commands.spawn((
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(Color::srgb(0.05, 0.05, 0.08)),
            ..default()
        },
        MainCamera,
        TiledParallaxCamera,
        Msaa::Off,
        DespawnOnExit(GameState::Gameplay),
        Transform::from_translation(start_pos),
        GlobalTransform::default(),
        PixelZoom::FitWidth(320),
        PixelViewport,
    ));
}

pub fn camera_follow(
    mut cams: Query<&mut Transform, (With<MainCamera>, Without<Player>)>,
    player_q: Query<&Transform, (With<Player>, Without<MainCamera>)>,
) {
    if let Ok(player_tf) = player_q.single() {
        for mut cam_tf in cams.iter_mut() {
            let target = Vec3::new(
                player_tf.translation.x,
                cam_tf.translation.y,
                cam_tf.translation.z,
            );

            // Lerp update position
            cam_tf.translation = cam_tf.translation.lerp(target, CAMERA_LERP);

            // Clamp to boundaries
            cam_tf.translation.x = cam_tf
                .translation
                .x
                .clamp(SCENE_BOUNDS_MIN.x, SCENE_BOUNDS_MAX.x);
        }
    }
}

pub fn handle_camera_zoom(
    input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut PixelZoom, With<MainCamera>>,
) {
    let mut zoom_change = 0.0;
    if input.just_pressed(KeyCode::Minus) {
        zoom_change = 0.25;
    }
    if input.just_pressed(KeyCode::Equal) {
        zoom_change = -0.25;
    }

    if zoom_change != 0.0 {
        for mut pixel_zoom in &mut query {
            match *pixel_zoom {
                PixelZoom::FitSize { width, height } => {
                    let scale_factor = 1.0 + zoom_change;
                    let new_width = (width as f32 * scale_factor) as i32;
                    let new_height = (height as f32 * scale_factor) as i32;

                    // Clamp to reasonable limits
                    let new_width = new_width.clamp(160, 1280);
                    let new_height = new_height.clamp(90, 720);

                    *pixel_zoom = PixelZoom::FitSize {
                        width: new_width,
                        height: new_height,
                    };
                }
                PixelZoom::FitWidth(width) => {
                    let scale_factor = 1.0 + zoom_change;
                    let new_width = (width as f32 * scale_factor) as i32;
                    let new_width = new_width.clamp(160, 1280);
                    *pixel_zoom = PixelZoom::FitWidth(new_width);
                }
                _ => {}
            }
        }
    }
}
