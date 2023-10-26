use bevy_ecs::{entity::Entity, prelude::Component};

use render_api::components::Transform;

use vortex_proto::components::ModelTransform;

#[derive(Component, Clone)]
pub struct ModelTransformControl {
    pub model_transform_entity: Entity,
}

impl ModelTransformControl {
    pub const RADIUS: f32 = 1.5;

    pub fn new(model_transform_entity: Entity) -> Self {
        Self {
            model_transform_entity,
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
}
