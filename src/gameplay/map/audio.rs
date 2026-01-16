use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

#[derive(Component)]
pub struct MapAudioSpawned;

pub fn spawn_map_audio(
    mut commands: Commands,
    query: Query<(Entity, &TiledMap), Without<MapAudioSpawned>>,
    map_assets: Res<Assets<TiledMapAsset>>,
    asset_server: Res<AssetServer>,
) {
    for (entity, tiled_map) in query.iter() {
        if let Some(map_asset) = map_assets.get(&tiled_map.0) {
            // Mark as spawned to avoid processing again
            commands.entity(entity).insert(MapAudioSpawned);

            let map = &map_asset.map;

            // Calculate map dimensions for coordinate conversion
            // Assuming orthogonal map and TilemapAnchor::Center
            let map_width = map.width as f32 * map.tile_width as f32;
            let map_height = map.height as f32 * map.tile_height as f32;

            for layer in map.layers() {
                if let Some(obj_layer) = layer.as_object_layer() {
                    for obj in obj_layer.objects() {
                        if obj.user_type == "SoundEmitter" {
                            // Extract properties
                            let mut audio_path = None;
                            let mut volume = 1.0;
                            let mut is_loop = false;
                            // let mut radius = 300.0; // Not used yet in PlaybackSettings directly

                            if let Some(tiled::PropertyValue::StringValue(s)) =
                                obj.properties.get("audio_path")
                            {
                                audio_path = Some(s.clone());
                            }
                            if let Some(tiled::PropertyValue::FloatValue(v)) =
                                obj.properties.get("volume")
                            {
                                volume = *v;
                            }
                            if let Some(tiled::PropertyValue::BoolValue(b)) =
                                obj.properties.get("loop")
                            {
                                is_loop = *b;
                            }

                            if let Some(path) = audio_path {
                                // Convert coordinates
                                // Tiled (0,0) is Top-Left. Y increases downwards.
                                // Bevy (0,0) is Center. Y increases upwards.
                                // Local X = -Width/2 + obj.x
                                // Local Y = Height/2 - obj.y
                                let x = -map_width / 2.0 + obj.x;
                                let y = map_height / 2.0 - obj.y;

                                let settings = PlaybackSettings {
                                    mode: if is_loop {
                                        bevy::audio::PlaybackMode::Loop
                                    } else {
                                        bevy::audio::PlaybackMode::Despawn
                                    },
                                    volume: bevy::audio::Volume::Linear(volume),
                                    spatial: true,
                                    spatial_scale: Some(bevy::audio::SpatialScale::new(1.0 / 32.0)), // 32 pixels = 1 meter
                                    ..default()
                                };

                                let audio_entity = commands
                                    .spawn((
                                        AudioPlayer(asset_server.load::<AudioSource>(path)),
                                        settings,
                                        Transform::from_xyz(x, y, 0.0),
                                        GlobalTransform::default(),
                                    ))
                                    .id();
                                commands.entity(entity).add_child(audio_entity);

                                info!("Spawned audio at ({}, {})", x, y);
                            }
                        }
                    }
                }
            }
        }
    }
}
