use bevy_ecs::component::Component;
use naia_bevy_shared::{Property, Protocol, ProtocolPlugin, Replicate, SignedVariableInteger};

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
    x: Property<SignedVariableInteger<4>>,
    y: Property<SignedVariableInteger<4>>,
    z: Property<SignedVariableInteger<4>>,
}

impl Vertex3d {
    pub fn new(x: u16, y: u16, z: u16) -> Self {
        Self::new_complete(x.into(), y.into(), z.into())
    }

    pub fn x(&self) -> u16 {
        self.x.to()
    }

    pub fn y(&self) -> u16 {
        self.y.to()
    }

    pub fn z(&self) -> u16 {
        self.z.to()
    }

    pub fn set_x(&mut self, x: u16) {
        *self.x = x.into();
    }

    pub fn set_y(&mut self, y: u16) {
        *self.y = y.into();
    }

    pub fn set_z(&mut self, z: u16) {
        *self.z = z.into();
    }
}

// Vertex2d
#[derive(Component, Replicate)]
pub struct Vertex2d {
    x: Property<SignedVariableInteger<4>>,
    y: Property<SignedVariableInteger<4>>,
}

impl Vertex2d {
    pub fn new(x: u16, y: u16) -> Self {
        Self::new_complete(x.into(), y.into())
    }

    pub fn x(&self) -> u16 {
        self.x.to()
    }

    pub fn y(&self) -> u16 {
        self.y.to()
    }

    pub fn set_x(&mut self, x: u16) {
        *self.x = x.into();
    }

    pub fn set_y(&mut self, y: u16) {
        *self.y = y.into();
    }
}