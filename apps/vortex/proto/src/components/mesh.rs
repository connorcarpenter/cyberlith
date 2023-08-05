use bevy_ecs::component::Component;

use naia_bevy_shared::{
    EntityProperty, Protocol, ProtocolPlugin, Replicate,
};

pub struct MeshComponentsPlugin;

impl ProtocolPlugin for MeshComponentsPlugin {
    fn build(&self, protocol: &mut Protocol) {
        protocol
            .add_component::<IsMesh>()
            .add_component::<MeshEdge>()
            .add_component::<MeshFace>();
    }
}

// IsMesh
#[derive(Component, Replicate)]
pub struct IsMesh;

impl IsMesh {
    pub fn new() -> Self {
        Self::new_complete()
    }
}

// MeshEdge
#[derive(Component, Replicate)]
pub struct MeshEdge {
    pub vertex_a: EntityProperty,
    pub vertex_b: EntityProperty,
}

impl MeshEdge {
    pub fn new() -> Self {
        Self::new_complete()
    }
}

// MeshFace
#[derive(Component, Replicate)]
pub struct MeshFace {
    pub vertex_a: EntityProperty,
    pub vertex_b: EntityProperty,
    pub vertex_c: EntityProperty,
}

impl MeshFace {
    pub fn new() -> Self {
        Self::new_complete()
    }
}