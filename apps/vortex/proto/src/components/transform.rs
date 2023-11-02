use bevy_ecs::component::Component;

use naia_bevy_shared::{
    EntityProperty, Property, Protocol, ProtocolPlugin, Replicate,
    Serde, SignedVariableInteger, UnsignedVariableInteger,
};

use math::{Quat, SerdeQuat, Vec3};

pub struct TransformComponentsPlugin;

impl ProtocolPlugin for TransformComponentsPlugin {
    fn build(&self, protocol: &mut Protocol) {
        protocol
            .add_component::<NetTransform>()
            .add_component::<SkinOrSceneEntity>();
    }
}

// FileOwnership
#[derive(Component, Replicate)]
pub struct SkinOrSceneEntity {
    pub value: EntityProperty,
    pub value_type: Property<NetTransformEntityType>,
}

impl SkinOrSceneEntity {
    pub fn new(value_type: NetTransformEntityType) -> Self {
        Self::new_complete(value_type)
    }
}

// NetTransformEntityType
#[derive(Serde, Copy, Clone, PartialEq, Debug)]
pub enum NetTransformEntityType {
    Uninit,
    Skin,
    Scene,
}

// NetTransform
#[derive(Component, Replicate)]
pub struct NetTransform {
    rotation: Property<SerdeQuat>,
    translation_x: Property<SignedVariableInteger<4>>,
    translation_y: Property<SignedVariableInteger<4>>,
    translation_z: Property<SignedVariableInteger<4>>,
    scale_x: Property<UnsignedVariableInteger<4>>,
    scale_y: Property<UnsignedVariableInteger<4>>,
    scale_z: Property<UnsignedVariableInteger<4>>,
}

impl NetTransform {
    pub fn new(
        rotation: SerdeQuat,
        translation_x: f32,
        translation_y: f32,
        translation_z: f32,
        scale_x: f32,
        scale_y: f32,
        scale_z: f32,
    ) -> Self {
        let translation_x = translation_x as i16;
        let translation_y = translation_y as i16;
        let translation_z = translation_z as i16;

        let scale_x = (scale_x * 100.0) as u16;
        let scale_y = (scale_y * 100.0) as u16;
        let scale_z = (scale_z * 100.0) as u16;

        Self::new_complete(
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

    pub fn translation_vec3(&self) -> Vec3 {
        Vec3::new(
            self.translation_x() as f32,
            self.translation_y() as f32,
            self.translation_z() as f32,
        )
    }

    pub fn set_translation_vec3(&mut self, vec3: &Vec3) {
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

    pub fn scale_vec3(&self) -> Vec3 {
        Vec3::new(self.scale_x(), self.scale_y(), self.scale_z())
    }

    pub fn set_scale_vec3(&mut self, vec3: &Vec3) {
        self.set_scale_x(vec3.x);
        self.set_scale_y(vec3.y);
        self.set_scale_z(vec3.z);
    }
}
