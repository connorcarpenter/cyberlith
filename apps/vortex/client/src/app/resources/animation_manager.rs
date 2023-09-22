use std::collections::HashMap;

use bevy_ecs::{
    entity::Entity,
    prelude::{Commands, Query},
    query::With,
    system::{Res, ResMut, Resource, SystemState},
    world::World,
};
use bevy_log::{info, warn};

use naia_bevy_client::{Client, CommandsExt, ReplicationConfig};

use math::{convert_2d_to_3d, Quat, Vec2, Vec3};
use render_api::components::{Camera, CameraProjection, Projection, Transform, Visibility};

use vortex_proto::components::{AnimFrame, AnimRotation, ShapeName, Vertex3d, VertexRoot};

use crate::app::{
    components::LocalAnimRotation,
    resources::{camera_manager::CameraManager, canvas::Canvas, vertex_manager::VertexManager},
};

#[derive(Resource)]
pub struct AnimationManager {
    pub current_skel_file: Option<Entity>,
    current_frame: Option<Entity>,
    // (file_entity, order) -> frame_entity
    frames: HashMap<(Entity, u8), Entity>,
    // rotation_entity -> (frame_entity, vertex_name)
    rotations: HashMap<Entity, (Entity, String)>,
    // (frame_entity, vertex_name) -> rotation_entity
    vertex_names: HashMap<(Entity, String), Entity>,

    last_rotation_dragged: Option<(Entity, Option<Quat>, Quat)>,
}

impl Default for AnimationManager {
    fn default() -> Self {
        Self {
            current_skel_file: None,
            current_frame: None,
            frames: HashMap::new(),
            rotations: HashMap::new(),
            vertex_names: HashMap::new(),
            last_rotation_dragged: None,
        }
    }
}

impl AnimationManager {
    pub(crate) fn current_frame(&self) -> Option<Entity> {
        self.current_frame
    }

    pub(crate) fn register_frame(
        &mut self,
        file_entity: Entity,
        frame_entity: Entity,
        frame: &AnimFrame,
    ) {
        let order = frame.get_order();
        self.frames.insert((file_entity, order), frame_entity);
        if self.current_frame.is_none() {
            self.current_frame = Some(frame_entity);
        }
    }

    pub(crate) fn deregister_frame(&mut self, file_entity: &Entity, frame_entity: &Entity, frame: &AnimFrame) {
        let order = frame.get_order();
        self.frames.remove(&(*file_entity, order));

        if let Some(current_frame_entity) = self.current_frame {
            if current_frame_entity == *frame_entity {
                self.current_frame = None;
            }
        }
    }

    pub(crate) fn register_rotation(
        &mut self,
        frame_entity: Entity,
        rotation_entity: Entity,
        vertex_name: String,
    ) {
        self.rotations.insert(rotation_entity, (frame_entity, vertex_name.clone()));
        self.vertex_names
            .insert((frame_entity, vertex_name), rotation_entity);
    }

    pub(crate) fn deregister_rotation(&mut self, rotation_entity: &Entity) {
        let (frame_entity, vertex_name) = self.rotations.remove(rotation_entity).unwrap();
        self.vertex_names.remove(&(frame_entity, vertex_name));
    }

    pub fn get_current_rotation(&self, vertex_name: &str) -> Option<&Entity> {
        let current_frame = self.current_frame?;
        self.vertex_names.get(&(current_frame, vertex_name.to_string()))
    }

    pub(crate) fn drag_vertex(
        &mut self,
        world: &mut World,
        vertex_3d_entity: Entity,
        vertex_2d_entity: Entity,
        mouse_position: Vec2,
    ) {
        // get rotation
        let Some(frame_entity) = self.current_frame else {
            info!("no frame");
            return;
        };

        let Ok(shape_name) = world.query::<&ShapeName>().get(world, vertex_3d_entity) else {
            return;
        };

        let shape_name: String = (*shape_name.value).clone();

        let mut system_state: SystemState<(
            Commands,
            Client,
            ResMut<Canvas>,
            Res<CameraManager>,
            Res<VertexManager>,
            Query<(&Camera, &Projection)>,
            Query<&Transform>,
            Query<&Vertex3d>,
            Query<(&mut AnimRotation, &LocalAnimRotation)>,
            Query<&ShapeName>,
        )> = SystemState::new(world);
        let (
            mut commands,
            mut client,
            mut canvas,
            camera_manager,
            vertex_manager,
            camera_q,
            transform_q,
            vertex_3d_q,
            mut rotation_q,
            name_q,
        ) = system_state.get_mut(world);

        //
        let rotation_entity_opt = self.get_current_rotation(&shape_name).copied();
        if let Some(rotation_entity) = rotation_entity_opt {
            let auth_status = commands.entity(rotation_entity).authority(&client).unwrap();
            if !(auth_status.is_requested() || auth_status.is_granted()) {
                // only continue to mutate if requested or granted authority over vertex
                info!("No authority over vertex rotation, skipping..");
                return;
            }
        }

        //

        // get parent 3d position
        let parent_vertex_3d_entity = vertex_manager
            .vertex_parent_3d_entity(&vertex_3d_entity)
            .unwrap();
        let parent_original_3d_position =
            vertex_3d_q.get(parent_vertex_3d_entity).unwrap().as_vec3();
        let parent_rotated_3d_position = transform_q
            .get(parent_vertex_3d_entity)
            .unwrap()
            .translation;

        // get old 3d position
        let original_3d_position = vertex_3d_q.get(vertex_3d_entity).unwrap().as_vec3();

        // get camera
        let camera_3d = camera_manager.camera_3d_entity().unwrap();
        let camera_transform: Transform = *transform_q.get(camera_3d).unwrap();
        let (camera, camera_projection) = camera_q.get(camera_3d).unwrap();

        let camera_viewport = camera.viewport.unwrap();
        let view_matrix = camera_transform.view_matrix();
        let projection_matrix = camera_projection.projection_matrix(&camera_viewport);

        // get 2d vertex transform
        let vertex_2d_transform = transform_q.get(vertex_2d_entity).unwrap();

        // convert 2d to 3d
        let new_3d_position = convert_2d_to_3d(
            &view_matrix,
            &projection_matrix,
            &camera_viewport.size_vec2(),
            &mouse_position,
            vertex_2d_transform.translation.z,
        );

        let base_direction = (original_3d_position - parent_original_3d_position).normalize();
        let target_direction = (new_3d_position - parent_rotated_3d_position).normalize();
        let axis_of_rotation = base_direction.cross(target_direction).normalize();
        let angle = base_direction.angle_between(target_direction);
        let mut rotation_angle = Quat::from_axis_angle(axis_of_rotation, angle);

        get_inversed_final_rotation(
            &vertex_manager,
            self,
            frame_entity,
            parent_vertex_3d_entity,
            &name_q,
            &rotation_q,
            &mut rotation_angle,
        );

        if let Some(rotation_entity) = rotation_entity_opt {
            let (mut anim_rotation, _) = rotation_q.get_mut(rotation_entity).unwrap();

            self.update_last_rotation_dragged(
                vertex_2d_entity,
                Some(anim_rotation.get_rotation()),
                rotation_angle,
            );

            anim_rotation.set_rotation(rotation_angle);
        } else {
            self.update_last_rotation_dragged(vertex_2d_entity, None, rotation_angle);

            // create new rotation entity
            self.create_networked_rotation(
                &mut commands,
                &mut client,
                frame_entity,
                shape_name.to_string(),
                rotation_angle,
            );
        };

        canvas.queue_resync_shapes();

        system_state.apply(world);
    }

    pub fn reset_last_rotation_dragged(&mut self) {
        self.last_rotation_dragged = None;
    }

    fn update_last_rotation_dragged(
        &mut self,
        vertex_2d_entity: Entity,
        old_rotation: Option<Quat>,
        new_rotation: Quat,
    ) {
        if let Some((_, old_rotation, _)) = self.last_rotation_dragged {
            self.last_rotation_dragged = Some((vertex_2d_entity, old_rotation, new_rotation));
        } else {
            self.last_rotation_dragged = Some((vertex_2d_entity, old_rotation, new_rotation));
        }
    }

    pub fn take_last_rotation_dragged(&mut self) -> Option<(Entity, Option<Quat>, Quat)> {
        self.last_rotation_dragged.take()
    }

    pub fn create_networked_rotation(
        &mut self,
        commands: &mut Commands,
        client: &mut Client,
        frame_entity: Entity,
        name: String,
        rotation: Quat,
    ) -> Entity {
        let mut component = AnimRotation::new(name.clone(), rotation.into());
        component.frame_entity.set(client, &frame_entity);
        let new_rotation_entity = commands
            .spawn_empty()
            .enable_replication(client)
            .configure_replication(ReplicationConfig::Delegated)
            .insert(component)
            .id();

        self.rotation_postprocess(commands, frame_entity, new_rotation_entity, name);

        return new_rotation_entity;
    }

    pub fn rotation_postprocess(
        &mut self,
        commands: &mut Commands,
        frame_entity: Entity,
        rotation_entity: Entity,
        vertex_name: String,
    ) {
        self.register_rotation(frame_entity, rotation_entity, vertex_name);

        commands
            .entity(rotation_entity)
            .insert(LocalAnimRotation::new());
    }

    pub(crate) fn drag_edge(
        &mut self,
        _commands: &mut Commands,
        _client: &Client,
        _edge_3d_entity: Entity,
        _mouse_position: Vec2,
        _delta: Vec2,
    ) {
        // unused for now
    }

    pub(crate) fn sync_vertices_3d(
        &self,
        vertex_manager: &VertexManager,
        vertex_3d_q: &Query<(Entity, &Vertex3d)>,
        transform_q: &mut Query<&mut Transform>,
        visibility_q: &Query<&Visibility>,
        name_q: &Query<&ShapeName>,
        rotation_q: &mut Query<(&AnimRotation, &mut LocalAnimRotation)>,
        root_q: &Query<Entity, With<VertexRoot>>,
    ) {
        let current_frame = self.current_frame.unwrap();

        // find root 3d vertex
        let mut root_3d_vertex = None;
        for vertex_3d_entity in root_q.iter() {
            if let Ok(visibility) = visibility_q.get(vertex_3d_entity) {
                if !visibility.visible {
                    continue;
                }
            };
            if vertex_3d_q.get(vertex_3d_entity).is_err() {
                continue;
            }
            if root_3d_vertex.is_some() {
                panic!("Multiple root 3d vertices found!");
            }
            root_3d_vertex = Some(vertex_3d_entity);
        }

        let Some(root_3d_vertex) = root_3d_vertex else {
            info!("skipping");
            return;
        };

        let (_, vertex_3d) = vertex_3d_q.get(root_3d_vertex).unwrap();
        let vertex_pos = vertex_3d.as_vec3();

        self.sync_vertices_3d_children(
            vertex_manager,
            vertex_3d_q,
            transform_q,
            name_q,
            rotation_q,
            current_frame,
            root_3d_vertex,
            vertex_pos,
            vertex_pos,
            Quat::IDENTITY,
        );
    }

    fn sync_vertices_3d_children(
        &self,
        vertex_manager: &VertexManager,
        vertex_3d_q: &Query<(Entity, &Vertex3d)>,
        transform_q: &mut Query<&mut Transform>,
        name_q: &Query<&ShapeName>,
        rotation_q: &mut Query<(&AnimRotation, &mut LocalAnimRotation)>,
        frame_entity: Entity,
        parent_vertex_3d_entity: Entity,
        original_parent_pos: Vec3,
        rotated_parent_pos: Vec3,
        parent_rotation: Quat,
    ) {
        let Some(children) = vertex_manager.vertex_children_3d_entities(&parent_vertex_3d_entity) else {
            return;
        };

        for child_vertex_3d_entity in children.iter() {
            let (_, vertex_3d) = vertex_3d_q.get(*child_vertex_3d_entity).unwrap();
            let original_child_pos = vertex_3d.as_vec3();

            let mut rotation = Quat::IDENTITY;
            if let Ok(name_component) = name_q.get(*child_vertex_3d_entity) {
                let name = (*name_component.value).clone();
                if let Some(rotation_entity) = self.vertex_names.get(&(frame_entity, name)) {
                    if let Ok((anim_rotation, mut local_anim_rotation)) =
                        rotation_q.get_mut(*rotation_entity)
                    {
                        rotation = anim_rotation.get_rotation();
                        local_anim_rotation.last_synced_quat = rotation;
                    }
                }
            }

            let rotation = (parent_rotation * rotation).normalize();
            let displacement = original_child_pos - original_parent_pos;
            let rotated_displacement = rotation * displacement;
            let rotated_child_pos = rotated_parent_pos + rotated_displacement;

            // update transform
            let Ok(mut vertex_3d_transform) = transform_q.get_mut(*child_vertex_3d_entity) else {
                warn!("Vertex3d entity {:?} has no Transform", child_vertex_3d_entity);
                continue;
            };
            vertex_3d_transform.translation = rotated_child_pos;

            // recurse
            self.sync_vertices_3d_children(
                vertex_manager,
                vertex_3d_q,
                transform_q,
                name_q,
                rotation_q,
                frame_entity,
                *child_vertex_3d_entity,
                original_child_pos,
                rotated_child_pos,
                rotation,
            );
        }
    }
}

fn get_inversed_final_rotation(
    vertex_manager: &VertexManager,
    anim_manager: &AnimationManager,
    frame_entity: Entity,
    vertex_entity: Entity,
    name_q: &Query<&ShapeName>,
    rotation_q: &Query<(&mut AnimRotation, &LocalAnimRotation)>,
    target_rotation: &mut Quat,
) {
    if let Ok(name_component) = name_q.get(vertex_entity) {
        let name = (*name_component.value).clone();
        if let Some(rotation_entity) = anim_manager.vertex_names.get(&(frame_entity, name)) {
            let (_, anim_rotation) = rotation_q.get(*rotation_entity).unwrap();
            *target_rotation =
                (anim_rotation.last_synced_quat.inverse() * *target_rotation).normalize();
        }
    }

    if let Some(parent_entity) = vertex_manager.vertex_parent_3d_entity(&vertex_entity) {
        get_inversed_final_rotation(
            vertex_manager,
            anim_manager,
            frame_entity,
            parent_entity,
            name_q,
            rotation_q,
            target_rotation,
        );
    }
}
