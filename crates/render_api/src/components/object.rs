use std::default::Default;

use bevy_ecs::bundle::Bundle;

use crate::{
    assets::Handle,
    base::{CpuMesh, PbrMaterial},
};

use super::transform::Transform;

#[derive(Default, Bundle)]
pub struct RenderObjectBundle {
    pub mesh: Handle<CpuMesh>,
    pub material: Handle<PbrMaterial>,
    pub transform: Transform,
}
