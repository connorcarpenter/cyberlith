use bevy_ecs::{entity::Entity, prelude::Component};

use render_api::components::Transform;

use vortex_proto::components::ModelTransform;

#[derive(Clone, Copy)]
pub enum ScaleAxis {
    X,
    Y,
    Z,
}

#[derive(Clone, Copy)]
pub enum ModelTransformControlType {
    Translation,
    RotationVertex,
    RotationEdge,
    Scale(ScaleAxis),
    NA,
}

#[derive(Component, Clone)]
pub struct ModelTransformControl {
    pub model_transform_entity: Entity,
    pub control_type: ModelTransformControlType,
}

impl ModelTransformControl {

    pub const EDGE_LENGTH: f32 = 20.0;

    pub const SCALE_EDGE_LENGTH: f32 = 14.0;

    pub fn new(model_transform_entity: Entity, control_type: ModelTransformControlType) -> Self {
        Self {
            model_transform_entity,
            control_type,
        }
    }
}

pub struct ModelTransformLocal;

impl ModelTransformLocal {
    pub fn to_transform(model_transform: &ModelTransform) -> Transform {
        let mut transform = Transform::from_translation(model_transform.translation_vec3());
        transform.rotation = model_transform.rotation();
        transform.scale = model_transform.scale_vec3();
        return transform;
    }

    pub fn set_transform(model_transform: &mut ModelTransform, transform: &Transform) {
        model_transform.set_translation_vec3(&transform.translation);
        model_transform.set_rotation(transform.rotation);
        model_transform.set_scale_vec3(&transform.scale);
    }
}
