use bevy_ecs::{system::{Resource, ResMut, SystemState}, world::World};
use bevy_ecs::entity::Entity;
use bevy_ecs::system::{Commands, Query, Res};
use naia_bevy_client::{Client, CommandsExt, ReplicationConfig};
use math::{Quat, quat_from_spin_direction, SerdeQuat, Vec3};
use render_api::components::Transform;
use vortex_proto::components::{EdgeAngle, ModelTransform, ShapeName, Vertex3d};

use crate::app::resources::{canvas::Canvas, input::InputManager, shape_data::CanvasShape};
use crate::app::resources::edge_manager::EdgeManager;

#[derive(Resource)]
pub struct ModelManager {

}

impl Default for ModelManager {
    fn default() -> Self {
        Self {

        }
    }
}

impl ModelManager {
    pub fn create_networked_model_transform(&mut self, world: &mut World, edge_2d_entity: Entity) -> Entity {

        let mut system_state: SystemState<(
            Commands,
            Client,
            ResMut<Canvas>,
            ResMut<InputManager>,
            Res<EdgeManager>,
            Query<&Transform>,
            Query<&Vertex3d>,
            Query<&EdgeAngle>,
            Query<&ShapeName>,
        )> = SystemState::new(world);
        let (
            mut commands,
            mut client,
            mut canvas,
            mut input_manager,
            edge_manager,
            transform_q,
            vertex_3d_q,
            edge_angle_q,
            shape_name_q,
        ) = system_state.get_mut(world);

        input_manager.deselect_shape(&mut canvas);

        let edge_3d_entity = edge_manager.edge_entity_2d_to_3d(&edge_2d_entity).unwrap();

        // get vertex from edge, in order to get name
        let (parent_vertex_3d_entity, vertex_3d_entity) = edge_manager.edge_get_endpoints(&edge_3d_entity);
        let Ok(shape_name) = shape_name_q.get(vertex_3d_entity);
        let vertex_name = (*shape_name.value).clone();

        // get translation for model transform (midpoint of edge)
        let parent_original_3d_position =
            vertex_3d_q.get(parent_vertex_3d_entity).unwrap().as_vec3();
        let original_3d_position = vertex_3d_q.get(vertex_3d_entity).unwrap().as_vec3();
        let translation = (parent_original_3d_position + original_3d_position) * 0.5;

        // get rotation
        let target_direction = (original_3d_position - parent_original_3d_position).normalize();
        let edge_old_angle = edge_angle_q.get(edge_3d_entity).unwrap();
        let edge_old_angle: f32 = edge_old_angle.get_radians();
        let rotation = quat_from_spin_direction(edge_old_angle, Vec3::Z, target_direction);

        // get scale
        // TODO: this is naive .. find scale by comparing edge length to skin/scene size
        let scale = Vec3::splat(1.0);

        let component = ModelTransform::new(
            vertex_name,
            SerdeQuat::from(rotation),
            translation.x,
            translation.y,
            translation.z,
            scale.x,
            scale.y,
            scale.z,
        );
        let new_model_transform_entity = commands
            .spawn_empty()
            .enable_replication(&mut client)
            .configure_replication(ReplicationConfig::Delegated)
            .insert(component)
            .id();

        // postprocess
        let (translation_entity, rotation_entity, scale_entity) = self.model_transform_postprocess(
            &mut commands,
            new_model_transform_entity,
            translation,
            rotation,
            scale,
        );

        system_state.apply(world);

        new_model_transform_entity
    }

    pub fn model_transform_postprocess(
        &mut self,
        commands: &mut Commands,
        new_model_transform_entity: Entity,
        translation: Vec3,
        rotation: Quat,
        scale: Vec3,
    ) -> (Entity, Entity, Entity) {
        let translation_entity = commands.spawn_empty()
            .insert(ModelTranslation::new(translation.x, translation.y, translation.z))
            .id();
        let rotation_entity = commands.spawn_empty().insert(ModelRotation::new(rotation)).id();
        let scale_entity = commands.spawn_empty().insert(ModelScale::new(scale.x, scale.y, scale.z)).id();
        return (translation_entity, rotation_entity, scale_entity);
    }
}