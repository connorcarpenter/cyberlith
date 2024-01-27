use bevy_ecs::component::Component;

use naia_bevy_shared::{
    EntityProperty, Property, Protocol, ProtocolPlugin, Replicate, Serde, UnsignedVariableInteger,
};

use math::{Quat, SerdeQuat};

pub struct AnimationComponentsPlugin;

impl ProtocolPlugin for AnimationComponentsPlugin {
    fn build(&self, protocol: &mut Protocol) {
        protocol
            .add_component::<AnimFrame>()
            .add_component::<AnimRotation>();
    }
}

// Transition
#[derive(Clone, PartialEq, Serde)]
pub struct Transition {
    duration_5ms: UnsignedVariableInteger<7>,
    //pub easing: Easing,
}

impl Transition {
    pub fn new(duration_ms: u16) -> Self {
        let duration_5ms = duration_ms / 5;
        Self {
            duration_5ms: duration_5ms.into(),
        }
    }

    pub fn get_duration_ms(&self) -> u16 {
        let duration_5ms: u16 = self.duration_5ms.to();
        duration_5ms * 5
    }

    pub fn set_duration_ms(&mut self, duration_ms: u16) {
        let duration_5ms = duration_ms / 5;
        self.duration_5ms = duration_5ms.into();
    }
}

// Frame
#[derive(Component, Replicate)]
pub struct AnimFrame {
    pub file_entity: EntityProperty,
    order: Property<UnsignedVariableInteger<4>>,
    pub transition: Property<Transition>,
}

impl AnimFrame {
    pub fn new(order: u8, transition: Transition) -> Self {
        Self::new_complete(order.into(), transition)
    }

    pub fn get_order(&self) -> u8 {
        self.order.to()
    }

    pub fn set_order(&mut self, order: u8) {
        *self.order = order.into();
    }
}

// AnimRotation
#[derive(Component, Replicate)]
pub struct AnimRotation {
    pub frame_entity: EntityProperty,
    pub vertex_name: Property<String>,
    rotation: Property<SerdeQuat>,
}

impl AnimRotation {
    pub fn new(vertex_name: String, rotation: SerdeQuat) -> Self {
        Self::new_complete(vertex_name, rotation)
    }

    pub fn get_rotation(&self) -> Quat {
        (*self.rotation).into()
    }

    pub fn set_rotation(&mut self, rotation: Quat) {
        *self.rotation = SerdeQuat::from(rotation);
    }

    pub fn get_rotation_serde(&self) -> SerdeQuat {
        *self.rotation
    }
}
