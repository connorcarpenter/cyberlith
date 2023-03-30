use std::default::Default;

use bevy_ecs::bundle::Bundle;

use crate::assets::Handle;
use crate::base::{PbrMaterial, TriMesh};

use super::transform::Transform;

#[derive(Default, Bundle)]
pub struct RenderObjectBundle {
    pub mesh: Handle<TriMesh>,
    pub material: Handle<PbrMaterial>,
    pub transform: Transform,
}
