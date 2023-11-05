use std::collections::HashSet;

use bevy_ecs::{
    entity::Entity,
    query::With,
    system::{Commands, Query, Res, ResMut, Resource, SystemState},
    world::World,
    event::EventWriter,
};

use naia_bevy_client::{Client, CommandsExt, ReplicationConfig};

use math::{Vec2, Vec3};

use render_api::{base::{Color, CpuMaterial, CpuMesh}, components::{RenderObjectBundle, RenderLayer, Transform}, resources::RenderFrame, Handle, Assets};

use vortex_proto::components::{IconEdge, OwnedByFile, IconVertex, IconFace};

use crate::app::{
    components::{
        IconEdgeLocal,
        OwnedByFileLocal,
        Edge2dLocal, Vertex2d,
    },
    resources::{
        icon_data::{IconFaceKey, IconVertexData},
        input::InputManager,
        shape_data::CanvasShape, tab_manager::TabManager, camera_manager::CameraManager, canvas::Canvas, icon_data::IconEdgeData
    },
    events::ShapeColorResyncEvent,
    shapes::create_2d_edge_line
};

#[derive(Resource)]
pub struct IconManager {

}

impl Default for IconManager {
    fn default() -> Self {
        Self {

        }
    }
}

impl IconManager {

    pub fn draw(&self, world: &mut World, current_file_entity: &Entity) {
        let Some(current_tab_state) = world.get_resource::<TabManager>().unwrap().current_tab_state() else {
            return;
        };
        let camera_state = &current_tab_state.camera_state;
        let camera_is_2d = camera_state.is_2d();
        if camera_is_2d {
            self.draw_2d(world, current_file_entity);
        }
    }

    fn draw_2d(&self, world: &mut World, current_file_entity: &Entity) {
        {

            let mut system_state: SystemState<(
                ResMut<RenderFrame>,
                Res<InputManager>,
                Query<(Entity, &OwnedByFileLocal), With<IconVertex>>,
                Query<(&Handle<CpuMesh>, &Handle<CpuMaterial>, &Transform, Option<&RenderLayer>)>,
            )> = SystemState::new(world);
            let (
                mut render_frame,
                input_manager,
                vertex_q,
                object_q,
            ) = system_state.get_mut(world);

            let mut edge_entities = HashSet::new();

            // draw vertices, collect edges
            for (vertex_entity, owned_by_file) in vertex_q.iter() {
                if owned_by_file.file_entity != *current_file_entity {
                    continue;
                }

                // draw vertex 2d
                let Some(data) = self.get_vertex_data(&vertex_entity) else {
                    continue;
                };

                let (mesh_handle, mat_handle, transform, render_layer_opt) = object_q.get(vertex_entity).unwrap();
                render_frame.draw_object(render_layer_opt, mesh_handle, mat_handle, transform);

                for edge_entity in data.edges.iter() {
                    edge_entities.insert(*edge_entity);
                }
            }

            // draw edges
            for edge_entity in edge_entities.iter() {
                let (mesh_handle, mat_handle, transform, render_layer_opt) =
                    object_q.get(*edge_entity).unwrap();
                render_frame.draw_object(render_layer_opt, mesh_handle, mat_handle, transform);
            }

            // draw select line & circle
            match input_manager.selected_shape_2d() {
                Some((_, CanvasShape::Edge)) => {
                    // draw select line
                    if let Some(select_line_entity) = input_manager.select_line_entity {
                        let (mesh_handle, mat_handle, transform, render_layer_opt) =
                            object_q.get(select_line_entity).unwrap();
                        render_frame.draw_object(
                            render_layer_opt,
                            mesh_handle,
                            &mat_handle,
                            transform,
                        );
                    }
                }
                Some((_, CanvasShape::Vertex)) => {
                    // draw select circle
                    if let Some(select_circle_entity) = input_manager.select_circle_entity {
                        let (mesh_handle, mat_handle, transform, render_layer_opt) =
                            object_q.get(select_circle_entity).unwrap();
                        render_frame.draw_object(
                            render_layer_opt,
                            mesh_handle,
                            &mat_handle,
                            transform,
                        );
                    }
                }
                _ => {}
            }
        }
    }

    // Vertices

    pub fn reset_last_vertex_dragged(&mut self) {
        self.drags = Vec::new();
        self.dragging_entity = None;
        self.dragging_start = None;
        self.dragging_end = None;
    }

    pub fn update_last_vertex_dragged(
        &mut self,
        vertex_entity: Entity,
        old_position: Vec2,
        new_position: Vec2,
    ) {
        if let Some(old_vertex_entity) = self.dragging_entity {
            // already dragging an entity
            if old_vertex_entity == vertex_entity {
                // dragging same entity
                self.dragging_end = Some(new_position);
            } else {
                // dragging different entity

                // finish current drag
                self.drags.push((
                    self.dragging_entity.unwrap(),
                    self.dragging_start.unwrap(),
                    self.dragging_end.unwrap(),
                ));
                self.dragging_entity = None;
                self.dragging_start = None;
                self.dragging_end = None;

                // start next drag
                self.dragging_entity = Some(vertex_entity);
                self.dragging_start = Some(old_position);
                self.dragging_end = Some(new_position);
            }
        } else {
            // not dragging an entity
            self.dragging_entity = Some(vertex_entity);
            self.dragging_start = Some(old_position);
            self.dragging_end = Some(new_position);
        }
    }

    pub fn take_drags(&mut self) -> Option<Vec<(Entity, Vec2, Vec2)>> {
        if self.dragging_entity.is_some() {
            // finish current drag
            self.drags.push((
                self.dragging_entity.unwrap(),
                self.dragging_start.unwrap(),
                self.dragging_end.unwrap(),
            ));
            self.dragging_entity = None;
            self.dragging_start = None;
            self.dragging_end = None;
        }

        if self.drags.is_empty() {
            return None;
        } else {
            let drags = std::mem::take(&mut self.drags);
            return Some(drags);
        }
    }

    pub fn create_networked_vertex(
        &mut self,
        commands: &mut Commands,
        client: &mut Client,
        camera_manager: &mut CameraManager,
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
        file_entity: Entity,
        position: Vec2,
        entities_to_release: &mut Vec<Entity>,
    ) -> Entity {
        // create new vertex
        let mut owned_by_file_component = OwnedByFile::new();
        owned_by_file_component
            .file_entity
            .set(client, &file_entity);
        let new_vertex_entity = commands
            .spawn_empty()
            .enable_replication(client)
            .configure_replication(ReplicationConfig::Delegated)
            .insert(IconVertex::from_vec2(position))
            .insert(owned_by_file_component)
            .id();

        entities_to_release.push(new_vertex_entity);

        // add local components to vertex
        self.vertex_postprocess(
            commands,
            meshes,
            materials,
            camera_manager,
            new_vertex_entity,
            Some(file_entity),
            Vertex2d::ENABLED_COLOR,
        );

        return new_vertex_entity;
    }

    pub fn vertex_postprocess(
        &mut self,
        commands: &mut Commands,
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
        camera_manager: &CameraManager,
        vertex_entity: Entity,
        ownership_opt: Option<Entity>,
        color: Color,
    ) {

        commands
            .entity(vertex_entity)
            .spawn(RenderObjectBundle::circle(
                meshes,
                materials,
                Vec2::ZERO,
                Vertex2d::RADIUS,
                Vertex2d::SUBDIVISIONS,
                color,
                None,
            ))
            .insert(camera_manager.layer_2d)
            .id();

        if let Some(file_entity) = ownership_opt {
            commands
                .entity(vertex_entity)
                .insert(OwnedByFileLocal::new(file_entity));
        }

        self.register_vertex(vertex_entity, );
    }

    pub fn register_vertex(
        &mut self,
        entity: Entity,
    ) {
        self.vertices.insert(
            entity,
            IconVertexData::new(),
        );
    }

    pub fn on_vertex_moved(
        &self,
        client: &Client,
        meshes: &mut Assets<CpuMesh>,
        mesh_handle_q: &Query<&Handle<CpuMesh>>,
        face_q: &Query<&IconFace>,
        transform_q: &mut Query<&mut Transform>,
        vertex_entity: &Entity,
    ) {
        let Some(vertex_data) = self.vertices.get(vertex_entity) else {
            panic!("IconVertex entity: `{:?}` has not been registered", vertex_entity);
        };

        for face_key in &vertex_data.faces {
            let Some(Some(net_face_data)) = self.face_keys.get(face_key) else {
                panic!("Face key: `{:?}` has not been registered", face_key);
            };
            if net_face_data.net_entity.is_none() {
                continue;
            }
            let net_face_entity = net_face_data.net_entity.unwrap();

            // need to get vertices from IconFace component because they are in the correct order
            let Ok(face) = face_q.get(net_face_entity) else {
                panic!("IconFace entity: `{:?}` has not been registered", net_face_entity);
            };
            let vertex_a = face.vertex_a.get(client).unwrap();
            let vertex_b = face.vertex_b.get(client).unwrap();
            let vertex_c = face.vertex_c.get(client).unwrap();

            let mut positions = [Vec3::ZERO, Vec3::ZERO, Vec3::ZERO];
            for (index, vertex) in [vertex_a, vertex_b, vertex_c].iter().enumerate() {
                positions[index] = transform_q.get(*vertex).unwrap().translation;
            }

            let (new_mesh, new_center) = RenderObjectBundle::world_triangle_mesh(positions);

            // update mesh
            let mesh_handle = mesh_handle_q.get(net_face_entity).unwrap();
            meshes.set(mesh_handle, new_mesh);

            // update transform
            let mut transform = transform_q.get_mut(net_face_entity).unwrap();
            transform.translation = new_center;
        }
    }

    pub fn cleanup_deleted_vertex(
        &mut self,
        canvas: &mut Canvas,
        input_manager: &mut InputManager,
        vertex_entity: &Entity,
    ) {
        // unregister vertex
        self.unregister_3d_vertex(vertex_entity);

        if input_manager.hovered_entity == Some((*vertex_entity, CanvasShape::Vertex)) {
            input_manager.hovered_entity = None;
        }

        canvas.queue_resync_shapes();
    }

    pub(crate) fn vertex_get_edges(&self, vertex_entity: &Entity) -> Option<&HashSet<Entity>> {
        self.vertices
            .get(vertex_entity)
            .map(|data| &data.edges)
    }

    pub(crate) fn vertex_add_edge(&mut self, vertex_entity: &Entity, edge_entity: Entity) {
        let Some(vertex_data) = self.vertices.get_mut(&vertex_entity) else {
            panic!("Vertex entity: `{:?}` has not been registered", vertex_entity);
        };
        vertex_data.add_edge(edge_entity);
    }

    pub(crate) fn vertex_remove_edge(
        &mut self,
        vertex_entity: &Entity,
        edge_entity: &Entity,
    ) {
        // at this point, vertex_entity may have already been deregistered
        if let Some(vertex_data) = self.vertices.get_mut(vertex_entity) {
            vertex_data.remove_edge(edge_entity);
        }
    }

    pub(crate) fn vertex_get_faces(&self, vertex_entity: &Entity) -> Option<&HashSet<IconFaceKey>> {
        self.vertices
            .get(vertex_entity)
            .map(|data| &data.faces)
    }

    pub(crate) fn vertex_add_face(&mut self, vertex_entity: &Entity, face_key: IconFaceKey) {
        // at this point, vertex_entity may have already been deregistered
        if let Some(vertex_data) = self.vertices.get_mut(vertex_entity) {
            vertex_data.add_face(face_key);
        };
    }

    pub(crate) fn vertex_remove_face(&mut self, vertex_entity: &Entity, face_key: &IconFaceKey) {
        // at this point, vertex_entity may have already been deregistered
        if let Some(vertex_data) = self.vertices.get_mut(vertex_entity) {
            vertex_data.remove_face(face_key);
        };
    }

    fn unregister_3d_vertex(&mut self, entity: &Entity) {
        self.vertices.remove(entity);
    }

    pub(crate) fn get_vertex_data(&self, entity: &Entity) -> Option<&IconVertexData> {
        self.vertices.get(entity)
    }

    pub(crate) fn get_connected_vertices(
        &self,
        vertex_entity: Entity,
    ) -> HashSet<Entity> {
        let mut set = HashSet::new();

        let Some(vertex_data) = self.vertices.get(&vertex_entity) else {
            panic!("Vertex entity: `{:?}` has not been registered", vertex_entity);
        };
        let edges = &vertex_data.edges;
        for edge_entity in edges {
            let (vertex_entity_a, vertex_entity_b) = self.edge_get_endpoints(edge_entity);

            if vertex_entity_a != vertex_entity {
                set.insert(vertex_entity_a);
            } else if vertex_entity_b != vertex_entity {
                set.insert(vertex_entity_b);
            }
        }

        set
    }

    // Edges
    pub fn create_networked_edge(
        &mut self,
        commands: &mut Commands,
        client: &mut Client,
        camera_manager: &mut CameraManager,
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
        shape_color_resync_events: &mut EventWriter<ShapeColorResyncEvent>,
        vertex_entity_a: Entity,
        vertex_entity_b: Entity,
        file_entity: Entity,
        entities_to_release: &mut Vec<Entity>,
    ) -> Entity {
        // create new edge
        let mut new_edge_component = IconEdge::new();
        new_edge_component
            .start
            .set(client, &vertex_entity_a);
        new_edge_component
            .end
            .set(client, &vertex_entity_b);
        let mut owned_by_file_component = OwnedByFile::new();
        owned_by_file_component
            .file_entity
            .set(client, &file_entity);
        let new_edge_entity = commands
            .spawn_empty()
            .enable_replication(client)
            .configure_replication(ReplicationConfig::Delegated)
            .insert(new_edge_component)
            .insert(owned_by_file_component)
            .id();

        // add local components to edge
        self.edge_postprocess(
            commands,
            meshes,
            materials,
            camera_manager,
            Some(shape_color_resync_events),
            new_edge_entity,
            vertex_entity_a,
            vertex_entity_b,
            Some(file_entity),
            Vertex2d::ENABLED_COLOR,
        );

        entities_to_release.push(new_edge_entity);

        new_edge_entity
    }

    pub fn edge_postprocess(
        &mut self,
        commands: &mut Commands,
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
        camera_manager: &CameraManager,
        shape_color_resync_events_opt: Option<&mut EventWriter<ShapeColorResyncEvent>>,
        edge_entity: Entity,
        vertex_entity_a: Entity,
        vertex_entity_b: Entity,
        ownership_opt: Option<Entity>,
        color: Color,
    ) {
        if let Some(shape_color_resync_events) = shape_color_resync_events_opt {
            // send shape color resync event
            shape_color_resync_events.send(ShapeColorResyncEvent);
        }

        // edge
        let shape_components = create_2d_edge_line(
            meshes,
            materials,
            Vec2::ZERO,
            Vec2::X,
            0.0,
            color,
            Edge2dLocal::NORMAL_THICKNESS,
        );
        commands
            .entity(edge_entity)
            .insert(shape_components)
            .insert(camera_manager.layer_2d)
            .insert(IconEdgeLocal::new(vertex_entity_a, vertex_entity_b))
            .id();
        if let Some(file_entity) = ownership_opt {
            commands
                .entity(edge_entity)
                .insert(OwnedByFileLocal::new(file_entity));
        }

        // register 3d & 2d edges together
        self.register_edge(
            edge_entity,
            vertex_entity_a,
            vertex_entity_b,
            ownership_opt,
        );
    }

    pub fn register_edge(
        &mut self,
        edge_entity: Entity,
        vertex_entity_a: Entity,
        vertex_entity_b: Entity,
        ownership_opt: Option<Entity>,
    ) {
        for vertex_entity in [vertex_entity_a, vertex_entity_b] {
            self.vertex_add_edge(&vertex_entity, edge_entity);
        }

        self.edges.insert(
            edge_entity,
            IconEdgeData::new(
                vertex_entity_a,
                vertex_entity_b,
            ),
        );

        let file_entity = ownership_opt.unwrap();
        self.check_for_new_faces(
            file_entity,
            vertex_entity_a,
            vertex_entity_b,
        );
    }

    // returns (deleted edge entity 2d, Vec<(deleted face entity 2d, deleted face entity 3d)>
    pub fn cleanup_deleted_edge(
        &mut self,
        commands: &mut Commands,
        canvas: &mut Canvas,
        input_manager: &mut InputManager,
        edge_entity: &Entity,
    ) -> (Entity, Vec<Entity>) {
        let mut deleted_local_face_entities = Vec::new();
        // cleanup faces
        {
            let face_keys: Vec<IconFaceKey> = self
                .edges
                .get(edge_entity)
                .unwrap()
                .faces
                .iter()
                .copied()
                .collect();
            if !face_keys.is_empty() {

                for face_key in face_keys {
                    let local_face_entity = self.cleanup_deleted_face_key(
                        commands,
                        canvas,
                        input_manager,
                        &face_key,
                    );
                    deleted_local_face_entities.push(local_face_entity);
                }
            }
        }

        // unregister edge
        self.unregister_3d_edge(edge_entity);

        if input_manager.hovered_entity == Some((*edge_entity, CanvasShape::Edge)) {
            input_manager.hovered_entity = None;
        }

        canvas.queue_resync_shapes();

        (*edge_entity, deleted_local_face_entities)
    }

    pub(crate) fn has_edge_entity(&self, edge_entity: &Entity) -> bool {
        self.edges.contains_key(edge_entity)
    }

    pub(crate) fn edge_connected_faces(&self, edge_entity: &Entity) -> Option<Vec<IconFaceKey>> {
        self.edges
            .get(edge_entity)
            .map(|data| data.faces.iter().copied().collect())
    }

    pub(crate) fn edge_add_face(&mut self, edge_entity: &Entity, face_key: IconFaceKey) {
        self.edges
            .get_mut(edge_entity)
            .unwrap()
            .faces
            .insert(face_key);
    }

    pub(crate) fn edge_remove_face(&mut self, edge_entity: &Entity, face_key: &IconFaceKey) {
        self.edges
            .get_mut(edge_entity)
            .unwrap()
            .faces
            .remove(face_key);
    }

    pub(crate) fn edge_get_endpoints(&self, edge_entity: &Entity) -> (Entity, Entity) {
        let edge_data = self.edges.get(edge_entity).unwrap();
        (edge_data.vertex_entity_a, edge_data.vertex_entity_b)
    }

    fn unregister_3d_edge(
        &mut self,
        edge_entity: &Entity,
    ) {
        if let Some(edge_data) = self.edges.remove(edge_entity) {
            // remove edge from vertices
            for vertex_entity in [
                edge_data.vertex_entity_a,
                edge_data.vertex_entity_b,
            ] {
                self.vertex_remove_edge(&vertex_entity, edge_entity);
            }
        }
    }

    pub(crate) fn edge_entity_from_vertices(
        &self,
        vertex_a: Entity,
        vertex_b: Entity,
    ) -> Option<Entity> {
        let vertex_a_edges = self.vertex_get_edges(&vertex_a)?;
        let vertex_b_edges = self.vertex_get_edges(&vertex_b)?;
        let intersecting_edge_entity = vertex_a_edges.intersection(&vertex_b_edges).next()?;
        Some(*intersecting_edge_entity)
    }

    // Faces
}