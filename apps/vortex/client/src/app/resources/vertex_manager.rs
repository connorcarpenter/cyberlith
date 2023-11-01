use std::collections::{HashMap, HashSet};

use bevy_ecs::{
    entity::Entity,
    event::EventWriter,
    system::{Commands, Query, Resource, SystemState},
    world::World,
};
use bevy_log::{info, warn};

use naia_bevy_client::{Client, CommandsExt, Replicate, ReplicationConfig};

use math::{convert_3d_to_2d, Vec2, Vec3};
use render_api::{
    base::{Color, CpuMaterial, CpuMesh},
    components::{Camera, CameraProjection, Projection, RenderObjectBundle, Transform, Visibility},
    Assets, Handle,
};

use vortex_proto::components::{
    Face3d, FileExtension, FileType, OwnedByFile, Vertex3d, VertexRoot,
};

use crate::app::{
    components::{
        DefaultDraw, Edge3dLocal, LocalShape, OwnedByFileLocal, Vertex2d,
        VertexEntry,
    },
    events::ShapeColorResyncEvent,
    resources::{
        action::{shape::ShapeAction, ActionStack},
        camera_manager::CameraManager,
        canvas::Canvas,
        edge_manager::EdgeManager,
        face_manager::FaceManager,
        input::InputManager,
        shape_data::{CanvasShape, FaceKey, Vertex3dData},
    },
};

#[derive(Resource)]
pub struct VertexManager {
    resync: bool,

    // 3d vertex entity -> 3d vertex data
    vertices_3d: HashMap<Entity, Vertex3dData>,
    // 2d vertex entity -> 3d vertex entity
    vertices_2d: HashMap<Entity, Entity>,

    drags: Vec<(Entity, Vec3, Vec3)>,
    dragging_entity: Option<Entity>,
    dragging_start: Option<Vec3>,
    dragging_end: Option<Vec3>,

    pub mat_enabled_vertex: Handle<CpuMaterial>,
    pub mat_disabled_vertex: Handle<CpuMaterial>,
    pub mat_root_vertex: Handle<CpuMaterial>,
}

impl Default for VertexManager {
    fn default() -> Self {
        Self {
            resync: false,
            vertices_3d: HashMap::new(),
            vertices_2d: HashMap::new(),
            drags: Vec::new(),
            dragging_entity: None,
            dragging_start: None,
            dragging_end: None,
            mat_enabled_vertex: Handle::default(),
            mat_disabled_vertex: Handle::default(),
            mat_root_vertex: Handle::default(),
        }
    }
}

impl VertexManager {
    pub fn setup(&mut self, materials: &mut Assets<CpuMaterial>) {
        self.mat_enabled_vertex = materials.add(Vertex2d::ENABLED_COLOR);
        self.mat_disabled_vertex = materials.add(Vertex2d::DISABLED_COLOR);
        self.mat_root_vertex = materials.add(Vertex2d::ROOT_COLOR);
    }

    pub fn queue_resync(&mut self) {
        self.resync = true;
    }

    pub fn sync_3d_vertices(&mut self, file_ext: FileExtension, world: &mut World) {
        // TODO: really should only do this if Vertex3d component was updated from server

        let mut system_state: SystemState<(
            Query<(Entity, &Vertex3d)>,
            Query<&mut Transform>,
            Query<&mut Visibility>,
            Query<&LocalShape>,
        )> = SystemState::new(world);
        let (
            vertex_3d_q,
            mut transform_q,
            mut visibility_q,
            local_shape_q,
        ) = system_state.get_mut(world);

        for (vertex_3d_entity, vertex_3d) in vertex_3d_q.iter() {
            // check visibility
            match file_ext {
                FileExtension::Skin => {
                    let mut disable = false;
                    if local_shape_q.get(vertex_3d_entity).is_err() {
                        disable = true;
                    }

                    if disable {
                        if let Ok(mut visibility) = visibility_q.get_mut(vertex_3d_entity) {
                            visibility.visible = false;
                        };
                    }
                }
                _ => {}
            }

            // get transform
            let Ok(mut vertex_3d_transform) = transform_q.get_mut(vertex_3d_entity) else {
                warn!("Vertex3d entity {:?} has no Transform", vertex_3d_entity);
                continue;
            };

            // update 3d vertices
            vertex_3d_transform.translation = vertex_3d.as_vec3();
        }
    }

    pub fn should_sync(&self) -> bool {
        self.resync
    }

    pub fn finish_resync(&mut self) {
        self.resync = false;
    }

    pub fn sync_2d_vertices(
        &mut self,
        file_ext: FileExtension,
        world: &mut World,
        camera_3d_entity: &Entity,
        camera_3d_scale: f32,
    ) {
        let mut system_state: SystemState<(
            Query<(&Camera, &Projection)>,
            Query<(Entity, &Vertex3d)>,
            Query<&mut Transform>,
            Query<&mut Visibility>,
            Query<&LocalShape>,
        )> = SystemState::new(world);
        let (camera_q, vertex_3d_q, mut transform_q, mut visibility_q, local_shape_q) =
            system_state.get_mut(world);

        let Ok((camera, camera_projection)) = camera_q.get(*camera_3d_entity) else {
            return;
        };

        let Ok(camera_transform) = transform_q.get(*camera_3d_entity) else {
            return;
        };

        let camera_viewport = camera.viewport.unwrap();
        let view_matrix = camera_transform.view_matrix();
        let projection_matrix = camera_projection.projection_matrix(&camera_viewport);
        let vertex_2d_scale = Vertex2d::RADIUS * camera_3d_scale;
        let compass_vertex_3d_scale = LocalShape::VERTEX_RADIUS / camera_3d_scale;
        let compass_vertex_2d_scale = Vertex2d::RADIUS;

        for (vertex_3d_entity, _) in vertex_3d_q.iter() {
            // get 3d transform
            let Ok(mut vertex_3d_transform) = transform_q.get_mut(vertex_3d_entity) else {
                warn!("Vertex3d entity {:?} has no Transform", vertex_3d_entity);
                continue;
            };
            let Some(vertex_2d_entity) = self.vertex_entity_3d_to_2d(&vertex_3d_entity) else {
                warn!("Vertex3d entity {:?} has no corresponding Vertex2d entity", vertex_3d_entity);
                continue;
            };
            if local_shape_q.get(vertex_3d_entity).is_ok() {
                vertex_3d_transform.scale = Vec3::splat(compass_vertex_3d_scale);
            } else {
                // vertex_3d_transform.scale = should put 3d vertex scale here?
                match file_ext {
                    FileExtension::Skin => {
                        // change visibility
                        let Ok(mut visibility) = visibility_q.get_mut(vertex_2d_entity) else {
                            panic!("Vertex2d entity {:?} has no Visibility", vertex_2d_entity);
                        };
                        visibility.visible = false;
                    }
                    _ => {}
                }
            }

            // update 2d vertices
            let (coords, depth) = convert_3d_to_2d(
                &view_matrix,
                &projection_matrix,
                &camera_viewport.size_vec2(),
                &vertex_3d_transform.translation,
            );

            let Ok(mut vertex_2d_transform) = transform_q.get_mut(vertex_2d_entity) else {
                panic!("Vertex2d entity {:?} has no Transform", vertex_2d_entity);
            };

            vertex_2d_transform.translation.x = coords.x;
            vertex_2d_transform.translation.y = coords.y;
            vertex_2d_transform.translation.z = depth;

            // update 2d compass
            vertex_2d_transform.scale = if local_shape_q.get(vertex_2d_entity).is_ok() {
                Vec3::splat(compass_vertex_2d_scale)
            } else {
                Vec3::splat(vertex_2d_scale)
            };
        }
    }

    pub fn reset_last_vertex_dragged(&mut self) {
        self.drags = Vec::new();
        self.dragging_entity = None;
        self.dragging_start = None;
        self.dragging_end = None;
    }

    pub fn update_last_vertex_dragged(
        &mut self,
        vertex_2d_entity: Entity,
        old_3d_position: Vec3,
        new_3d_position: Vec3,
    ) {
        if let Some(old_vertex_2d_entity) = self.dragging_entity {
            // already dragging an entity
            if old_vertex_2d_entity == vertex_2d_entity {
                // dragging same entity
                self.dragging_end = Some(new_3d_position);
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
                self.dragging_entity = Some(vertex_2d_entity);
                self.dragging_start = Some(old_3d_position);
                self.dragging_end = Some(new_3d_position);
            }
        } else {
            // not dragging an entity
            self.dragging_entity = Some(vertex_2d_entity);
            self.dragging_start = Some(old_3d_position);
            self.dragging_end = Some(new_3d_position);
        }
    }

    pub fn take_drags(&mut self) -> Option<Vec<(Entity, Vec3, Vec3)>> {
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
        file_type: FileExtension,
        file_entity: Entity,
        parent_vertex_3d_entity_opt: Option<Entity>,
        position: Vec3,
        entities_to_release: &mut Vec<Entity>,
    ) -> (Entity, Entity) {
        // create new 3d vertex
        let mut owned_by_file_component = OwnedByFile::new();
        owned_by_file_component
            .file_entity
            .set(client, &file_entity);
        let new_vertex_3d_entity = commands
            .spawn_empty()
            .enable_replication(client)
            .configure_replication(ReplicationConfig::Delegated)
            .insert(Vertex3d::from_vec3(position))
            .insert(owned_by_file_component)
            .insert(FileType::new(file_type))
            .id();

        entities_to_release.push(new_vertex_3d_entity);

        // create new 2d vertex, add local components to 3d vertex
        let new_vertex_2d_entity = self.vertex_3d_postprocess(
            commands,
            meshes,
            materials,
            camera_manager,
            new_vertex_3d_entity,
            parent_vertex_3d_entity_opt,
            false,
            Some(file_entity),
            Vertex2d::ENABLED_COLOR,
            file_type == FileExtension::Mesh,
        );

        return (new_vertex_2d_entity, new_vertex_3d_entity);
    }

    pub(crate) fn create_networked_children_tree(
        &mut self,
        action_stack: &mut ActionStack<ShapeAction>,
        commands: &mut Commands,
        client: &mut Client,
        camera_manager: &mut CameraManager,
        edge_manager: &mut EdgeManager,
        face_manager: &mut FaceManager,
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
        shape_color_resync_events: &mut EventWriter<ShapeColorResyncEvent>,
        parent_vertex_2d_entity: Entity,
        parent_vertex_3d_entity: Entity,
        children: Vec<VertexEntry>,
        file_entity: Entity,
        entities_to_release: &mut Vec<Entity>,
    ) {
        for child in children {
            let position = child.position();
            let grandchildren_opt = child.children();
            let old_child_vertex_3d_entity = child.entity_3d();
            let old_child_vertex_2d_entity = child.entity_2d();
            let edge_angle = child.edge_angle();

            let (new_child_vertex_2d_entity, new_child_vertex_3d_entity) = self
                .create_networked_vertex(
                    commands,
                    client,
                    camera_manager,
                    meshes,
                    materials,
                    FileExtension::Skel,
                    file_entity,
                    Some(parent_vertex_3d_entity),
                    position,
                    entities_to_release,
                );
            action_stack.migrate_vertex_entities(
                old_child_vertex_2d_entity,
                new_child_vertex_2d_entity,
                old_child_vertex_3d_entity,
                new_child_vertex_3d_entity,
            );
            edge_manager.create_networked_edge(
                commands,
                client,
                camera_manager,
                self,
                face_manager,
                meshes,
                materials,
                shape_color_resync_events,
                parent_vertex_2d_entity,
                parent_vertex_3d_entity,
                new_child_vertex_2d_entity,
                new_child_vertex_3d_entity,
                file_entity,
                FileExtension::Skel,
                Some(edge_angle),
                entities_to_release,
            );
            if let Some(grandchildren) = grandchildren_opt {
                self.create_networked_children_tree(
                    action_stack,
                    commands,
                    client,
                    camera_manager,
                    edge_manager,
                    face_manager,
                    meshes,
                    materials,
                    shape_color_resync_events,
                    new_child_vertex_2d_entity,
                    new_child_vertex_3d_entity,
                    grandchildren,
                    file_entity,
                    entities_to_release,
                );
            }
        }
    }

    pub fn vertex_3d_postprocess(
        &mut self,
        commands: &mut Commands,
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
        camera_manager: &CameraManager,
        vertex_3d_entity: Entity,
        parent_vertex_3d_entity_opt: Option<Entity>,
        is_root: bool,
        ownership_opt: Option<Entity>,
        color: Color,
        default_draw: bool,
    ) -> Entity {
        // vertex 3d
        commands
            .entity(vertex_3d_entity)
            .insert(RenderObjectBundle::sphere(
                meshes,
                materials,
                Vec3::ZERO,
                Vertex2d::RADIUS,
                Vertex2d::SUBDIVISIONS,
                color,
            ))
            .insert(camera_manager.layer_3d);

        // vertex 2d
        let vertex_2d_entity = commands
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
            .insert(Vertex2d)
            .id();

        if default_draw {
            commands.entity(vertex_2d_entity).insert(DefaultDraw);
            commands.entity(vertex_3d_entity).insert(DefaultDraw);
        }

        if let Some(file_entity) = ownership_opt {
            // info!(
            //     "adding OwnedByFileLocal({:?}) to entity 2d: `{:?}` & 3d: `{:?}`",
            //     file_entity, vertex_2d_entity, vertex_3d_entity,
            // );
            commands
                .entity(vertex_2d_entity)
                .insert(OwnedByFileLocal::new(file_entity));
            commands
                .entity(vertex_3d_entity)
                .insert(OwnedByFileLocal::new(file_entity));
        }

        if is_root {
            commands.entity(vertex_2d_entity).insert(VertexRoot);
        }

        // info!(
        //     "created Vertex3d: `{:?}`, created 2d entity: {:?}",
        //     vertex_3d_entity, vertex_2d_entity,
        // );

        // register 3d & 2d vertices together
        self.register_3d_vertex(
            parent_vertex_3d_entity_opt,
            vertex_3d_entity,
            vertex_2d_entity,
            ownership_opt,
        );

        vertex_2d_entity
    }

    pub fn register_3d_vertex(
        &mut self,
        parent_vertex_3d_entity_opt: Option<Entity>,
        entity_3d: Entity,
        entity_2d: Entity,
        ownership_opt: Option<Entity>,
    ) {
        self.vertices_3d.insert(
            entity_3d,
            Vertex3dData::new(parent_vertex_3d_entity_opt, entity_2d, ownership_opt),
        );
        self.vertices_2d.insert(entity_2d, entity_3d);

        if let Some(parent_vertex_3d_entity) = parent_vertex_3d_entity_opt {
            let Some(parent_vertex_3d_data) = self.vertices_3d.get_mut(&parent_vertex_3d_entity) else {
                panic!("Vertex3d entity: `{:?}` has not been registered", parent_vertex_3d_entity);
            };
            parent_vertex_3d_data.add_child(entity_3d);
        }
    }

    pub fn vertex_parent_3d_entity(&self, vertex_3d_entity: &Entity) -> Option<Entity> {
        let vertex_3d_data = self.vertices_3d.get(vertex_3d_entity)?;
        vertex_3d_data.parent_3d_entity_opt
    }

    pub fn vertex_children_3d_entities(
        &self,
        vertex_3d_entity: &Entity,
    ) -> Option<&HashSet<Entity>> {
        let vertex_3d_data = self.vertices_3d.get(vertex_3d_entity)?;
        vertex_3d_data.children_3d_entities_opt.as_ref()
    }

    pub fn new_local_vertex(
        &mut self,
        commands: &mut Commands,
        camera_manager: &mut CameraManager,
        edge_manager: &mut EdgeManager,
        face_manager: &mut FaceManager,
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
        parent_vertex_2d_entity_opt: Option<Entity>,
        position: Vec3,
        color: Color,
    ) -> (Entity, Entity, Option<Entity>, Option<Entity>) {
        // vertex 3d
        let mut vertex_3d_component = Vertex3d::new(0, 0, 0);
        vertex_3d_component.localize();
        vertex_3d_component.set_vec3(&position);
        let new_vertex_3d_entity = commands.spawn_empty().insert(vertex_3d_component).id();

        // vertex 2d
        let parent_vertex_3d_entity_opt =
            parent_vertex_2d_entity_opt.map(|parent_vertex_2d_entity| {
                self.vertex_entity_2d_to_3d(&parent_vertex_2d_entity)
                    .unwrap()
            });
        let new_vertex_2d_entity = self.vertex_3d_postprocess(
            commands,
            meshes,
            materials,
            camera_manager,
            new_vertex_3d_entity,
            parent_vertex_3d_entity_opt,
            false,
            None,
            color,
            true,
        );

        commands.entity(new_vertex_2d_entity).insert(LocalShape);
        commands.entity(new_vertex_3d_entity).insert(LocalShape);

        let mut new_edge_2d_entity_opt = None;
        let mut new_edge_3d_entity_opt = None;

        if let Some(parent_vertex_2d_entity) = parent_vertex_2d_entity_opt {
            // edge 3d
            let parent_vertex_3d_entity = self
                .vertex_entity_2d_to_3d(&parent_vertex_2d_entity)
                .unwrap();
            let new_edge_3d_entity = commands
                .spawn_empty()
                .insert(Edge3dLocal::new(
                    parent_vertex_3d_entity,
                    new_vertex_3d_entity,
                ))
                .id();

            // edge 2d
            let new_edge_2d_entity = edge_manager.edge_3d_postprocess(
                commands,
                meshes,
                materials,
                camera_manager,
                self,
                face_manager,
                None,
                new_edge_3d_entity,
                parent_vertex_2d_entity,
                parent_vertex_3d_entity,
                new_vertex_2d_entity,
                new_vertex_3d_entity,
                None,
                color,
                false,
                None,
                true,
            );

            commands.entity(new_edge_2d_entity).insert(LocalShape);
            commands.entity(new_edge_3d_entity).insert(LocalShape);

            new_edge_2d_entity_opt = Some(new_edge_2d_entity);
            new_edge_3d_entity_opt = Some(new_edge_3d_entity);
        }

        return (
            new_vertex_2d_entity,
            new_vertex_3d_entity,
            new_edge_2d_entity_opt,
            new_edge_3d_entity_opt,
        );
    }

    pub fn on_vertex_3d_moved(
        &self,
        client: &Client,
        face_manager: &FaceManager,
        meshes: &mut Assets<CpuMesh>,
        mesh_handle_q: &Query<&Handle<CpuMesh>>,
        face_3d_q: &Query<&Face3d>,
        transform_q: &mut Query<&mut Transform>,
        vertex_3d_entity: &Entity,
    ) {
        let Some(vertex_3d_data) = self.vertices_3d.get(vertex_3d_entity) else {
            panic!("Vertex3d entity: `{:?}` has not been registered", vertex_3d_entity);
        };

        for face_3d_key in &vertex_3d_data.faces_3d {
            let Some(Some(face_3d_data)) = face_manager.face_keys.get(face_3d_key) else {
                panic!("Face3d key: `{:?}` has not been registered", face_3d_key);
            };
            if face_3d_data.entity_3d.is_none() {
                continue;
            }
            let face_3d_entity = face_3d_data.entity_3d.unwrap();

            // need to get vertices from Face3d component because they are in the correct order
            let Ok(face_3d) = face_3d_q.get(face_3d_entity) else {
                panic!("Face3d entity: `{:?}` has not been registered", face_3d_entity);
            };
            let vertex_3d_a = face_3d.vertex_a.get(client).unwrap();
            let vertex_3d_b = face_3d.vertex_b.get(client).unwrap();
            let vertex_3d_c = face_3d.vertex_c.get(client).unwrap();

            let mut positions = [Vec3::ZERO, Vec3::ZERO, Vec3::ZERO];
            for (index, vertex) in [vertex_3d_a, vertex_3d_b, vertex_3d_c].iter().enumerate() {
                positions[index] = transform_q.get(*vertex).unwrap().translation;
            }

            let (new_mesh, new_center) = RenderObjectBundle::world_triangle_mesh(positions);

            // update mesh
            let mesh_handle = mesh_handle_q.get(face_3d_entity).unwrap();
            meshes.set(mesh_handle, new_mesh);

            // update transform
            let mut transform = transform_q.get_mut(face_3d_entity).unwrap();
            transform.translation = new_center;
        }
    }

    // returns entity 2d
    pub fn cleanup_deleted_vertex(
        &mut self,
        commands: &mut Commands,
        canvas: &mut Canvas,
        input_manager: &mut InputManager,
        entity_3d: &Entity,
    ) -> Entity {
        // unregister vertex
        let Some(vertex_2d_entity) = self.unregister_3d_vertex(entity_3d) else {
            panic!(
                "Vertex3d entity: `{:?}` has no corresponding Vertex2d entity",
                entity_3d
            );
        };

        // despawn 2d vertex
        info!("despawn 2d vertex {:?}", vertex_2d_entity);
        commands.entity(vertex_2d_entity).despawn();

        if input_manager.hovered_entity == Some((vertex_2d_entity, CanvasShape::Vertex)) {
            input_manager.hovered_entity = None;
        }

        canvas.queue_resync_shapes();

        vertex_2d_entity
    }

    pub(crate) fn has_vertex_entity_3d(&self, entity_3d: &Entity) -> bool {
        self.vertices_3d.contains_key(entity_3d)
    }

    pub(crate) fn vertex_entity_3d_to_2d(&self, entity_3d: &Entity) -> Option<Entity> {
        self.vertices_3d.get(entity_3d).map(|data| data.entity_2d)
    }

    pub(crate) fn vertex_entity_2d_to_3d(&self, entity_2d: &Entity) -> Option<Entity> {
        self.vertices_2d.get(entity_2d).copied()
    }

    pub(crate) fn vertex_get_edges(&self, vertex_3d_entity: &Entity) -> Option<&HashSet<Entity>> {
        self.vertices_3d
            .get(vertex_3d_entity)
            .map(|data| &data.edges_3d)
    }

    pub(crate) fn vertex_add_edge(&mut self, vertex_3d_entity: &Entity, edge_3d_entity: Entity) {
        let Some(vertex_3d_data) = self.vertices_3d.get_mut(&vertex_3d_entity) else {
            panic!("Vertex3d entity: `{:?}` has not been registered", vertex_3d_entity);
        };
        vertex_3d_data.add_edge(edge_3d_entity);
    }

    pub(crate) fn vertex_remove_edge(
        &mut self,
        vertex_3d_entity: &Entity,
        edge_3d_entity: &Entity,
    ) {
        // at this point, vertex_3d_entity may have already been deregistered
        if let Some(vertex_3d_data) = self.vertices_3d.get_mut(vertex_3d_entity) {
            vertex_3d_data.remove_edge(edge_3d_entity);
        }
    }

    pub(crate) fn vertex_get_faces(&self, vertex_3d_entity: &Entity) -> Option<&HashSet<FaceKey>> {
        self.vertices_3d
            .get(vertex_3d_entity)
            .map(|data| &data.faces_3d)
    }

    pub(crate) fn vertex_add_face(&mut self, vertex_3d_entity: &Entity, face_3d_key: FaceKey) {
        // at this point, vertex_3d_entity may have already been deregistered
        if let Some(vertex_3d_data) = self.vertices_3d.get_mut(vertex_3d_entity) {
            vertex_3d_data.add_face(face_3d_key);
        };
    }

    pub(crate) fn vertex_remove_face(&mut self, vertex_3d_entity: &Entity, face_3d_key: &FaceKey) {
        // at this point, vertex_3d_entity may have already been deregistered
        if let Some(vertex_3d_data) = self.vertices_3d.get_mut(vertex_3d_entity) {
            vertex_3d_data.remove_face(face_3d_key);
        };
    }

    // returns 2d vertex entity
    fn unregister_3d_vertex(&mut self, entity_3d: &Entity) -> Option<Entity> {
        if let Some(data) = self.vertices_3d.remove(entity_3d) {
            let entity_2d = data.entity_2d;
            self.vertices_2d.remove(&entity_2d);

            if let Some(parent_3d_entity) = data.parent_3d_entity_opt {
                // at this point it's possible parent already deregistered
                if let Some(parent_3d_data) = self.vertices_3d.get_mut(&parent_3d_entity) {
                    parent_3d_data.remove_child(entity_3d);
                };
            }

            return Some(entity_2d);
        }
        return None;
    }

    pub(crate) fn get_vertex_3d_data(&self, entity: &Entity) -> Option<&Vertex3dData> {
        self.vertices_3d.get(entity)
    }

    pub(crate) fn get_connected_vertices(
        &self,
        edge_manager: &EdgeManager,
        vertex_3d_entity: Entity,
    ) -> HashSet<Entity> {
        let mut set = HashSet::new();

        let Some(vertex_data) = self.vertices_3d.get(&vertex_3d_entity) else {
            panic!("Vertex3d entity: `{:?}` has not been registered", vertex_3d_entity);
        };
        let edges = &vertex_data.edges_3d;
        for edge_entity in edges {
            let (vertex_a_3d_entity, vertex_b_3d_entity) =
                edge_manager.edge_get_endpoints(edge_entity);

            if vertex_a_3d_entity != vertex_3d_entity {
                set.insert(vertex_a_3d_entity);
            } else if vertex_b_3d_entity != vertex_3d_entity {
                set.insert(vertex_b_3d_entity);
            }
        }

        set
    }
}
