use bevy_ecs::component::Component;

use naia_bevy_shared::{
    EntityProperty, Property, Protocol, ProtocolPlugin, Replicate, Serde, SignedVariableInteger,
    UnsignedInteger, UnsignedVariableInteger,
};

use math::{Quat, SerdeQuat, Vec3};

pub struct VertexComponentsPlugin;

impl ProtocolPlugin for VertexComponentsPlugin {
    fn build(&self, protocol: &mut Protocol) {
        protocol
            .add_component::<Vertex3d>()
            .add_component::<VertexRoot>()
            .add_component::<Edge3d>()
            .add_component::<EdgeAngle>()
            .add_component::<Face3d>()
            .add_component::<OwnedByFile>()
            .add_component::<FileType>()
            .add_component::<ShapeName>()
            .add_component::<PaletteColor>()
            .add_component::<BackgroundSkinColor>()
            .add_component::<FaceColor>()
            .add_component::<ModelTransform>();
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

// FileOwnership
#[derive(Component, Replicate)]
pub struct OwnedByFile {
    pub file_entity: EntityProperty,
}

impl OwnedByFile {
    pub fn new() -> Self {
        Self::new_complete()
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
    pub edge_a: EntityProperty,
    pub edge_b: EntityProperty,
    pub edge_c: EntityProperty,
}

impl Face3d {
    pub fn new() -> Self {
        Self::new_complete()
    }
}

// FileType
#[derive(Serde, Copy, Clone, PartialEq, Debug)]
pub enum FileExtension {
    Skel,
    Mesh,
    Anim,
    Palette,
    Skin,
    Model,
    Scene,
    Unknown,
}

impl From<&str> for FileExtension {
    fn from(file_name: &str) -> Self {
        // split file name by '.'
        let split: Vec<_> = file_name.split('.').collect();
        let ext: &str = split.last().unwrap();

        //info!("file_name: {}, ext: {}", file_name, ext);

        // match file extension to enum
        match ext {
            "skel" => FileExtension::Skel,
            "mesh" => FileExtension::Mesh,
            "anim" => FileExtension::Anim,
            "palette" => FileExtension::Palette,
            "skin" => FileExtension::Skin,
            "model" => FileExtension::Model,
            "scene" => FileExtension::Scene,
            _ => FileExtension::Unknown,
        }
    }
}

impl FileExtension {
    pub fn can_io(&self) -> bool {
        match self {
            FileExtension::Skel
            | FileExtension::Mesh
            | FileExtension::Anim
            | FileExtension::Palette
            | FileExtension::Skin
            | FileExtension::Model
            | FileExtension::Scene => true,
            _ => false,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            FileExtension::Skel => ".skel".to_string(),
            FileExtension::Mesh => ".mesh".to_string(),
            FileExtension::Anim => ".anim".to_string(),
            FileExtension::Palette => ".palette".to_string(),
            FileExtension::Skin => ".skin".to_string(),
            FileExtension::Model => ".model".to_string(),
            FileExtension::Scene => ".scene".to_string(),
            FileExtension::Unknown => "<UNKNOWN>".to_string(),
        }
    }
}

#[derive(Component, Replicate)]
pub struct FileType {
    pub value: Property<FileExtension>,
}

impl FileType {
    pub fn new(value: FileExtension) -> Self {
        Self::new_complete(value)
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
}

// PaletteColor
#[derive(Component, Replicate)]
pub struct PaletteColor {
    pub index: Property<u8>,
    pub r: Property<u8>,
    pub g: Property<u8>,
    pub b: Property<u8>,
    pub file_entity: EntityProperty,
}

impl PaletteColor {
    pub fn new(index: u8, r: u8, g: u8, b: u8) -> Self {
        Self::new_complete(index, r, g, b)
    }
}

// FaceColor
#[derive(Component, Replicate)]
pub struct FaceColor {
    pub skin_file_entity: EntityProperty,
    pub face_3d_entity: EntityProperty,
    pub palette_color_entity: EntityProperty,
}

impl FaceColor {
    pub fn new() -> Self {
        Self::new_complete()
    }
}

// BackgroundSkinColor
#[derive(Component, Replicate)]
pub struct BackgroundSkinColor {
    pub skin_file_entity: EntityProperty,
    pub palette_color_entity: EntityProperty,
}

impl BackgroundSkinColor {
    pub fn new() -> Self {
        Self::new_complete()
    }
}

// ModelTransform
#[derive(Component, Replicate)]
pub struct ModelTransform {
    pub vertex_name: Property<String>,
    rotation: Property<SerdeQuat>,
    translation_x: Property<SignedVariableInteger<4>>,
    translation_y: Property<SignedVariableInteger<4>>,
    translation_z: Property<SignedVariableInteger<4>>,
    scale_x: Property<UnsignedVariableInteger<4>>,
    scale_y: Property<UnsignedVariableInteger<4>>,
    scale_z: Property<UnsignedVariableInteger<4>>,
}

impl ModelTransform {
    pub fn new(
        vertex_name: String,
        rotation: SerdeQuat,
        translation_x: i16,
        translation_y: i16,
        translation_z: i16,
        scale_x: f32,
        scale_y: f32,
        scale_z: f32,
    ) -> Self {
        let scale_x = (scale_x * 100.0) as u16;
        let scale_y = (scale_y * 100.0) as u16;
        let scale_z = (scale_z * 100.0) as u16;

        Self::new_complete(
            vertex_name,
            rotation,
            translation_x.into(),
            translation_y.into(),
            translation_z.into(),
            scale_x.into(),
            scale_y.into(),
            scale_z.into(),
        )
    }

    pub fn rotation(&self) -> Quat {
        (*self.rotation).into()
    }

    pub fn set_rotation(&mut self, rotation: Quat) {
        *self.rotation = SerdeQuat::from(rotation);
    }

    pub fn get_rotation_serde(&self) -> SerdeQuat {
        *self.rotation
    }

    pub fn translation_x(&self) -> i16 {
        self.translation_x.to()
    }

    pub fn translation_y(&self) -> i16 {
        self.translation_y.to()
    }

    pub fn translation_z(&self) -> i16 {
        self.translation_z.to()
    }

    pub fn set_translation_x(&mut self, x: i16) {
        *self.translation_x = x.into();
    }

    pub fn set_translation_y(&mut self, y: i16) {
        *self.translation_y = y.into();
    }

    pub fn set_translation_z(&mut self, z: i16) {
        *self.translation_z = z.into();
    }

    pub fn translation_as_vec3(&self) -> Vec3 {
        Vec3::new(
            self.translation_x() as f32,
            self.translation_y() as f32,
            self.translation_z() as f32,
        )
    }

    pub fn translation_set_vec3(&mut self, vec3: &Vec3) {
        self.set_translation_x(vec3.x as i16);
        self.set_translation_y(vec3.y as i16);
        self.set_translation_z(vec3.z as i16);
    }

    pub fn scale_x(&self) -> f32 {
        let scale_x: u16 = self.scale_x.to();
        scale_x as f32 / 100.0
    }

    pub fn scale_y(&self) -> f32 {
        let scale_y: u16 = self.scale_y.to();
        scale_y as f32 / 100.0
    }

    pub fn scale_z(&self) -> f32 {
        let scale_z: u16 = self.scale_z.to();
        scale_z as f32 / 100.0
    }

    pub fn set_scale_x(&mut self, val: f32) {
        *self.scale_x = ((val * 100.0) as u16).into();
    }

    pub fn set_scale_y(&mut self, val: f32) {
        *self.scale_y = ((val * 100.0) as u16).into();
    }

    pub fn set_scale_z(&mut self, val: f32) {
        *self.scale_z = ((val * 100.0) as u16).into();
    }
}
