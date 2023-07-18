use bevy_ecs::component::Component;

use naia_bevy_shared::{
    EntityProperty, Property, Protocol, ProtocolPlugin, Replicate, SignedVariableInteger,
};

use math::Vec3;

pub struct VertexComponentsPlugin;

impl ProtocolPlugin for VertexComponentsPlugin {
    fn build(&self, protocol: &mut Protocol) {
        protocol
            .add_component::<Vertex3d>()
            .add_component::<VertexChild>()
            .add_component::<VertexRootChild>();
    }
}

// VertexChild
#[derive(Component, Replicate)]
pub struct VertexChild {
    pub parent_id: EntityProperty,
}

impl VertexChild {
    pub fn new() -> Self {
        Self::new_complete()
    }
}

// VertexRootChild
#[derive(Component, Replicate)]
pub struct VertexRootChild;

impl VertexRootChild {
    pub fn new() -> Self {
        Self::new_complete()
    }
}

// Vertex3d
#[derive(Component, Replicate)]
pub struct Vertex3d {
    x: Property<VertexSerdeInt>,
    y: Property<VertexSerdeInt>,
    z: Property<VertexSerdeInt>,
}

pub type VertexSerdeInt = SignedVariableInteger<4>;

impl Vertex3d {
    pub fn new(x: i16, y: i16, z: i16) -> Self {
        Self::new_complete(x.into(), y.into(), z.into())
    }

    pub fn x(&self) -> i16 {
        self.x.to()
    }

    pub fn y(&self) -> i16 {
        self.y.to()
    }

    pub fn z(&self) -> i16 {
        self.z.to()
    }

    pub fn set_x(&mut self, x: i16) {
        *self.x = x.into();
    }

    pub fn set_y(&mut self, y: i16) {
        *self.y = y.into();
    }

    pub fn set_z(&mut self, z: i16) {
        *self.z = z.into();
    }

    pub fn as_vec3(&self) -> Vec3 {
        Vec3::new(self.x() as f32, self.y() as f32, self.z() as f32)
    }
}
