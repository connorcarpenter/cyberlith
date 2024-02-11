use bevy_ecs::component::Component;

use naia_bevy_shared::{
    EntityProperty, Property, Protocol, ProtocolPlugin, Replicate, Serde, SignedVariableInteger,
    UnsignedInteger,
};

use math::Vec3;

pub struct ShapeComponentsPlugin;

impl ProtocolPlugin for ShapeComponentsPlugin {
    fn build(&self, protocol: &mut Protocol) {
        protocol
            .add_component::<Vertex3d>()
            .add_component::<VertexRoot>()
            .add_component::<Edge3d>()
            .add_component::<EdgeAngle>()
            .add_component::<Face3d>()
            .add_component::<ShapeName>();
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

// VertexRoot
#[derive(Component, Replicate)]
pub struct VertexRoot;

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
    pub edge_a: EntityProperty,
    pub edge_b: EntityProperty,
    pub edge_c: EntityProperty,
}

impl Face3d {
    pub fn new() -> Self {
        Self::new_complete()
    }
}

// ShapeName
#[derive(Component, Replicate)]
pub struct ShapeName {
    pub value: Property<String>,
}

impl ShapeName {
    pub fn new(value: String) -> Self {
        Self::new_complete(value)
    }
}

// EdgeAngle
#[derive(Component, Replicate)]
pub struct EdgeAngle {
    value: Property<SerdeRotation>,
}

impl EdgeAngle {
    pub fn new(value_f32: f32) -> Self {
        let rotation = SerdeRotation::from_degrees(value_f32);

        Self::new_complete(rotation)
    }

    // angle in degrees
    pub fn get_radians(&self) -> f32 {
        self.value.get_radians()
    }

    // angle in degrees
    pub fn set_radians(&mut self, value: f32) {
        self.value.set_radians(value);
    }

    // angle in degrees
    pub fn get_degrees(&self) -> f32 {
        self.value.get_degrees()
    }

    pub fn get_serde(&self) -> SerdeRotation {
        *self.value
    }

    // angle in degrees
    pub fn set_degrees(&mut self, value_f32: f32) {
        self.value.set_degrees(value_f32);
    }
}

#[derive(Serde, Copy, Clone, PartialEq, Debug)]
pub struct SerdeRotation {
    value: UnsignedInteger<6>,
}

impl SerdeRotation {
    const MAX_ANGLES: f32 = 64.0;
    const MAX_DEGREES: f32 = 360.0;

    pub fn from_degrees(degrees: f32) -> Self {
        let value_u8 = (degrees * Self::MAX_ANGLES / Self::MAX_DEGREES) as u8;
        let integer = UnsignedInteger::<6>::new(value_u8);

        Self { value: integer }
    }

    pub fn from_radians(radians: f32) -> Self {
        let degrees = f32::to_degrees(radians);
        Self::from_degrees(degrees)
    }

    // angle in degrees
    pub fn get_radians(&self) -> f32 {
        let degrees = self.get_degrees();
        f32::to_radians(degrees)
    }

    // angle in degrees
    pub fn set_radians(&mut self, value: f32) {
        let degrees = f32::to_degrees(value);
        self.set_degrees(degrees);
    }

    // angle in degrees
    pub fn get_degrees(&self) -> f32 {
        let value_u8: u8 = self.value.to();
        let value_f32 = value_u8 as f32;
        value_f32 * Self::MAX_DEGREES / Self::MAX_ANGLES
    }

    // angle in degrees
    pub fn set_degrees(&mut self, value_f32: f32) {
        let value_u8 = (value_f32 * Self::MAX_ANGLES / Self::MAX_DEGREES) as u8;
        let integer = UnsignedInteger::<6>::new(value_u8);
        self.value = integer;
    }

    pub fn to_inner(&self) -> u8 {
        self.value.to()
    }

    pub fn from_inner(value: u8) -> Self {
        let integer = UnsignedInteger::<6>::new(value);
        Self { value: integer }
    }
}
