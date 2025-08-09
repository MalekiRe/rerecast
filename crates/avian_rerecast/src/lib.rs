//! Backend for using [`avian3d`](https://docs.rs/avian3d) with [`bevy_rerecast`](https://docs.rs/bevy_rerecast).

use avian3d::prelude::*;
use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_rerecast_core::{NavmeshApp as _, NavmeshSettings, rerecast::TriMesh};

mod collider_to_trimesh;
pub use crate::collider_to_trimesh::ColliderToTriMesh;

/// Everything you need to get started with the Navmesh plugin.
pub mod prelude {
    pub use crate::AvianBackendPlugin;
}

/// The plugin of the crate. Will make all entities with [`Collider`] a collider belonging to a static [`RigidBody`] available for navmesh generation.
#[non_exhaustive]
#[derive(Debug, Default)]
pub struct AvianBackendPlugin;

impl Plugin for AvianBackendPlugin {
    fn build(&self, app: &mut App) {
        app.set_navmesh_affector_backend(collider_backend);
    }
}

fn collider_backend(
    input: In<NavmeshSettings>,
    colliders: Query<(Entity, &Collider, &Position, &Rotation, &ColliderOf)>,
    bodies: Query<&RigidBody>,
) -> TriMesh {
    colliders
        .iter()
        .filter_map(|(entity, collider, pos, rot, collider_of)| {
            if input
                .filter
                .as_ref()
                .is_some_and(|entities| !entities.contains(&entity))
            {
                return None;
            }
            let body = bodies.get(collider_of.body).ok()?;
            if !body.is_static() {
                return None;
            }
            let subdivisions = 10;
            collider.to_trimesh(*pos, *rot, subdivisions)
        })
        .fold(TriMesh::default(), |mut acc, t| {
            acc.extend(t);
            acc
        })
}
