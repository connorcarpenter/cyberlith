use std::{collections::HashMap, f32::consts::FRAC_PI_2};

use bevy_ecs::{
    entity::Entity,
    system::{Commands, Query, Resource},
};
use bevy_log::{info, warn};

use naia_bevy_client::{Client, CommandsExt, ReplicationConfig};

use math::{Vec2, Vec3};
use render_api::{
    base::{Color, CpuMaterial, CpuMesh},
    components::{RenderObjectBundle, Transform, Visibility},
    shapes::{
        angle_between, get_2d_line_transform_endpoint, set_2d_line_transform,
        set_2d_line_transform_from_angle,
    },
    Assets,
};

use vortex_proto::components::{Edge3d, EdgeAngle, FileType, FileTypeValue, OwnedByFile};

use crate::app::{
    components::{Edge2dLocal, Edge3dLocal, LocalShape, OwnedByFileLocal, Vertex2d},
    resources::{
        camera_manager::CameraManager,
        canvas::Canvas,
        face_manager::FaceManager,
        input_manager::InputManager,
        shape_data::{CanvasShape, Edge3dData, FaceKey},
        shape_manager::ShapeManager,
        vertex_manager::VertexManager,
    },
    set_3d_line_transform,
    shapes::{
        create_2d_edge_arrow, create_2d_edge_line, create_3d_edge_diamond, create_3d_edge_line,
    },
};

#[derive(Resource)]
pub struct EdgeManager {
    // 3d edge entity -> 3d edge data
    pub(crate) edges_3d: HashMap<Entity, Edge3dData>,
    // 2d edge entity -> 3d edge entity
    edges_2d: HashMap<Entity, Entity>,

    pub(crate) last_edge_dragged: Option<(Entity, f32, f32)>,

    edge_angle_visibility: bool,
}

impl Default for EdgeManager {
    fn default() -> Self {
        Self {
            edges_3d: HashMap::new(),
            edges_2d: HashMap::new(),
            last_edge_dragged: None,
            edge_angle_visibility: false,
        }
    }
}

impl EdgeManager {
    pub fn sync_2d_edges(
        vertex_manager: &VertexManager,
        edge_2d_q: &Query<(Entity, &Edge2dLocal)>,
        transform_q: &mut Query<&mut Transform>,
        owned_by_q: &Query<&OwnedByFileLocal>,
        compass_q: &Query<&LocalShape>,
        current_tab_file_entity: Entity,
        camera_3d_scale: f32,
    ) {
        let edge_2d_scale = Edge2dLocal::NORMAL_THICKNESS * camera_3d_scale;

        for (edge_2d_entity, edge_endpoints) in edge_2d_q.iter() {
            let Some(end_3d_entity) = vertex_manager.vertex_entity_2d_to_3d(&edge_endpoints.end) else {
                warn!("Edge entity {:?} has no 3d endpoint entity", edge_2d_entity);
                continue;
            };

            // check if vertex is owned by the current tab
            if !ShapeManager::is_owned_by_tab_or_unowned(
                current_tab_file_entity,
                owned_by_q,
                end_3d_entity,
            ) {
                continue;
            }

            let Ok(start_transform) = transform_q.get(edge_endpoints.start) else {
                warn!(
                    "2d Edge start entity {:?} has no transform",
                    edge_endpoints.start,
                );
                continue;
            };

            let start_pos = start_transform.translation.truncate();

            let Ok(end_transform) = transform_q.get(edge_endpoints.end) else {
                warn!(
                    "2d Edge end entity {:?} has no transform",
                    edge_endpoints.end,
                );
                continue;
            };

            let end_pos = end_transform.translation.truncate();
            let depth = (start_transform.translation.z + end_transform.translation.z) / 2.0;

            let Ok(mut edge_2d_transform) = transform_q.get_mut(edge_2d_entity) else {
                warn!("2d Edge entity {:?} has no transform", edge_2d_entity);
                continue;
            };

            set_2d_line_transform(&mut edge_2d_transform, start_pos, end_pos, depth);

            if compass_q.get(edge_2d_entity).is_ok() {
                edge_2d_transform.scale.y = Edge2dLocal::NORMAL_THICKNESS;
            } else {
                edge_2d_transform.scale.y = edge_2d_scale;
            }
        }
    }

    pub fn sync_3d_edges(
        &self,
        edge_3d_q: &Query<(Entity, &Edge3dLocal, Option<&EdgeAngle>)>,
        transform_q: &mut Query<&mut Transform>,
        owned_by_q: &Query<&OwnedByFileLocal>,
        visibility_q: &mut Query<&mut Visibility>,
        compass_q: &Query<&LocalShape>,
        current_tab_file_entity: Entity,
        camera_3d_scale: f32,
    ) {
        let edge_angle_base_circle_scale =
            Edge2dLocal::EDGE_ANGLE_BASE_CIRCLE_RADIUS * camera_3d_scale;
        let edge_angle_end_circle_scale =
            Edge2dLocal::EDGE_ANGLE_END_CIRCLE_RADIUS * camera_3d_scale;
        let edge_angle_length = Edge2dLocal::EDGE_ANGLE_LENGTH * camera_3d_scale;
        let edge_angle_thickness = Edge2dLocal::EDGE_ANGLE_THICKNESS * camera_3d_scale;
        let compass_edge_3d_scale = LocalShape::EDGE_THICKNESS / camera_3d_scale;

        for (edge_entity, edge_endpoints, edge_angle_opt) in edge_3d_q.iter() {
            // check if vertex is owned by the current tab
            if !ShapeManager::is_owned_by_tab_or_unowned(
                current_tab_file_entity,
                owned_by_q,
                edge_entity,
            ) {
                continue;
            }

            let edge_angle_opt = edge_angle_opt.map(|angle| angle.get_radians());

            let edge_start_entity = edge_endpoints.start;
            let edge_end_entity = edge_endpoints.end;

            let Ok(start_transform) = transform_q.get(edge_start_entity) else {
                warn!(
                    "3d Edge start entity {:?} has no transform",
                    edge_start_entity,
                );
                continue;
            };
            let start_pos = start_transform.translation;
            let Ok(end_transform) = transform_q.get(edge_end_entity) else {
                warn!("3d Edge end entity {:?} has no transform", edge_end_entity);
                continue;
            };
            let end_pos = end_transform.translation;
            let mut edge_transform = transform_q.get_mut(edge_entity).unwrap();
            set_3d_line_transform(&mut edge_transform, start_pos, end_pos, edge_angle_opt);
            if compass_q.get(edge_entity).is_ok() {
                edge_transform.scale.x = compass_edge_3d_scale;
                edge_transform.scale.y = compass_edge_3d_scale;
            }

            // update 2d edge angle
            if let Some(edge_angle) = edge_angle_opt {
                let edge_3d_data = self.edges_3d.get(&edge_entity).unwrap();
                let (base_circle_entity, angle_edge_entity, end_circle_entity) =
                    edge_3d_data.angle_entities_opt.unwrap();
                for entity in [base_circle_entity, angle_edge_entity, end_circle_entity] {
                    let Ok(mut visibility) = visibility_q.get_mut(entity) else {
                        warn!("Edge angle entity {:?} has no transform", entity);
                        continue;
                    };
                    visibility.visible = self.edge_angle_visibility;
                }

                if self.edge_angle_visibility {
                    let edge_2d_entity = edge_3d_data.entity_2d;

                    let edge_2d_transform = transform_q.get(edge_2d_entity).unwrap();
                    let start_pos = edge_2d_transform.translation.truncate();
                    let end_pos = get_2d_line_transform_endpoint(&edge_2d_transform);
                    let base_angle = angle_between(&start_pos, &end_pos);
                    let middle_pos = (start_pos + end_pos) / 2.0;
                    let edge_depth = edge_2d_transform.translation.z;

                    let Ok(mut angle_transform) = transform_q.get_mut(angle_edge_entity) else {
                        warn!("Edge angle entity {:?} has no transform", angle_edge_entity);
                        continue;
                    };

                    let edge_angle_drawn = base_angle + edge_angle + FRAC_PI_2;
                    let edge_depth_drawn = edge_depth - 1.0;
                    set_2d_line_transform_from_angle(
                        &mut angle_transform,
                        middle_pos,
                        edge_angle_drawn,
                        edge_angle_length,
                        edge_depth_drawn,
                    );
                    angle_transform.scale.y = edge_angle_thickness;
                    let edge_angle_endpoint = get_2d_line_transform_endpoint(&angle_transform);

                    let Ok(mut base_circle_transform) = transform_q.get_mut(base_circle_entity) else {
                        warn!("Edge angle base circle entity {:?} has no transform", base_circle_entity);
                        continue;
                    };
                    base_circle_transform.translation.x = middle_pos.x;
                    base_circle_transform.translation.y = middle_pos.y;
                    base_circle_transform.translation.z = edge_depth_drawn;
                    base_circle_transform.scale = Vec3::splat(edge_angle_base_circle_scale);

                    let Ok(mut end_circle_transform) = transform_q.get_mut(end_circle_entity) else {
                        warn!("Edge angle end circle entity {:?} has no transform", end_circle_entity);
                        continue;
                    };
                    end_circle_transform.translation.x = edge_angle_endpoint.x;
                    end_circle_transform.translation.y = edge_angle_endpoint.y;
                    end_circle_transform.translation.z = edge_depth_drawn;
                    end_circle_transform.scale = Vec3::splat(edge_angle_end_circle_scale);
                }
            }
        }
    }

    // return (new edge 2d entity, new edge 3d entity)
    pub fn create_networked_edge(
        &mut self,
        commands: &mut Commands,
        client: &mut Client,
        camera_manager: &mut CameraManager,
        vertex_manager: &mut VertexManager,
        face_manager: &mut FaceManager,
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
        parent_vertex_2d_entity: Entity,
        child_vertex_2d_entity: Entity,
        child_vertex_3d_entity: Entity,
        file_entity: Entity,
        file_type: FileTypeValue,
        edge_angle: Option<f32>,
        entities_to_release: &mut Vec<Entity>,
    ) -> (Entity, Entity) {
        // create new 3d edge
        let parent_vertex_3d_entity = vertex_manager
            .vertex_entity_2d_to_3d(&parent_vertex_2d_entity)
            .unwrap();

        let mut new_edge_3d_component = Edge3d::new();
        new_edge_3d_component
            .start
            .set(client, &parent_vertex_3d_entity);
        new_edge_3d_component
            .end
            .set(client, &child_vertex_3d_entity);
        let mut owned_by_file_component = OwnedByFile::new();
        owned_by_file_component
            .file_entity
            .set(client, &file_entity);
        let new_edge_3d_entity = commands
            .spawn_empty()
            .enable_replication(client)
            .configure_replication(ReplicationConfig::Delegated)
            .insert(new_edge_3d_component)
            .insert(owned_by_file_component)
            .insert(FileType::new(file_type))
            .id();

        if file_type == FileTypeValue::Skel {
            let edge_angle_f32 = edge_angle.unwrap();
            commands
                .entity(new_edge_3d_entity)
                .insert(EdgeAngle::new(edge_angle_f32));
        }

        // create new 2d edge, add local components to 3d edge
        let new_edge_2d_entity = self.edge_3d_postprocess(
            commands,
            meshes,
            materials,
            camera_manager,
            vertex_manager,
            face_manager,
            new_edge_3d_entity,
            parent_vertex_2d_entity,
            parent_vertex_3d_entity,
            child_vertex_2d_entity,
            child_vertex_3d_entity,
            Some(file_entity),
            Vertex2d::CHILD_COLOR,
            file_type == FileTypeValue::Skel,
            edge_angle,
        );

        entities_to_release.push(new_edge_3d_entity);

        (new_edge_2d_entity, new_edge_3d_entity)
    }

    pub fn edge_3d_postprocess(
        &mut self,
        commands: &mut Commands,
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
        camera_manager: &CameraManager,
        vertex_manager: &mut VertexManager,
        face_manager: &mut FaceManager,
        edge_3d_entity: Entity,
        vertex_a_2d_entity: Entity,
        vertex_a_3d_entity: Entity,
        vertex_b_2d_entity: Entity,
        vertex_b_3d_entity: Entity,
        ownership_opt: Option<Entity>,
        color: Color,
        arrows_not_lines: bool,
        edge_angle_opt: Option<f32>,
    ) -> Entity {
        // edge 3d
        let shape_components = if arrows_not_lines {
            create_3d_edge_diamond(
                meshes,
                materials,
                Vec3::ZERO,
                Vec3::X,
                color,
                Edge3dLocal::NORMAL_THICKNESS,
            )
        } else {
            create_3d_edge_line(
                meshes,
                materials,
                Vec3::ZERO,
                Vec3::X,
                color,
                Edge3dLocal::NORMAL_THICKNESS,
            )
        };
        commands
            .entity(edge_3d_entity)
            .insert(shape_components)
            .insert(camera_manager.layer_3d)
            .insert(Edge3dLocal::new(vertex_a_3d_entity, vertex_b_3d_entity));

        // edge 2d
        let edge_2d_entity = {
            let shape_components = if arrows_not_lines {
                create_2d_edge_arrow(
                    meshes,
                    materials,
                    Vec2::ZERO,
                    Vec2::X,
                    0.0,
                    color,
                    Edge2dLocal::NORMAL_THICKNESS,
                    2.0,
                )
            } else {
                create_2d_edge_line(
                    meshes,
                    materials,
                    Vec2::ZERO,
                    Vec2::X,
                    0.0,
                    color,
                    Edge2dLocal::NORMAL_THICKNESS,
                )
            };
            let edge_2d_entity = commands
                .spawn_empty()
                .insert(shape_components)
                .insert(camera_manager.layer_2d)
                .insert(Edge2dLocal::new(vertex_a_2d_entity, vertex_b_2d_entity))
                .id();
            if let Some(file_entity) = ownership_opt {
                commands
                    .entity(edge_2d_entity)
                    .insert(OwnedByFileLocal::new(file_entity));
                commands
                    .entity(edge_3d_entity)
                    .insert(OwnedByFileLocal::new(file_entity));
            }
            edge_2d_entity
        };

        // Edge Angle
        let edge_angle_entities_opt = if let Some(_edge_angle) = edge_angle_opt {
            let shape_components = create_2d_edge_line(
                meshes,
                materials,
                Vec2::ZERO,
                Vec2::X,
                0.0,
                Color::DARK_BLUE,
                Edge2dLocal::NORMAL_THICKNESS,
            );
            let edge_angle_line_entity = commands
                .spawn_empty()
                .insert(shape_components)
                .insert(camera_manager.layer_2d)
                .id();
            let edge_angle_circle_entities = {
                let mut circle_entities = Vec::new();
                for _ in 0..2 {
                    let id = commands
                        .spawn_empty()
                        .insert(RenderObjectBundle::circle(
                            meshes,
                            materials,
                            Vec2::ZERO,
                            1.0,
                            Vertex2d::SUBDIVISIONS,
                            Color::DARK_BLUE,
                            None,
                        ))
                        .insert(camera_manager.layer_2d)
                        .id();
                    circle_entities.push(id);
                }
                circle_entities
            };
            if let Some(file_entity) = ownership_opt {
                commands
                    .entity(edge_angle_line_entity)
                    .insert(OwnedByFileLocal::new(file_entity));
                for circle_entity in edge_angle_circle_entities.iter() {
                    commands
                        .entity(*circle_entity)
                        .insert(OwnedByFileLocal::new(file_entity));
                }
            }
            Some((
                edge_angle_circle_entities[0],
                edge_angle_line_entity,
                edge_angle_circle_entities[1],
            ))
        } else {
            None
        };

        // register 3d & 2d edges together
        self.register_3d_edge(
            vertex_manager,
            face_manager,
            edge_3d_entity,
            edge_2d_entity,
            vertex_a_3d_entity,
            vertex_b_3d_entity,
            ownership_opt,
            edge_angle_entities_opt,
        );

        edge_2d_entity
    }

    pub fn register_3d_edge(
        &mut self,
        vertex_manager: &mut VertexManager,
        face_manager: &mut FaceManager,
        edge_3d_entity: Entity,
        edge_2d_entity: Entity,
        vertex_a_3d_entity: Entity,
        vertex_b_3d_entity: Entity,
        ownership_opt: Option<Entity>,
        // (base circle, line, end circle)
        angle_entities_opt: Option<(Entity, Entity, Entity)>,
    ) {
        for vertex_3d_entity in [vertex_a_3d_entity, vertex_b_3d_entity] {
            let Some(vertex_3d_data) = vertex_manager.vertices_3d.get_mut(&vertex_3d_entity) else {
                panic!("Vertex3d entity: `{:?}` has not been registered", vertex_3d_entity);
            };
            vertex_3d_data.add_edge(edge_3d_entity);
        }

        info!(
            "register_3d_edge(3d: `{:?}`, 2d: `{:?}`)",
            edge_3d_entity, edge_2d_entity
        );

        self.edges_3d.insert(
            edge_3d_entity,
            Edge3dData::new(
                edge_2d_entity,
                vertex_a_3d_entity,
                vertex_b_3d_entity,
                angle_entities_opt,
            ),
        );
        self.edges_2d.insert(edge_2d_entity, edge_3d_entity);

        if let Some(file_entity) = ownership_opt {
            face_manager.check_for_new_faces(
                vertex_manager,
                &self,
                vertex_a_3d_entity,
                vertex_b_3d_entity,
                file_entity,
            );
        }
    }

    // returns (deleted edge entity 2d, Vec<(deleted face entity 2d, deleted face entity 3d)>
    pub fn cleanup_deleted_edge(
        &mut self,
        commands: &mut Commands,
        canvas: &mut Canvas,
        input_manager: &mut InputManager,
        vertex_manager: &mut VertexManager,
        face_manager: &mut FaceManager,
        entity_3d: &Entity,
    ) -> (Entity, Vec<Entity>) {
        let mut deleted_face_2d_entities = Vec::new();
        // cleanup faces
        {
            let face_3d_keys: Vec<FaceKey> = self
                .edges_3d
                .get(entity_3d)
                .unwrap()
                .faces_3d
                .iter()
                .copied()
                .collect();
            for face_3d_key in face_3d_keys {
                let face_2d_entity = face_manager.cleanup_deleted_face_key(
                    commands,
                    canvas,
                    input_manager,
                    vertex_manager,
                    self,
                    &face_3d_key,
                );
                deleted_face_2d_entities.push(face_2d_entity);
            }
        }

        // unregister edge
        let Some(edge_2d_entity) = self.unregister_3d_edge(vertex_manager,entity_3d) else {
            panic!(
                "Edge3d entity: `{:?}` has no corresponding Edge2d entity",
                entity_3d
            );
        };

        // despawn 2d edge
        info!("despawn 2d edge {:?}", edge_2d_entity);
        commands.entity(edge_2d_entity).despawn();

        if input_manager.hovered_entity == Some((edge_2d_entity, CanvasShape::Edge)) {
            input_manager.hovered_entity = None;
        }

        canvas.queue_resync_shapes();

        (edge_2d_entity, deleted_face_2d_entities)
    }

    pub(crate) fn has_edge_entity_3d(&self, entity_3d: &Entity) -> bool {
        self.edges_3d.contains_key(entity_3d)
    }

    pub(crate) fn edge_entity_2d_to_3d(&self, entity_2d: &Entity) -> Option<Entity> {
        self.edges_2d.get(entity_2d).copied()
    }

    pub(crate) fn edge_entity_3d_to_2d(&self, entity_2d: &Entity) -> Option<Entity> {
        self.edges_3d.get(entity_2d).map(|data| data.entity_2d)
    }

    pub(crate) fn edge_connected_faces(&self, edge_3d_entity: &Entity) -> Option<Vec<FaceKey>> {
        self.edges_3d
            .get(edge_3d_entity)
            .map(|data| data.faces_3d.iter().copied().collect())
    }

    pub fn edge_angle_visibility_toggle(&mut self, canvas: &mut Canvas) {
        if canvas.current_file_type != FileTypeValue::Skel {
            return;
        }

        self.edge_angle_visibility = !self.edge_angle_visibility;

        canvas.queue_resync_shapes();
    }

    // returns 2d edge entity
    fn unregister_3d_edge(
        &mut self,
        vertex_manager: &mut VertexManager,
        edge_3d_entity: &Entity,
    ) -> Option<Entity> {
        if let Some(entity_3d_data) = self.edges_3d.remove(edge_3d_entity) {
            let edge_2d_entity = entity_3d_data.entity_2d;

            info!(
                "deregister_3d_edge(3d: `{:?}`, 2d: `{:?}`)",
                edge_3d_entity, edge_2d_entity
            );

            self.edges_2d.remove(&edge_2d_entity);

            // remove edge from vertices
            for vertex_3d_entity in [
                entity_3d_data.vertex_a_3d_entity,
                entity_3d_data.vertex_b_3d_entity,
            ] {
                if let Some(vertex_3d_data) = vertex_manager.vertices_3d.get_mut(&vertex_3d_entity)
                {
                    vertex_3d_data.remove_edge(edge_3d_entity);
                }
            }

            return Some(edge_2d_entity);
        }
        return None;
    }

    pub(crate) fn edge_2d_entity_from_vertices(
        &self,
        vertex_manager: &VertexManager,
        vertex_2d_a: Entity,
        vertex_2d_b: Entity,
    ) -> Option<Entity> {
        let vertex_3d_a = vertex_manager.vertex_entity_2d_to_3d(&vertex_2d_a)?;
        let vertex_3d_b = vertex_manager.vertex_entity_2d_to_3d(&vertex_2d_b)?;
        let vertex_a_data = vertex_manager.vertices_3d.get(&vertex_3d_a)?;
        let vertex_b_data = vertex_manager.vertices_3d.get(&vertex_3d_b)?;
        let intersecting_edge_3d_entity = vertex_a_data
            .edges_3d
            .intersection(&vertex_b_data.edges_3d)
            .next()?;
        let edge_2d_entity = self.edge_entity_3d_to_2d(&intersecting_edge_3d_entity)?;
        Some(edge_2d_entity)
    }
}