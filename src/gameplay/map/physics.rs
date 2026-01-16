use bevy::prelude::*;
use bevy_ecs_tiled::physics::backend::TiledPhysicsBackend;
use bevy_ecs_tiled::physics::collider::{ColliderCreated, TiledColliderSource};
use bevy_ecs_tiled::prelude::geo::MultiPolygon;
use bevy_ecs_tiled::prelude::*;

use crate::common::{ColliderSize, DespawnOnExit, Platform};
use crate::states::GameState;

#[derive(Default, Clone, Debug, Reflect)]
pub struct GamePhysicsBackend;

impl TiledPhysicsBackend for GamePhysicsBackend {
    fn spawn_colliders(
        &self,
        commands: &mut Commands,
        source: &TiledEvent<ColliderCreated>,
        multi_polygon: &MultiPolygon<f32>,
    ) -> Vec<Entity> {
        let mut entities = Vec::new();

        for polygon in multi_polygon.0.iter() {
            // Calculate bounding box from the polygon points
            let (min, max) =
                polygon
                    .exterior()
                    .points()
                    .fold((Vec2::MAX, Vec2::MIN), |(min, max), p| {
                        let v = Vec2::new(p.x(), p.y());
                        (min.min(v), max.max(v))
                    });

            let size = max - min;
            let center = (min + max) / 2.0;

            if size.x <= 0.0 || size.y <= 0.0 {
                continue;
            }

            let name = match source.event.source {
                TiledColliderSource::Object => "Object Collider",
                TiledColliderSource::TilesLayer => "Tile Collider",
            };

            // Spawn the collider
            // The polygon coordinates are already relative to the parent (Object or Layer)
            // The plugin will automatically parent this entity to the object/layer
            let entity = commands
                .spawn((
                    Transform::from_xyz(center.x, center.y, 0.0),
                    ColliderSize(size),
                    Platform,
                    DespawnOnExit(GameState::Gameplay),
                    Name::new(name),
                ))
                .id();

            entities.push(entity);
        }

        entities
    }
}
