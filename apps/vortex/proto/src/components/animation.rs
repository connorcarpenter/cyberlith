use bevy_ecs::component::Component;

use naia_bevy_shared::{EntityProperty, Property, Protocol, ProtocolPlugin, Replicate, UnsignedVariableInteger};

use math::{Quat, SerdeQuat};

pub struct AnimationComponentsPlugin;

impl ProtocolPlugin for AnimationComponentsPlugin {
    fn build(&self, protocol: &mut Protocol) {
        protocol.add_component::<AnimRotation>();
    }
}

// Frame
#[derive(Component, Replicate)]
pub struct AnimFrame {
    order: Property<UnsignedVariableInteger<4>>,
    duration_5ms: Property<UnsignedVariableInteger<4>>,
}

impl AnimFrame {
    pub fn new(order: u8, duration_ms: u16) -> Self {
        let duration_5ms = duration_ms / 5;
        Self::new_complete(order.into(), duration_5ms.into())
    }

    pub fn get_order(&self) -> u8 {
        self.order.to()
    }

    pub fn set_order(&mut self, order: u8) {
        *self.order = order.into();
    }

    pub fn get_duration_ms(&self) -> u16 {
        let duration_5ms: u16 = self.duration_5ms.to();
        duration_5ms * 5
    }

    pub fn set_duration_ms(&mut self, duration_ms: u16) {
        let duration_5ms = duration_ms / 5;
        *self.duration_5ms = duration_5ms.into();
    }
}

// AnimRotation
#[derive(Component, Replicate)]
pub struct AnimRotation {
    pub frame_entity: EntityProperty,
    pub vertex_3d_entity: EntityProperty,
    rotation: Property<SerdeQuat>,
}

impl AnimRotation {
    pub fn new(rotation: Quat) -> Self {
        let serde_quat = SerdeQuat::from(rotation);
        Self::new_complete(serde_quat)
    }

    pub fn get_rotation(&self) -> Quat {
        (*self.rotation).into()
    }

    pub fn set_rotation(&mut self, rotation: Quat) {
        *self.rotation = SerdeQuat::from(rotation);
    }
}