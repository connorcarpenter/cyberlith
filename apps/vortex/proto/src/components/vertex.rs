use bevy_ecs::component::Component;

use naia_bevy_shared::{
    EntityProperty, Property, Protocol, ProtocolPlugin, Replicate, Serde, SignedVariableInteger,
};

use math::Vec3;

use crate::types::TabId;

pub struct VertexComponentsPlugin;

impl ProtocolPlugin for VertexComponentsPlugin {
    fn build(&self, protocol: &mut Protocol) {
        protocol
            .add_component::<Vertex3d>()
            .add_component::<VertexChild>()
            .add_component::<VertexRoot>()
            .add_component::<OwnedByTab>()
            .add_component::<VertexType>()
            .add_component::<Edge3d>()
            .add_component::<Face3d>();
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

    pub fn set_vec3(&mut self, vec3: &Vec3) {
        self.set_x(vec3.x as i16);
        self.set_y(vec3.y as i16);
        self.set_z(vec3.z as i16);
    }

    pub fn from_vec3(vec3: Vec3) -> Self {
        Self::new(vec3.x as i16, vec3.y as i16, vec3.z as i16)
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

// VertexRoot
#[derive(Component, Replicate)]
pub struct VertexRoot;

// TabOwnership
#[derive(Component, Replicate)]
pub struct OwnedByTab {
    pub tab_id: Property<TabId>,
}

impl OwnedByTab {
    pub fn new(tab_id: TabId) -> Self {
        Self::new_complete(tab_id)
    }
}

#[derive(Serde, PartialEq, Clone, Copy)]
pub enum VertexTypeValue {
    Mesh,
    Skel,
}

// VertexType
#[derive(Component, Replicate)]
pub struct VertexType {
    pub value: Property<VertexTypeValue>,
}

impl VertexType {
    pub fn new(value: VertexTypeValue) -> Self {
        Self::new_complete(value)
    }
}

// Edge3d
#[derive(Component, Replicate)]
pub struct Edge3d {
    pub start: EntityProperty,
    pub end: EntityProperty,
}

impl Edge3d {
    pub fn new() -> Self {
        Self::new_complete()
    }
}

// Face3d
#[derive(Component, Replicate)]
pub struct Face3d {
    pub vertex_a: EntityProperty,
    pub vertex_b: EntityProperty,
    pub vertex_c: EntityProperty,
}

impl Face3d {
    pub fn new() -> Self {
        Self::new_complete()
    }
}
