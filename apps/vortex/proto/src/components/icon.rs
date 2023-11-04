use bevy_ecs::component::Component;

use naia_bevy_shared::{
    EntityProperty, Property, Protocol, ProtocolPlugin, Replicate,
};

use crate::components::VertexSerdeInt;

pub struct IconComponentsPlugin;

impl ProtocolPlugin for IconComponentsPlugin {
    fn build(&self, protocol: &mut Protocol) {
        protocol
            .add_component::<IconVertex>()
            .add_component::<IconEdge>()
            .add_component::<IconFace>();
    }
}

// IconVertex
#[derive(Component, Replicate)]
pub struct IconVertex {
    x: Property<VertexSerdeInt>,
    y: Property<VertexSerdeInt>,
}

impl IconVertex {
    pub fn new(x: i16, y: i16) -> Self {
        Self::new_complete(x.into(), y.into())
    }

    pub fn x(&self) -> i16 {
        self.x.to()
    }

    pub fn y(&self) -> i16 {
        self.y.to()
    }

    pub fn set_x(&mut self, x: i16) {
        *self.x = x.into();
    }

    pub fn set_y(&mut self, y: i16) {
        *self.y = y.into();
    }
}

// IconEdge
#[derive(Component, Replicate)]
pub struct IconEdge {
    pub start: EntityProperty,
    pub end: EntityProperty,
}

impl IconEdge {
    pub fn new() -> Self {
        Self::new_complete()
    }
}

// IconFace
#[derive(Component, Replicate)]
pub struct IconFace {
    pub vertex_a: EntityProperty,
    pub vertex_b: EntityProperty,
    pub vertex_c: EntityProperty,
    pub edge_a: EntityProperty,
    pub edge_b: EntityProperty,
    pub edge_c: EntityProperty,
}

impl IconFace {
    pub fn new() -> Self {
        Self::new_complete()
    }
}