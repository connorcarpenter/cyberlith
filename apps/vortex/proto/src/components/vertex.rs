use bevy_ecs::component::Component;
use naia_bevy_shared::{Property, Protocol, ProtocolPlugin, Replicate};

pub struct VertexComponentsPlugin;

impl ProtocolPlugin for VertexComponentsPlugin {
    fn build(&self, protocol: &mut Protocol) {
        protocol
            .add_component::<Vertex3d>()
            .add_component::<Vertex2d>();
    }
}

// Vertex3d
#[derive(Component, Replicate)]
pub struct Vertex3d {
    pub x: Property<u16>,
    pub y: Property<u16>,
    pub z: Property<u16>,
}

impl Vertex3d {
    pub fn new(x: u16, y: u16, z: u16) -> Self {
        Self::new_complete(x, y, z)
    }
}

// Vertex2d
#[derive(Component, Replicate)]
pub struct Vertex2d {
    pub x: Property<u16>,
    pub y: Property<u16>,
}

impl Vertex2d {
    pub fn new(x: u16, y: u16) -> Self {
        Self::new_complete(x, y)
    }
}