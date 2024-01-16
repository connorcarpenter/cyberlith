use bevy_ecs::{entity::Entity, prelude::Component};

use render_api::components::Transform;

use vortex_proto::components::NetTransform;

#[derive(Clone, Copy)]
pub enum ScaleAxis {
    X,
    Y,
    Z,
}

#[derive(Clone, Copy)]
pub enum NetTransformControlType {
    Translation,
    RotationVertex,
    RotationEdge,
    Scale(ScaleAxis),
    NA,
}

#[derive(Component, Clone)]
pub struct NetTransformControl {
    pub net_transform_entity: Entity,
    pub control_type: NetTransformControlType,
}

impl NetTransformControl {
    pub const EDGE_LENGTH: f32 = 20.0;

    pub const SCALE_EDGE_LENGTH: f32 = 14.0;

    pub fn new(net_transform_entity: Entity, control_type: NetTransformControlType) -> Self {
        Self {
            net_transform_entity,
            control_type,
        }
    }
}

pub struct NetTransformLocal;

impl NetTransformLocal {
    pub fn to_transform(net_transform: &NetTransform) -> Transform {
        let mut transform = Transform::from_translation(net_transform.translation_vec3());
        transform.rotation = net_transform.rotation();
        transform.scale = net_transform.scale_vec3();
        return transform;
    }

    pub fn set_transform(net_transform: &mut NetTransform, transform: &Transform) {
        net_transform.set_translation_vec3(&transform.translation);
        net_transform.set_rotation(transform.rotation);
        net_transform.set_scale_vec3(&transform.scale);
    }
}
