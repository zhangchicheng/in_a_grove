use crate::common::Velocity;
use crate::gameplay::mechanics::{CharacterState, CharacterStatus, HitEvent};
use bevy::prelude::*;
use bevy_aseprite_ultra::prelude::*;
use std::collections::{HashMap, HashSet};

/// Component to track which entities have been hit by the current attack
/// to prevent hitting the same entity multiple times per swing.
#[derive(Component, Default)]
pub struct HitTracker {
    pub hits: HashSet<Entity>,
    pub last_animation_tag: Option<String>,
    pub projectile_spawned: bool,
}

#[derive(Component)]
pub struct Projectile {
    pub damage: f32,
    pub owner: Entity,
}

/// Resource to store frame data/attack properties.
/// For now, we can hardcode or load this.
#[derive(Resource, Default)]
pub struct AttackConfig {
    // Map animation tag to damage
    pub attacks: HashMap<String, AttackProperties>,
}

pub struct AttackProperties {
    pub damage: f32,
    pub knockback: f32,
}

impl Default for AttackProperties {
    fn default() -> Self {
        Self {
            damage: 10.0,
            knockback: 50.0,
        }
    }
}

pub fn setup_combat(mut config: ResMut<AttackConfig>) {
    // Example: Register attacks
    // The key should match the animation tag in Aseprite
    config.attacks.insert(
        "attack_1".to_string(),
        AttackProperties {
            damage: 10.0,
            knockback: 100.0,
        },
    );
    config.attacks.insert(
        "attack_2".to_string(),
        AttackProperties {
            damage: 20.0,
            knockback: 150.0,
        },
    );
    config.attacks.insert(
        "attack_3".to_string(),
        AttackProperties {
            damage: 30.0,
            knockback: 200.0,
        },
    );
    config.attacks.insert(
        "throw_spear".to_string(),
        AttackProperties {
            damage: 15.0,
            knockback: 50.0,
        },
    );
    config.attacks.insert(
        "parry".to_string(),
        AttackProperties {
            damage: 5.0,
            knockback: 200.0,
        },
    );
    config.attacks.insert(
        "spear".to_string(),
        AttackProperties {
            damage: 15.0,
            knockback: 100.0,
        },
    );
}

pub fn manage_hit_tracker(mut query: Query<(Entity, &AseAnimation, &mut HitTracker)>) {
    for (_entity, anim, mut tracker) in &mut query {
        let current_tag = anim.animation.tag.clone();

        // If animation changed, clear the tracker
        if tracker.last_animation_tag != current_tag {
            // info!("HitTracker: Entity {:?} changed anim from {:?} to {:?}. Clearing hits.", entity, tracker.last_animation_tag, current_tag);
            tracker.hits.clear();
            tracker.projectile_spawned = false;
            tracker.last_animation_tag = current_tag;
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn attack_judgment(
    mut hit_writer: MessageWriter<HitEvent>,
    mut params: ParamSet<(
        Query<(
            Entity,
            &GlobalTransform,
            &AseAnimation,
            &Sprite,
            &HitTracker,
            Option<&Projectile>,
            Option<&CharacterState>,
        )>,
        Query<(
            Entity,
            &GlobalTransform,
            &AseAnimation,
            &Sprite,
            Option<&CharacterState>,
        )>,
        Query<&mut HitTracker>,
    )>,
    aseprites: Res<Assets<Aseprite>>,
    layouts: Res<Assets<TextureAtlasLayout>>,
    config: Res<AttackConfig>,
) {
    struct ActiveAttack {
        attacker_entity: Entity,
        hitboxes: Vec<Rect>,
        damage: f32,
        knockback: f32,
        projectile_owner: Option<Entity>,
        already_hit: HashSet<Entity>,
        tag: String,
    }

    let mut active_attacks = Vec::new();

    // 1. Collect Active Attacks
    for (
        attacker_entity,
        attacker_tf,
        attacker_anim,
        attacker_sprite,
        tracker,
        projectile,
        state,
    ) in params.p0().iter()
    {
        // Skip dead attackers
        if state.is_some_and(|s| matches!(s.status, CharacterStatus::Dead)) {
            continue;
        }

        let Some(attacker_ase) = aseprites.get(&attacker_anim.aseprite) else {
            continue;
        };

        // Optimization: Only check if current animation is an attack
        let Some(tag) = &attacker_anim.animation.tag else {
            continue;
        };

        if !tag.contains("attack")
            && !tag.contains("parry")
            && !tag.contains("throw_spear")
            && !tag.contains("spear")
        {
            continue;
        }

        let mut local_hitboxes = Vec::new();
        let attacker_canvas_size = get_canvas_size(&attacker_ase.atlas_layout, &layouts);

        if let Some(atlas) = &attacker_sprite.texture_atlas {
            let frame_idx = atlas.index;

            for (slice_name, slice) in &attacker_ase.slices {
                if slice_name.contains("Hitbox") {
                    // Assuming static slices for now as SliceMeta doesn't expose keys in this version
                    local_hitboxes.push((slice_name.clone(), slice.rect));
                }
            }
        }

        let mut world_hitboxes = Vec::new();
        for (_slice_name, rect) in local_hitboxes {
            let center = rect.center();
            let size = rect.size();

            let mut offset = if attacker_canvas_size != Vec2::ZERO {
                Vec2::new(
                    center.x - attacker_canvas_size.x / 2.0,
                    attacker_canvas_size.y / 2.0 - center.y,
                )
            } else {
                Vec2::new(center.x, -center.y)
            };

            if attacker_sprite.flip_x {
                offset.x = -offset.x;
            }

            let world_center = attacker_tf.translation().xy() + offset;
            world_hitboxes.push(Rect::from_center_size(world_center, size));
        }

        if !world_hitboxes.is_empty() {
            let mut damage = 10.0;
            let mut knockback = 50.0;

            if let Some(props) = config.attacks.get(tag) {
                damage = props.damage;
                knockback = props.knockback;
            }

            if let Some(proj) = projectile {
                damage = proj.damage;
            }

            active_attacks.push(ActiveAttack {
                attacker_entity,
                hitboxes: world_hitboxes,
                damage,
                knockback,
                projectile_owner: projectile.map(|p| p.owner),
                already_hit: tracker.hits.clone(),
                tag: tag.clone(),
            });
        }
    }

    let mut hits_to_record = Vec::new();

    // 2. Check against Victims
    for (
        victim_entity,
        victim_tf,
        victim_anim,
        victim_sprite,
        state,
    ) in params.p1().iter()
    {
        // Skip dead entities
        if let Some(state) = &state {
            if matches!(state.status, CharacterStatus::Dead) {
                continue;
            }
        } else {
            // Debug: Why is state None?
            // info!("Victim {:?} has no CharacterState", victim_entity);
        }

        let Some(victim_ase) = aseprites.get(&victim_anim.aseprite) else {
            continue;
        };
        let victim_canvas_size = get_canvas_size(&victim_ase.atlas_layout, &layouts);

        for attack in &active_attacks {
            if attack.attacker_entity == victim_entity {
                continue; // Don't hit self
            }

            if attack.projectile_owner == Some(victim_entity) {
                continue; // Don't hit owner
            }

            if attack.already_hit.contains(&victim_entity) {
                continue; // Already hit
            }

            let mut hit_confirmed = false;

            if let Some(atlas) = &victim_sprite.texture_atlas {
                let frame_idx = atlas.index;

                for (slice_name, slice) in &victim_ase.slices {
                    if slice_name.contains("Hurtbox") {
                        let rect = slice.rect;

                        let center = rect.center();
                        let size = rect.size();

                        let mut offset = if victim_canvas_size != Vec2::ZERO {
                            Vec2::new(
                                center.x - victim_canvas_size.x / 2.0,
                                victim_canvas_size.y / 2.0 - center.y,
                            )
                        } else {
                            Vec2::new(center.x, -center.y)
                        };

                        if victim_sprite.flip_x {
                            offset.x = -offset.x;
                        }

                        let world_center = victim_tf.translation().xy() + offset;
                        let victim_rect = Rect::from_center_size(world_center, size);

                        for hitbox in &attack.hitboxes {
                            if !hitbox.intersect(victim_rect).is_empty() {
                                hit_confirmed = true;
                                break;
                            }
                        }
                    }
                    if hit_confirmed {
                        break;
                    }
                }
            }

            if hit_confirmed {
                // Check for Parry
                let is_parrying = if let Some(tag) = victim_anim.animation.tag.as_deref() {
                    tag == "parry" && victim_anim.animation.playing
                } else {
                    false
                };

                if is_parrying {
                    info!(
                        "Parried! Entity {:?} parried attack from {:?}",
                        victim_entity, attack.attacker_entity
                    );
                    hits_to_record.push((attack.attacker_entity, victim_entity));
                    continue;
                }

                // Send HitEvent
                hit_writer.write(HitEvent {
                    attacker: attack.attacker_entity,
                    victim: victim_entity,
                    move_id: attack.tag.clone(),
                    damage: attack.damage,
                    knockback: attack.knockback,
                    hit_pos: attack.hitboxes[0].center(),
                });

                hits_to_record.push((attack.attacker_entity, victim_entity));
            }
        }
    }

    // 3. Update HitTrackers
    let mut trackers = params.p2();
    for (attacker, victim) in hits_to_record {
        if let Ok(mut tracker) = trackers.get_mut(attacker) {
            tracker.hits.insert(victim);
        }
    }
}

#[allow(clippy::collapsible_if)]
pub fn spawn_projectiles(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &GlobalTransform,
        &AseAnimation,
        &Sprite,
        &mut HitTracker,
    )>,
    aseprites: Res<Assets<Aseprite>>,
    layouts: Res<Assets<TextureAtlasLayout>>,
) {
    for (entity, transform, anim, sprite, mut tracker) in &mut query {
        if anim.animation.tag.as_deref() == Some("throw_spear") {
            if tracker.projectile_spawned {
                continue;
            }

            if let Some(atlas) = &sprite.texture_atlas {
                let current_frame = atlas.index;

                if let Some(aseprite) = aseprites.get(&anim.aseprite) {
                    if let Some(spawn_slice) = aseprite.slices.get("Spawn_point") {
                        if current_frame == 37 {
                            // Found Spawn_point key for current frame

                            // Find Anchor for "spear" (frame 37)
                            let spear_frame_index = 37;

                            if let Some(anchor_slice) = aseprite.slices.get("Anchor") {
                                {
                                    let canvas_size =
                                        get_canvas_size(&aseprite.atlas_layout, &layouts);

                                    let spawn_offset = calculate_offset(
                                        spawn_slice.rect,
                                        canvas_size,
                                        sprite.flip_x,
                                    );
                                    let anchor_offset = calculate_offset(
                                        anchor_slice.rect,
                                        canvas_size,
                                        sprite.flip_x,
                                    );

                                    let spawn_pos =
                                        transform.translation().xy() + spawn_offset - anchor_offset;

                                    // Character faces left by default, so default velocity is Left (-400.0)
                                    let mut spear_velocity = Vec2::new(-400.0, 0.0);
                                    if sprite.flip_x {
                                        spear_velocity.x = -spear_velocity.x;
                                    }

                                    commands.spawn((
                                        Sprite {
                                            image: aseprite.atlas_image.clone(),
                                            flip_x: sprite.flip_x,
                                            texture_atlas: Some(TextureAtlas {
                                                layout: aseprite.atlas_layout.clone(),
                                                index: spear_frame_index,
                                            }),
                                            ..default()
                                        },
                                        Transform::from_translation(
                                            spawn_pos.extend(transform.translation().z),
                                        ),
                                        GlobalTransform::default(),
                                        Visibility::default(),
                                        InheritedVisibility::default(),
                                        ViewVisibility::default(),
                                        Velocity {
                                            lin: spear_velocity,
                                            on_ground: false,
                                            jumps_left: 0,
                                        },
                                        Projectile {
                                            damage: 15.0,
                                            owner: entity,
                                        },
                                        HitTracker::default(),
                                        // Removed AseAnimation to prevent it from playing other animations (like idle/walk)
                                        // Since "spear" is a single frame (37), we just set the TextureAtlas index manually above.
                                    ));

                                    tracker.projectile_spawned = true;
                                    info!("Spawned spear at {:?}", spawn_pos);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn projectile_movement(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &Velocity), With<Projectile>>,
) {
    let dt = time.delta_secs();
    for (mut transform, vel) in &mut query {
        transform.translation.x += vel.lin.x * dt;
        transform.translation.y += vel.lin.y * dt;
    }
}

fn calculate_offset(rect: Rect, canvas_size: Vec2, flip_x: bool) -> Vec2 {
    let center = rect.center();
    let mut offset = if canvas_size != Vec2::ZERO {
        Vec2::new(
            center.x - canvas_size.x / 2.0,
            canvas_size.y / 2.0 - center.y,
        )
    } else {
        Vec2::new(center.x, -center.y)
    };

    if flip_x {
        offset.x = -offset.x;
    }
    offset
}

fn get_canvas_size(
    layout_handle: &Handle<TextureAtlasLayout>,
    layouts: &Assets<TextureAtlasLayout>,
) -> Vec2 {
    if let Some(layout) = layouts.get(layout_handle) {
        return layout
            .textures
            .iter()
            .map(|r| r.size())
            .reduce(|a, b| a.max(b))
            .map(|s| s.as_vec2())
            .unwrap_or(Vec2::ZERO);
    }
    Vec2::ZERO
}
