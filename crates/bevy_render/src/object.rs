use std::default::Default;

use bevy_ecs::{bundle::Bundle};

use crate::{Handle, Mesh, StandardMaterial, transform::Transform};

#[derive(Default, Bundle)]
pub struct RenderObjectBundle {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    pub transform: Transform,
}