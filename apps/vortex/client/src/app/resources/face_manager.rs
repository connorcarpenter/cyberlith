use std::collections::HashMap;

use bevy_ecs::{
    entity::Entity,
    system::{Commands, Query, Res, ResMut, Resource, SystemState},
    world::World,
};
use bevy_log::{info, warn};

use naia_bevy_client::{Client, CommandsExt, ReplicationConfig};

use math::{Vec2, Vec3};
use render_api::{
    base::{CpuMaterial, CpuMesh},
    components::{RenderObjectBundle, Transform, Visibility},
    shapes::{HollowTriangle, Triangle},
    Assets,
};

use vortex_proto::components::{Face3d, FileExtension, OwnedByFile};

use crate::app::{
    components::{DefaultDraw, Face3dLocal, FaceIcon2d, OwnedByFileLocal},
    resources::{
        camera_manager::CameraManager,
        canvas::Canvas,
        edge_manager::EdgeManager,
        input_manager::InputManager,
        shape_data::{CanvasShape, FaceData, FaceKey},
        vertex_manager::VertexManager,
    },
};

#[derive(Resource)]
pub struct FaceManager {
    resync: bool,
    // 3d face key -> 3d face entity
    pub(crate) face_keys: HashMap<FaceKey, Option<FaceData>>,
    // 3d face entity -> 3d face data
    faces_3d: HashMap<Entity, FaceKey>,
    // 2d face entity -> 3d face entity
    faces_2d: HashMap<Entity, FaceKey>,
    // queue of new faces to process
    new_face_keys: Vec<(FaceKey, Entity)>,
}

impl Default for FaceManager {
    fn default() -> Self {
        Self {
            resync: false,
            new_face_keys: Vec::new(),
            face_keys: HashMap::new(),
            faces_2d: HashMap::new(),
            faces_3d: HashMap::new(),
        }
    }
}

impl FaceManager {
    pub fn queue_resync(&mut self) {
        self.resync = true;
    }

    pub fn sync_2d_faces(
        &mut self,
        file_ext: FileExtension,
        face_2d_q: &Query<(Entity, &FaceIcon2d)>,
        transform_q: &mut Query<&mut Transform>,
        visibility_q: &mut Query<&mut Visibility>,
        camera_3d_scale: f32,
    ) {
        if !self.resync {
            return;
        }

        self.resync = false;

        let face_2d_scale = FaceIcon2d::SIZE * camera_3d_scale;

        for (face_2d_entity, face_icon) in face_2d_q.iter() {
            // check visibility
            let Ok(mut visibility) = visibility_q.get_mut(face_2d_entity) else {
                panic!("entity has no Visibility");
            };
            if !visibility.visible {
                continue;
            }
            if file_ext == FileExtension::Skin {
                let face_key = self.face_key_from_2d_entity(&face_2d_entity).unwrap();
                let Some(Some(face_data)) = self.face_keys.get(&face_key) else {
                    panic!("FaceKey: `{:?}` has not been registered", face_key);
                };
                if face_data.entity_3d.is_none() {
                    visibility.visible = false;
                }
            }

            // find center of all of face_icon's vertices
            let Ok(vertex_a_transform) = transform_q.get(face_icon.vertex_2d_a()) else {
                warn!("Face entity {:?}'s vertex_a has no transform", face_2d_entity);
                continue;
            };
            let Ok(vertex_b_transform) = transform_q.get(face_icon.vertex_2d_b()) else {
                warn!("Face entity {:?}'s vertex_b has no transform", face_2d_entity);
                continue;
            };
            let Ok(vertex_c_transform) = transform_q.get(face_icon.vertex_2d_c()) else {
                warn!("Face entity {:?}'s vertex_c has no transform", face_2d_entity);
                continue;
            };

            let center_translation = Vec3::new(
                (vertex_a_transform.translation.x
                    + vertex_b_transform.translation.x
                    + vertex_c_transform.translation.x)
                    / 3.0,
                (vertex_a_transform.translation.y
                    + vertex_b_transform.translation.y
                    + vertex_c_transform.translation.y)
                    / 3.0,
                (vertex_a_transform.translation.z
                    + vertex_b_transform.translation.z
                    + vertex_c_transform.translation.z)
                    / 3.0,
            );

            if let Ok(mut face_transform) = transform_q.get_mut(face_2d_entity) {
                face_transform.translation = center_translation;
                face_transform.scale = Vec3::splat(face_2d_scale);
            } else {
                warn!("Face entity {:?} has no transform", face_2d_entity);
            }
        }
    }

    pub fn process_new_faces(
        &mut self,
        commands: &mut Commands,
        canvas: &mut Canvas,
        camera_manager: &CameraManager,
        vertex_manager: &mut VertexManager,
        edge_manager: &mut EdgeManager,
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
    ) {
        if self.new_face_keys.is_empty() {
            return;
        }

        let keys = std::mem::take(&mut self.new_face_keys);
        for (face_key, file_entity) in keys {
            self.process_new_face(
                commands,
                camera_manager,
                vertex_manager,
                edge_manager,
                meshes,
                materials,
                file_entity,
                &face_key,
            );
        }

        canvas.queue_resync_shapes();
    }

    // return face 2d entity
    pub fn process_new_face(
        &mut self,
        commands: &mut Commands,
        camera_manager: &CameraManager,
        vertex_manager: &mut VertexManager,
        edge_manager: &mut EdgeManager,
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
        file_entity: Entity,
        face_key: &FaceKey,
    ) -> Entity {
        if self.has_2d_face(face_key) {
            panic!("face key already registered! `{:?}`", face_key);
        }
        info!("processing new face: `{:?}`", face_key);
        let vertex_3d_a = face_key.vertex_3d_a;
        let vertex_3d_b = face_key.vertex_3d_b;
        let vertex_3d_c = face_key.vertex_3d_c;

        // 2d face needs to have it's own button mesh, matching the 2d vertices
        let vertex_2d_a = vertex_manager.vertex_entity_3d_to_2d(&vertex_3d_a).unwrap();
        let vertex_2d_b = vertex_manager.vertex_entity_3d_to_2d(&vertex_3d_b).unwrap();
        let vertex_2d_c = vertex_manager.vertex_entity_3d_to_2d(&vertex_3d_c).unwrap();

        let entity_2d = commands
            .spawn_empty()
            .insert(FaceIcon2d::new(vertex_2d_a, vertex_2d_b, vertex_2d_c))
            .insert(RenderObjectBundle::equilateral_triangle(
                meshes,
                materials,
                Vec2::ZERO,
                FaceIcon2d::SIZE,
                FaceIcon2d::COLOR,
                Some(1),
            ))
            .insert(camera_manager.layer_2d)
            .insert(DefaultDraw)
            .id();

        info!("spawned 2d face entity: {:?}", entity_2d);

        info!(
            "adding OwnedByFile({:?}) to entity {:?}",
            file_entity, entity_2d
        );
        commands
            .entity(entity_2d)
            .insert(OwnedByFileLocal::new(file_entity));

        // add face to vertex data
        for vertex_3d_entity in [&vertex_3d_a, &vertex_3d_b, &vertex_3d_c] {
            vertex_manager.vertex_add_face(vertex_3d_entity, *face_key)
        }

        // add face to edge data
        let mut edge_entities = Vec::new();
        for (vert_a, vert_b) in [
            (&vertex_3d_a, &vertex_3d_b),
            (&vertex_3d_b, &vertex_3d_c),
            (&vertex_3d_c, &vertex_3d_a),
        ] {
            // find edge in common
            let vertex_a_edges = vertex_manager.vertex_get_edges(vert_a).unwrap();
            let vertex_b_edges = vertex_manager.vertex_get_edges(vert_b).unwrap();
            let intersection = vertex_a_edges.intersection(vertex_b_edges);
            let mut found_edge = false;
            for edge_entity in intersection {
                if found_edge {
                    panic!("should only be one edge between any two vertices!");
                }
                found_edge = true;

                edge_manager.edge_add_face(edge_entity, *face_key);

                edge_entities.push(*edge_entity);
            }
        }

        // register face data
        self.face_keys.insert(
            *face_key,
            Some(FaceData::new(
                entity_2d,
                file_entity,
                edge_entities[0],
                edge_entities[1],
                edge_entities[2],
            )),
        );
        self.faces_2d.insert(entity_2d, *face_key);

        entity_2d
    }

    pub fn create_networked_face_from_world(&mut self, world: &mut World, face_2d_entity: Entity) {
        let Some(face_3d_key) = self.face_key_from_2d_entity(&face_2d_entity) else {
            panic!(
                "Face2d entity: `{:?}` has no corresponding FaceKey",
                face_2d_entity
            );
        };
        let Some(Some(face_3d_data)) = self.face_keys.get(&face_3d_key) else {
            panic!(
                "Face3d entity: `{:?}` has not been registered",
                face_3d_key
            );
        };
        if face_3d_data.entity_3d.is_some() {
            panic!("already create face 3d entity! cannot do this twice!");
        }

        let mut system_state: SystemState<(
            Commands,
            Client,
            Res<CameraManager>,
            ResMut<Assets<CpuMesh>>,
            ResMut<Assets<CpuMaterial>>,
            Query<&Transform>,
        )> = SystemState::new(world);
        let (mut commands, mut client, camera_manager, mut meshes, mut materials, transform_q) =
            system_state.get_mut(world);

        self.create_networked_face(
            &mut commands,
            &mut client,
            &mut meshes,
            &mut materials,
            &camera_manager,
            &transform_q,
            &face_3d_key,
            [
                face_3d_data.edge_3d_a,
                face_3d_data.edge_3d_b,
                face_3d_data.edge_3d_c,
            ],
            face_3d_data.file_entity,
        );

        system_state.apply(world);
    }

    pub fn create_networked_face(
        &mut self,
        commands: &mut Commands,
        client: &mut Client,
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
        camera_manager: &CameraManager,
        transform_q: &Query<&Transform>,
        face_key: &FaceKey,
        edge_3d_entities: [Entity; 3],
        file_entity: Entity,
    ) {
        // get 3d vertex entities & positions
        let mut positions = [Vec3::ZERO, Vec3::ZERO, Vec3::ZERO];
        let mut vertex_3d_entities = [
            Entity::PLACEHOLDER,
            Entity::PLACEHOLDER,
            Entity::PLACEHOLDER,
        ];

        for (index, vertex_3d_entity) in [
            face_key.vertex_3d_a,
            face_key.vertex_3d_b,
            face_key.vertex_3d_c,
        ]
        .iter()
        .enumerate()
        {
            let vertex_transform = transform_q.get(*vertex_3d_entity).unwrap();
            positions[index] = vertex_transform.translation;
            vertex_3d_entities[index] = *vertex_3d_entity;
        }

        // possibly reorder vertices to be counter-clockwise with respect to camera
        let camera_3d_entity = camera_manager.camera_3d_entity().unwrap();
        let camera_transform = transform_q.get(camera_3d_entity).unwrap();
        if math::reorder_triangle_winding(&mut positions, camera_transform.translation, true) {
            vertex_3d_entities.swap(1, 2);
        }

        // set up networked face component
        let mut face_3d_component = Face3d::new();
        face_3d_component
            .vertex_a
            .set(client, &vertex_3d_entities[0]);
        face_3d_component
            .vertex_b
            .set(client, &vertex_3d_entities[1]);
        face_3d_component
            .vertex_c
            .set(client, &vertex_3d_entities[2]);
        face_3d_component.edge_a.set(client, &edge_3d_entities[0]);
        face_3d_component.edge_b.set(client, &edge_3d_entities[1]);
        face_3d_component.edge_c.set(client, &edge_3d_entities[2]);

        // get owned_by_file component
        let mut owned_by_file_component = OwnedByFile::new();
        owned_by_file_component
            .file_entity
            .set(client, &file_entity);

        // set up 3d entity
        let face_3d_entity = commands
            .spawn_empty()
            .enable_replication(client)
            .configure_replication(ReplicationConfig::Delegated)
            .insert(owned_by_file_component)
            .insert(OwnedByFileLocal::new(file_entity))
            .insert(face_3d_component)
            .id();

        self.face_3d_postprocess(
            commands,
            meshes,
            materials,
            &camera_manager,
            face_key,
            face_3d_entity,
            positions,
        );
    }

    pub fn face_3d_postprocess(
        &mut self,
        commands: &mut Commands,
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
        camera_manager: &CameraManager,
        face_key: &FaceKey,
        face_3d_entity: Entity,
        positions: [Vec3; 3],
    ) {
        commands
            .entity(face_3d_entity)
            .insert(RenderObjectBundle::world_triangle(
                meshes,
                materials,
                positions,
                Face3dLocal::COLOR,
            ))
            .insert(camera_manager.layer_3d)
            .insert(Face3dLocal)
            .insert(DefaultDraw);

        self.register_3d_face(face_3d_entity, face_key);

        // change 2d icon to use non-hollow triangle
        let face_2d_entity = self.face_2d_entity_from_face_key(&face_key).unwrap();
        commands
            .entity(face_2d_entity)
            .insert(meshes.add(Triangle::new_2d_equilateral()));
    }

    // returns 2d face entity
    pub fn register_3d_face(&mut self, entity_3d: Entity, face_key: &FaceKey) {
        self.faces_3d.insert(entity_3d, *face_key);

        let Some(Some(face_3d_data)) = self.face_keys.get_mut(face_key) else {
            panic!("Face3d key: `{:?}` has not been registered", face_key);
        };
        face_3d_data.entity_3d = Some(entity_3d);
    }

    pub fn remove_new_face_key(&mut self, face_key: &FaceKey) {
        self.new_face_keys.retain(|(key, _)| key != face_key);
    }

    pub(crate) fn cleanup_deleted_face_3d(
        &mut self,
        commands: &mut Commands,
        meshes: &mut Assets<CpuMesh>,
        face_3d_entity: &Entity,
    ) {
        // unregister face
        if let Some(face_2d_entity) = self.unregister_3d_face(face_3d_entity) {
            commands
                .entity(face_2d_entity)
                .insert(meshes.add(HollowTriangle::new_2d_equilateral()));
        }
    }

    // returns face 2d entity
    pub(crate) fn cleanup_deleted_face_key(
        &mut self,
        commands: &mut Commands,
        canvas: &mut Canvas,
        input_manager: &mut InputManager,
        vertex_manager: &mut VertexManager,
        edge_manager: &mut EdgeManager,
        face_key: &FaceKey,
    ) -> Entity {
        // unregister face
        let Some(face_2d_entity) = self.unregister_face_key(vertex_manager, edge_manager, face_key) else {
            panic!(
                "FaceKey: `{:?}` has no corresponding Face2d entity",
                face_key
            );
        };

        // despawn 2d face
        info!("despawn 2d face {:?}", face_2d_entity);
        commands.entity(face_2d_entity).despawn();

        if input_manager.hovered_entity == Some((face_2d_entity, CanvasShape::Face)) {
            input_manager.hovered_entity = None;
        }

        canvas.queue_resync_shapes();

        face_2d_entity
    }

    pub(crate) fn has_2d_face(&self, face_key: &FaceKey) -> bool {
        if let Some(Some(_)) = self.face_keys.get(face_key) {
            return true;
        }
        return false;
    }

    pub(crate) fn has_face_entity_3d(&self, entity_3d: &Entity) -> bool {
        self.faces_3d.contains_key(entity_3d)
    }

    pub(crate) fn face_entity_2d_to_3d(&self, entity_2d: &Entity) -> Option<Entity> {
        let Some(face_key) = self.faces_2d.get(entity_2d) else {
            return None;
        };
        let Some(Some(face_3d_data)) = self.face_keys.get(face_key) else {
            return None;
        };
        face_3d_data.entity_3d
    }

    pub(crate) fn face_entity_3d_to_2d(&self, entity_3d: &Entity) -> Option<Entity> {
        let Some(face_key) = self.faces_3d.get(entity_3d) else {
            return None;
        };
        self.face_2d_entity_from_face_key(face_key)
    }

    pub(crate) fn face_2d_entity_from_face_key(&self, face_key: &FaceKey) -> Option<Entity> {
        let Some(Some(face_3d_data)) = self.face_keys.get(face_key) else {
            return None;
        };
        Some(face_3d_data.entity_2d)
    }

    fn face_key_from_2d_entity(&self, entity_2d: &Entity) -> Option<FaceKey> {
        self.faces_2d.get(entity_2d).copied()
    }

    pub(crate) fn face_3d_entity_from_face_key(&self, face_key: &FaceKey) -> Option<Entity> {
        let Some(Some(face_3d_data)) = self.face_keys.get(face_key) else {
            return None;
        };
        face_3d_data.entity_3d
    }

    // returns 2d face entity
    fn unregister_face_key(
        &mut self,
        vertex_manager: &mut VertexManager,
        edge_manager: &mut EdgeManager,
        face_key: &FaceKey,
    ) -> Option<Entity> {
        info!("unregistering face key: `{:?}`", face_key);
        if let Some(Some(face_3d_data)) = self.face_keys.remove(&face_key) {
            let entity_2d = face_3d_data.entity_2d;
            self.faces_2d.remove(&entity_2d);

            // remove face from vertices
            for vertex_3d_entity in [
                face_key.vertex_3d_a,
                face_key.vertex_3d_b,
                face_key.vertex_3d_c,
            ] {
                vertex_manager.vertex_remove_face(&vertex_3d_entity, face_key);
            }

            // remove face from edges
            for edge_3d_entity in [
                face_3d_data.edge_3d_a,
                face_3d_data.edge_3d_b,
                face_3d_data.edge_3d_c,
            ] {
                edge_manager.edge_remove_face(&edge_3d_entity, face_key);
            }

            return Some(entity_2d);
        } else {
            return None;
        }
    }

    // returns 2d face entity
    fn unregister_3d_face(&mut self, entity_3d: &Entity) -> Option<Entity> {
        info!("unregistering 3d face entity: `{:?}`", entity_3d);
        let Some(face_key) = self.faces_3d.remove(entity_3d) else {
            panic!("no face 3d found for entity {:?}", entity_3d);
        };

        if let Some(Some(face_3d_data)) = self.face_keys.get_mut(&face_key) {
            face_3d_data.entity_3d = None;
            info!("remove entity 3d: `{:?}` from face 3d data", entity_3d);

            let face_2d_entity = face_3d_data.entity_2d;
            return Some(face_2d_entity);
        }

        return None;
    }

    pub(crate) fn check_for_new_faces(
        &mut self,
        vertex_manager: &VertexManager,
        edge_manager: &EdgeManager,
        vertex_a_3d_entity: Entity,
        vertex_b_3d_entity: Entity,
        file_entity: Entity,
    ) {
        let vertex_a_connected_vertices =
            vertex_manager.get_connected_vertices(edge_manager, vertex_a_3d_entity);
        let vertex_b_connected_vertices =
            vertex_manager.get_connected_vertices(edge_manager, vertex_b_3d_entity);

        let common_vertices =
            vertex_a_connected_vertices.intersection(&vertex_b_connected_vertices);
        for common_vertex in common_vertices {
            let face_key = FaceKey::new(vertex_a_3d_entity, vertex_b_3d_entity, *common_vertex);
            if !self.face_keys.contains_key(&face_key) {
                self.face_keys.insert(face_key, None);
                self.new_face_keys.push((face_key, file_entity));
            }
        }
    }
}
