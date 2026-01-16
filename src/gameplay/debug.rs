use bevy::prelude::*;
use bevy_aseprite_ultra::prelude::*;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (draw_debug_hitboxes, debug_print_aseprite_info));
    }
}

fn debug_print_aseprite_info(
    mut events: MessageReader<AssetEvent<Aseprite>>,
    aseprites: Res<Assets<Aseprite>>,
) {
    for event in events.read() {
        if let AssetEvent::Added { id } = event
            && let Some(aseprite) = aseprites.get(*id)
        {
            info!("=== Aseprite Asset Loaded [{:?}] ===", id);

            info!("--- Tags ---");
            for (name, tag) in &aseprite.tags {
                info!("Tag: {} -> Range: {:?}", name, tag.range);
            }

            info!("--- Slices ---");
            for (name, slice) in &aseprite.slices {
                info!("Slice: {} -> Rect: {:?}", name, slice.rect);
            }
            info!("=====================================");
        }
    }
}

fn draw_debug_hitboxes(
    mut gizmos: Gizmos,
    query: Query<(Entity, &GlobalTransform, &AseAnimation, &Sprite)>,
    aseprites: Res<Assets<Aseprite>>,
    layouts: Res<Assets<TextureAtlasLayout>>,
) {
    for (_entity, transform, anim, sprite) in &query {
        if let Some(aseprite) = aseprites.get(&anim.aseprite) {
            // Try to get the canvas size from the atlas layout (using max frame size as approximation)
            let canvas_size = if let Some(layout) = layouts.get(&aseprite.atlas_layout) {
                layout
                    .textures
                    .iter()
                    .map(|r| r.size())
                    .reduce(|a, b| a.max(b))
                    .map(|s| s.as_vec2())
                    .unwrap_or(Vec2::ZERO)
            } else {
                Vec2::ZERO
            };

            // Use per-frame slice data from Aseprite asset
            if let Some(atlas) = &sprite.texture_atlas {
                let frame_idx = atlas.index;
                // info!("Entity {:?} Frame: {}", entity, frame_idx);

                for (slice_name, slice) in &aseprite.slices {
                    // Use static slice data
                    draw_slice(
                        &mut gizmos,
                        transform,
                        sprite.flip_x,
                        slice_name,
                        &slice.rect,
                        canvas_size,
                    );
                }
            }
        }
    }
}

fn draw_slice(
    gizmos: &mut Gizmos,
    transform: &GlobalTransform,
    flip_x: bool,
    slice_name: &str,
    rect: &Rect,
    canvas_size: Vec2,
) {
    // info!("Slice found: {}", slice_name);
    let color = match slice_name {
        "Hitbox" => Color::srgb(1.0, 0.0, 0.0),      // Red
        "Hurtbox" => Color::srgb(0.0, 1.0, 0.0),     // Green
        "Anchor" => Color::srgb(1.0, 1.0, 0.0),      // Yellow
        "Spawn_point" => Color::srgb(0.0, 1.0, 1.0), // Cyan
        _ => {
            // Fallback for partial matches or unknown types
            if slice_name.contains("Hitbox") {
                Color::srgb(1.0, 0.0, 0.0)
            } else if slice_name.contains("Hurtbox") {
                Color::srgb(0.0, 1.0, 0.0)
            } else {
                Color::srgb(0.0, 0.0, 1.0) // Blue
            }
        }
    };

    let size = rect.size();
    let center = rect.center();

    let offset = if canvas_size != Vec2::ZERO {
        Vec2::new(
            center.x - canvas_size.x / 2.0,
            canvas_size.y / 2.0 - center.y,
        )
    } else {
        Vec2::new(center.x, -center.y)
    };

    // Simple approximation for flip:
    let mut final_offset = offset;
    if flip_x {
        final_offset.x = -final_offset.x;
    }

    let world_pos = transform.translation().xy() + final_offset;

    gizmos.rect_2d(world_pos, size, color);
}
