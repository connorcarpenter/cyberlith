use std::default::Default;

use bevy_ecs::bundle::Bundle;

use crate::{
    assets::Handle,
    base::{CpuMaterial, CpuMesh},
};

use super::transform::Transform;

#[derive(Default, Bundle)]
pub struct RenderObjectBundle {
    pub mesh: Handle<CpuMesh>,
    pub material: Handle<CpuMaterial>,
    pub transform: Transform,
}
