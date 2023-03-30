use std::default::Default;

use bevy_ecs::bundle::Bundle;

use crate::assets::{Handle, Material, Mesh};

use super::transform::Transform;

#[derive(Default, Bundle)]
pub struct RenderObjectBundle {
    pub mesh: Handle<Mesh>,
    pub material: Handle<Material>,
    pub transform: Transform,
}
