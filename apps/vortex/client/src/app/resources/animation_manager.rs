use std::collections::HashMap;

use bevy_ecs::{entity::Entity, prelude::{Commands, Query}, system::Resource};
use bevy_log::warn;

use naia_bevy_client::Client;

use math::Vec2;
use render_api::components::{Transform, Visibility};

use vortex_proto::components::{AnimFrame, Vertex3d};

#[derive(Resource)]
pub struct AnimationManager {
    pub current_skel_file: Option<Entity>,
    current_frame: Option<Entity>,
    // (file_entity, order) -> frame_entity
    frames: HashMap<(Entity, u8), Entity>,
    // (frame_entity, vertex_name) -> rotation_entity
    vertex_names: HashMap<(Entity, String), Entity>
}

impl Default for AnimationManager {
    fn default() -> Self {
        Self {
            current_skel_file: None,
            current_frame: None,
            frames: HashMap::new(),
            vertex_names: HashMap::new(),
        }
    }
}

impl AnimationManager {

    pub(crate) fn register_frame(&mut self, file_entity: Entity, frame_entity: Entity, frame: &AnimFrame) {
        let order = frame.get_order();
        self.frames.insert((file_entity, order), frame_entity);
    }

    pub(crate) fn deregister_frame(&mut self, file_entity: &Entity, frame: &AnimFrame) {
        let order = frame.get_order();
        self.frames.remove(&(*file_entity, order));
    }

    pub(crate) fn register_rotation(&mut self, frame_entity: Entity, rotation_entity: Entity, vertex_name: String) {
        self.vertex_names.insert((frame_entity, vertex_name), rotation_entity);
    }

    pub(crate) fn deregister_rotation(&mut self, frame_entity: &Entity, vertex_name: &str) {
        self.vertex_names.remove(&(*frame_entity, vertex_name.to_string()));
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

    pub(crate) fn drag_vertex(
        &mut self,
        commands: &mut Commands,
        client: &Client,
        vert_3d_entity: Entity,
        mouse_position: Vec2,
        delta: Vec2,
    ) {
        // let auth_status =
        //     commands.entity(vertex_3d_entity).authority(client).unwrap();
        // if !(auth_status.is_requested() || auth_status.is_granted()) {
        //     // only continue to mutate if requested or granted authority over vertex
        //     info!("No authority over vertex, skipping..");
        //     return;
        // }
        //
        // // get camera
        // let camera_3d = camera_manager.camera_3d_entity().unwrap();
        // let camera_transform: Transform = *transform_q.get(camera_3d).unwrap();
        // let (camera, camera_projection) = camera_q.get(camera_3d).unwrap();
        //
        // let camera_viewport = camera.viewport.unwrap();
        // let view_matrix = camera_transform.view_matrix();
        // let projection_matrix =
        //     camera_projection.projection_matrix(&camera_viewport);
        //
        // // get 2d vertex transform
        // let vertex_2d_transform = transform_q.get(vertex_2d_entity).unwrap();
        //
        // // convert 2d to 3d
        // let new_3d_position = convert_2d_to_3d(
        //     &view_matrix,
        //     &projection_matrix,
        //     &camera_viewport.size_vec2(),
        //     &mouse_position,
        //     vertex_2d_transform.translation.z,
        // );
        //
        // // set networked 3d vertex position
        // let mut vertex_3d = vertex_3d_q.get_mut(vertex_3d_entity).unwrap();
        //
        // if let Some((_, old_3d_position, _)) =
        //     vertex_manager.last_vertex_dragged
        // {
        //     vertex_manager.last_vertex_dragged =
        //         Some((vertex_2d_entity, old_3d_position, new_3d_position));
        // } else {
        //     let old_3d_position = vertex_3d.as_vec3();
        //     vertex_manager.last_vertex_dragged =
        //         Some((vertex_2d_entity, old_3d_position, new_3d_position));
        // }
        //
        // vertex_3d.set_x(new_3d_position.x as i16);
        // vertex_3d.set_y(new_3d_position.y as i16);
        // vertex_3d.set_z(new_3d_position.z as i16);
    }

    pub(crate) fn sync_vertices_3d(
        &self,
        vertex_3d_q: &Query<(Entity, &Vertex3d)>,
        transform_q: &mut Query<&mut Transform>,
        visibility_q: &Query<&Visibility>
    ) {
        for (vertex_3d_entity, vertex_3d) in vertex_3d_q.iter() {
            // check visibility
            if let Ok(visibility) = visibility_q.get(vertex_3d_entity) {
                if !visibility.visible {
                    continue;
                }
            };

            // get transform
            let Ok(mut vertex_3d_transform) = transform_q.get_mut(vertex_3d_entity) else {
                warn!("Vertex3d entity {:?} has no Transform", vertex_3d_entity);
                continue;
            };

            // update 3d vertices
            vertex_3d_transform.translation.x = vertex_3d.x().into();
            vertex_3d_transform.translation.y = vertex_3d.y().into();
            vertex_3d_transform.translation.z = vertex_3d.z().into();
        }
    }
}
