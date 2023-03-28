use std::default::Default;

use bevy_ecs::bundle::Bundle;

use crate::assets::{Handle, Mesh, StandardMaterial};

use super::transform::Transform;

#[derive(Default, Bundle)]
pub struct RenderObjectBundle {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    pub transform: Transform,
}
