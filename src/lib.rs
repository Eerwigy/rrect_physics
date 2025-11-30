//! An axis-aligned round rectangle implementation for the bevy game engine

mod components;
#[cfg(feature = "physics")]
mod spatial_grid;

pub use components::{Collider, ColliderType, Force, Movement, PartialForce, Position};
pub use spatial_grid::SpatialHashGrid;

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
#[cfg(feature = "physics")]
use bevy_math::prelude::*;
#[cfg(feature = "physics")]
use bevy_platform::collections::{HashMap, HashSet};
#[cfg(feature = "physics")]
use bevy_time::prelude::*;
#[cfg(feature = "render")]
use bevy_transform::components::Transform;

/// Physics plugin for singleplayer games
#[cfg(feature = "singleplayer")]
pub struct PvwRRectPhysicsPlugin {
    pub spatial_grid_size: f32,
}

#[cfg(feature = "singleplayer")]
impl Default for PvwRRectPhysicsPlugin {
    fn default() -> Self {
        Self {
            spatial_grid_size: SpatialHashGrid::DEFAULT_CELL_SIZE,
        }
    }
}

#[cfg(feature = "singleplayer")]
impl Plugin for PvwRRectPhysicsPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "reflect")]
        app.add_plugins(type_registry);
        app.init_resource::<TileSize>();
        app.insert_resource(SpatialHashGrid {
            cell_size: self.spatial_grid_size,
            ..Default::default()
        });
        app.add_message::<CollisionMessage>();
        app.configure_sets(FixedUpdate, PhysicsSystems);
        app.add_systems(
            FixedUpdate,
            (
                update_velocity_and_predict,
                update_spatial_hash_grid,
                check_collisions_and_resolve,
            )
                .chain()
                .in_set(PhysicsSystems),
        );
        app.add_systems(Update, update_translation);
        app.add_systems(PostUpdate, translation_just_added);
    }
}

/// Physics plugin for multiplayer games on client side
#[cfg(feature = "client")]
pub struct PvwRRectPhysicsPluginClient;

#[cfg(feature = "client")]
impl Plugin for PvwRRectPhysicsPluginClient {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "reflect")]
        app.add_plugins(type_registry);
        app.init_resource::<TileSize>();
        app.add_systems(Update, update_translation);
        app.add_systems(PostUpdate, translation_just_added);
    }
}

/// Physics plugin for multiplayer games on client side
#[cfg(feature = "server")]
pub struct PvwRRectPhysicsPluginServer {
    pub spatial_grid_size: f32,
}

#[cfg(feature = "server")]
impl Default for PvwRRectPhysicsPluginServer {
    fn default() -> Self {
        Self {
            spatial_grid_size: SpatialHashGrid::DEFAULT_CELL_SIZE,
        }
    }
}

#[cfg(feature = "server")]
impl Plugin for PvwRRectPhysicsPluginServer {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "reflect")]
        app.add_plugins(type_registry);
        app.insert_resource(SpatialHashGrid {
            cell_size: self.spatial_grid_size,
            ..Default::default()
        });
        app.add_message::<CollisionMessage>();
        app.configure_sets(FixedUpdate, PhysicsSystems);
        app.add_systems(
            FixedUpdate,
            (
                update_velocity_and_predict,
                update_spatial_hash_grid,
                check_collisions_and_resolve,
            )
                .chain()
                .in_set(PhysicsSystems),
        );
    }
}

#[cfg(feature = "reflect")]
fn type_registry(app: &mut App) {
    app.register_type::<Position>();
    app.register_type::<Movement>();
    app.register_type::<Collider>();
    app.register_type::<ColliderType>();
    app.register_type::<Force>();
}

#[cfg(feature = "render")]
#[derive(Debug, Resource, Clone, Copy)]
pub struct TileSize(f32, Vec2);

#[cfg(feature = "render")]
impl Default for TileSize {
    fn default() -> Self {
        Self::new(8.0)
    }
}

#[cfg(feature = "render")]
impl TileSize {
    pub fn new(size: f32) -> Self {
        Self(size, Vec2::splat(size))
    }

    pub fn size(&self) -> f32 {
        self.0
    }

    pub fn vec(&self) -> Vec2 {
        self.1
    }
}

#[cfg(feature = "physics")]
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct PhysicsSystems;

#[cfg(feature = "physics")]
#[derive(Message)]
pub struct CollisionMessage(pub Entity, pub Entity);

#[cfg(feature = "physics")]
fn update_velocity_and_predict(
    mut query: Query<(&mut Movement, &mut Position)>,
    time: Res<Time<Fixed>>,
) {
    let dt = time.delta_secs();

    for (mut vel, mut pos) in &mut query {
        vel.velocity = Vec2::ZERO;
        vel.apply_damping(dt);

        for force in vel.forces.clone().values() {
            vel.velocity += force.force * dt;
        }

        vel.velocity = vel.velocity.clamp_length_max(Movement::MAX_VELOCITY * dt);

        pos.0 += vel.velocity;
    }
}

#[cfg(feature = "physics")]
fn update_spatial_hash_grid(
    mut spatial_grid: ResMut<SpatialHashGrid>,
    query: Query<(Entity, &Position, &Collider)>,
) {
    let mut ent_list = HashSet::new();
    for (ent, pos, coll) in &query {
        ent_list.insert(ent);
        spatial_grid.insert_or_update(ent, pos, coll);
    }

    let mut to_remove = Vec::new();
    for ent in spatial_grid.ent_to_grid.keys() {
        if !ent_list.contains(ent) {
            to_remove.push(*ent);
        }
    }

    for ent in to_remove {
        spatial_grid.remove(ent);
    }
}

#[cfg(feature = "physics")]
fn check_collisions_and_resolve(
    mut messages: MessageWriter<CollisionMessage>,
    mut query: Query<(&mut Position, &Collider, Entity)>,
    spatial_grid: Res<SpatialHashGrid>,
) {
    let len = query.iter().len();
    let mut detection_data = HashMap::with_capacity(len);
    let mut dynamic_positions = HashMap::with_capacity(len);

    for (pos, coll, ent) in query.iter() {
        detection_data.insert(ent, (*pos, *coll));
        if matches!(coll.ctype, ColliderType::Dynamic(_)) {
            dynamic_positions.insert(ent, pos.0);
        }
    }

    let mut checked = HashSet::with_capacity(len * 2);

    for (&entity_a, &(mut pos_a, collider_a)) in &detection_data {
        // Optimisation hack for tilemaps
        if matches!(collider_a.ctype, ColliderType::Static) {
            continue;
        }

        let Some(neighbors) = spatial_grid.iter(entity_a) else {
            continue;
        };

        for &entity_b in neighbors.iter() {
            let Some(&(mut pos_b, collider_b)) = detection_data.get(&entity_b) else {
                continue;
            };

            if entity_a == entity_b {
                continue;
            }

            let pair = if entity_a < entity_b {
                (entity_a, entity_b)
            } else {
                (entity_b, entity_a)
            };

            if !checked.insert(pair) {
                continue;
            }

            if let Some(pos) = dynamic_positions.get(&entity_a) {
                pos_a.0 += pos;
            }

            if let Some(pos) = dynamic_positions.get(&entity_b) {
                pos_b.0 += pos;
            }

            let offset = pos_b.0 - pos_a.0;
            let offset_abs = offset.abs();

            let avg_size = (collider_a.size + collider_b.size) * 0.5;

            // check AABB collision
            if offset_abs.x >= avg_size.x || offset_abs.y >= avg_size.y {
                continue;
            }

            let mtv: Vec2;
            let radii = collider_a.radius + collider_b.radius;
            let dist = offset_abs - avg_size + radii;

            // check inner AABB collision
            if dist.x < 0.0 || dist.y < 0.0 {
                let overlap = avg_size - offset_abs;

                if overlap.x < overlap.y {
                    mtv = Vec2::new(overlap.x * offset.x.signum(), 0.0);
                } else {
                    mtv = Vec2::new(0.0, overlap.y * offset.y.signum());
                }
            } else {
                // check corners
                let dist_sq = dist.length_squared();
                if dist_sq > radii * radii {
                    continue;
                }

                let dist_length = dist_sq.sqrt();
                mtv = (dist / dist_length) * (radii - dist_length) * offset.signum();
            }

            messages.write(CollisionMessage(entity_a, entity_b));

            match (collider_a.ctype, collider_b.ctype) {
                // resolve collision by pushing one of the collider away
                (ColliderType::Dynamic(_), ColliderType::Static) => {
                    *dynamic_positions.entry(entity_a).or_insert(pos_a.0) -= mtv;
                },

                // in this case we push both away based on their masses
                (ColliderType::Dynamic(mass_a), ColliderType::Dynamic(mass_b)) => {
                    let total_mass = mass_a + mass_b;
                    let mass_share_a = mass_a / total_mass;
                    let mass_share_b = mass_b / total_mass;

                    *dynamic_positions.entry(entity_a).or_insert(pos_a.0) -= mtv * mass_share_b;
                    *dynamic_positions.entry(entity_b).or_insert(pos_b.0) += mtv * mass_share_a;
                },
                _ => {},
            }
        }
    }

    for (mut next_pos, _, entity) in &mut query {
        if let Some(new_pos_vec) = dynamic_positions.get(&entity) {
            next_pos.0 = *new_pos_vec;
        }
    }
}

#[cfg(feature = "render")]
fn translation_just_added(
    mut query: Query<(&mut Transform, &Position), Or<(Added<Transform>, Added<Position>)>>,
    tile_size: Res<TileSize>,
) {
    let size = tile_size.size();
    for (mut transf, pos) in &mut query {
        transf.translation = vec3(pos.0.x * size, pos.0.y * size, transf.translation.z);
    }
}

#[cfg(feature = "render")]
fn update_translation(mut query: Query<(&mut Transform, &Position)>, tile_size: Res<TileSize>) {
    let size = tile_size.size();
    for (mut transform, pos) in &mut query {
        let z_index = transform.translation.z;
        let temp = Vec3::new(pos.0.x * size, pos.0.y * size, z_index);
        transform.translation = transform.translation.lerp(temp, 0.2);
    }
}
