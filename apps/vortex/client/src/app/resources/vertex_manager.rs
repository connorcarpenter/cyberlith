use std::collections::{HashMap, HashSet};

use bevy_ecs::{
    entity::Entity,
    system::{Commands, Query, Resource},
};
use bevy_log::{info, warn};

use naia_bevy_client::{Client, CommandsExt, Replicate, ReplicationConfig};

use math::{convert_3d_to_2d, Vec2, Vec3};
use render_api::{
    base::{Color, CpuMaterial, CpuMesh},
    components::{Camera, CameraProjection, Projection, RenderObjectBundle, Transform},
    Assets, Handle,
};

use vortex_proto::components::{
    Face3d, FileType, FileTypeValue, OwnedByFile, Vertex3d, VertexRoot,
};

use crate::app::{
    components::{Edge3dLocal, LocalShape, OwnedByFileLocal, Vertex2d, VertexEntry},
    resources::{
        action::{ActionStack, ShapeAction},
        camera_manager::CameraManager,
        edge_manager::EdgeManager,
        face_manager::FaceManager,
        shape_data::{CanvasShape, FaceKey, Vertex3dData},
        shape_manager::ShapeManager,
    },
};
use crate::app::resources::canvas::Canvas;
use crate::app::resources::input_manager::InputManager;

#[derive(Resource)]
pub struct VertexManager {
    // 3d vertex entity -> 3d vertex data
    pub(crate) vertices_3d: HashMap<Entity, Vertex3dData>,
    // 2d vertex entity -> 3d vertex entity
    vertices_2d: HashMap<Entity, Entity>,

    pub(crate) last_vertex_dragged: Option<(Entity, Vec3, Vec3)>,
}

impl Default for VertexManager {
    fn default() -> Self {
        Self {
            vertices_3d: HashMap::new(),
            vertices_2d: HashMap::new(),
            last_vertex_dragged: None,
        }
    }
}

impl VertexManager {
    pub fn sync_vertices(
        &self,
        camera_3d_entity: &Entity,
        camera_3d_scale: f32,
        camera_q: &Query<(&Camera, &Projection)>,
        vertex_3d_q: &mut Query<(Entity, &mut Vertex3d)>,
        transform_q: &mut Query<&mut Transform>,
        owned_by_q: &Query<&OwnedByFileLocal>,
        compass_q: &Query<&LocalShape>,
        current_tab_file_entity: Entity,
    ) {
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

        for (vertex_3d_entity, vertex_3d) in vertex_3d_q.iter() {
            // check if vertex is owned by the current tab
            if !ShapeManager::is_owned_by_tab_or_unowned(
                current_tab_file_entity,
                owned_by_q,
                vertex_3d_entity,
            ) {
                continue;
            }

            // get transform
            let Ok(mut vertex_3d_transform) = transform_q.get_mut(vertex_3d_entity) else {
                warn!("Vertex3d entity {:?} has no Transform", vertex_3d_entity);
                continue;
            };

            // update 3d vertices
            vertex_3d_transform.translation.x = vertex_3d.x().into();
            vertex_3d_transform.translation.y = vertex_3d.y().into();
            vertex_3d_transform.translation.z = vertex_3d.z().into();

            if compass_q.get(vertex_3d_entity).is_ok() {
                vertex_3d_transform.scale = Vec3::splat(compass_vertex_3d_scale);
            } else {
                // vertex_3d_transform.scale = should put 3d vertex scale here?
            }

            // update 2d vertices
            let (coords, depth) = convert_3d_to_2d(
                &view_matrix,
                &projection_matrix,
                &camera_viewport.size_vec2(),
                &vertex_3d_transform.translation,
            );

            let Some(vertex_2d_entity) = self.vertex_entity_3d_to_2d(&vertex_3d_entity) else {
                panic!("Vertex3d entity {:?} has no corresponding Vertex2d entity", vertex_3d_entity);
            };
            let Ok(mut vertex_2d_transform) = transform_q.get_mut(vertex_2d_entity) else {
                panic!("Vertex2d entity {:?} has no Transform", vertex_2d_entity);
            };

            vertex_2d_transform.translation.x = coords.x;
            vertex_2d_transform.translation.y = coords.y;
            vertex_2d_transform.translation.z = depth;

            // update 2d compass
            if compass_q.get(vertex_2d_entity).is_ok() {
                vertex_2d_transform.scale = Vec3::splat(compass_vertex_2d_scale);
            } else {
                vertex_2d_transform.scale = Vec3::splat(vertex_2d_scale);
            }
        }
    }

    pub fn create_networked_vertex(
        &mut self,
        commands: &mut Commands,
        client: &mut Client,
        camera_manager: &mut CameraManager,
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
        position: Vec3,
        file_entity: Entity,
        file_type: FileTypeValue,
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
            false,
            Some(file_entity),
            Vertex2d::CHILD_COLOR,
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
        parent_vertex_2d_entity: Entity,
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
                    position,
                    file_entity,
                    FileTypeValue::Skel,
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
                parent_vertex_2d_entity,
                new_child_vertex_2d_entity,
                new_child_vertex_3d_entity,
                file_entity,
                FileTypeValue::Skel,
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
                    new_child_vertex_2d_entity,
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
        is_root: bool,
        ownership_opt: Option<Entity>,
        color: Color,
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

        if let Some(file_entity) = ownership_opt {
            info!(
                "adding OwnedByFileLocal({:?}) to entity 2d: `{:?}` & 3d: `{:?}`",
                file_entity, vertex_2d_entity, vertex_3d_entity,
            );
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
        self.register_3d_vertex(vertex_3d_entity, vertex_2d_entity);

        vertex_2d_entity
    }

    pub fn register_3d_vertex(&mut self, entity_3d: Entity, entity_2d: Entity) {
        self.vertices_3d
            .insert(entity_3d, Vertex3dData::new(entity_2d));
        self.vertices_2d.insert(entity_2d, entity_3d);
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
        let new_vertex_2d_entity = self.vertex_3d_postprocess(
            commands,
            meshes,
            materials,
            camera_manager,
            new_vertex_3d_entity,
            parent_vertex_2d_entity_opt.is_none(),
            None,
            color,
        );

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
                new_edge_3d_entity,
                parent_vertex_2d_entity,
                parent_vertex_3d_entity,
                new_vertex_2d_entity,
                new_vertex_3d_entity,
                None,
                color,
                false,
                None,
            );
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

    pub(crate) fn vertex_connected_edges(&self, vertex_3d_entity: &Entity) -> Option<Vec<Entity>> {
        self.vertices_3d
            .get(vertex_3d_entity)
            .map(|data| data.edges_3d.iter().map(|e| *e).collect())
    }

    pub(crate) fn vertex_connected_faces(&self, vertex_3d_entity: &Entity) -> Option<Vec<FaceKey>> {
        self.vertices_3d
            .get(vertex_3d_entity)
            .map(|data| data.faces_3d.iter().copied().collect())
    }

    // returns 2d vertex entity
    fn unregister_3d_vertex(&mut self, entity_3d: &Entity) -> Option<Entity> {
        if let Some(data) = self.vertices_3d.remove(entity_3d) {
            let entity_2d = data.entity_2d;
            self.vertices_2d.remove(&entity_2d);
            return Some(entity_2d);
        }
        return None;
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
            let edge_data = edge_manager.edges_3d.get(edge_entity).unwrap();
            let vertex_a_3d_entity = edge_data.vertex_a_3d_entity;
            let vertex_b_3d_entity = edge_data.vertex_b_3d_entity;

            if vertex_a_3d_entity != vertex_3d_entity {
                set.insert(vertex_a_3d_entity);
            } else if vertex_b_3d_entity != vertex_3d_entity {
                set.insert(vertex_b_3d_entity);
            }
        }

        set
    }
}
