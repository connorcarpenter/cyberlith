use std::{collections::HashMap, f32::consts::FRAC_PI_2};

use bevy_ecs::{
    entity::Entity,
    event::EventWriter,
    system::{Commands, Query, Resource},
};
use bevy_log::{info, warn};

use naia_bevy_client::{Client, CommandsExt, ReplicationConfig};

use math::{Vec2, Vec3};
use render_api::{
    base::{Color, CpuMaterial, CpuMesh},
    components::{RenderLayer, RenderObjectBundle, Transform, Visibility},
    resources::RenderFrame,
    shapes::{
        angle_between, get_2d_line_transform_endpoint, set_2d_line_transform,
        set_2d_line_transform_from_angle,
    },
    Assets, Handle,
};

use vortex_proto::components::{
    Edge3d, EdgeAngle, FileExtension, FileType, OwnedByFile, ShapeName,
};

use crate::app::{
    components::{DefaultDraw, Edge2dLocal, Edge3dLocal, LocalShape, OwnedByFileLocal, Vertex2d},
    events::ShapeColorResyncEvent,
    resources::{
        camera_manager::CameraManager,
        canvas::Canvas,
        face_manager::FaceManager,
        file_manager::FileManager,
        input::InputManager,
        shape_data::{CanvasShape, Edge3dData, FaceKey},
        tab_manager::TabManager,
        vertex_manager::VertexManager,
    },
    set_3d_line_transform,
    shapes::{
        create_2d_edge_arrow, create_2d_edge_line, create_3d_edge_diamond, create_3d_edge_line,
    },
};

#[derive(Resource)]
pub struct EdgeManager {
    resync: bool,
    // 3d edge entity -> 3d edge data
    edges_3d: HashMap<Entity, Edge3dData>,
    // 2d edge entity -> 3d edge entity
    edges_2d: HashMap<Entity, Entity>,

    last_edge_dragged: Option<(Entity, f32, f32)>,

    edge_angle_visibility: bool,
}

impl Default for EdgeManager {
    fn default() -> Self {
        Self {
            resync: false,
            edges_3d: HashMap::new(),
            edges_2d: HashMap::new(),
            last_edge_dragged: None,
            edge_angle_visibility: true,
        }
    }
}

impl EdgeManager {
    pub fn queue_resync(&mut self) {
        self.resync = true;
    }

    pub fn get_should_sync(&self) -> bool {
        self.resync
    }

    pub fn finish_sync(&mut self) {
        self.resync = false;
    }

    pub fn sync_2d_edges(
        &self,
        edge_2d_q: &Query<(Entity, &Edge2dLocal)>,
        transform_q: &mut Query<&mut Transform>,
        visibility_q: &mut Query<&mut Visibility>,
        local_shape_q: &Query<&LocalShape>,
        camera_3d_scale: f32,
    ) {
        let edge_2d_scale = Edge2dLocal::NORMAL_THICKNESS * camera_3d_scale;

        for (edge_2d_entity, edge_endpoints) in edge_2d_q.iter() {
            // visibility
            let Ok(visibility) = visibility_q.get(edge_2d_entity) else {
                panic!("entity has no Visibility");
            };
            if !visibility.visible {
                continue;
            }

            Self::sync_2d_edge(transform_q, &edge_2d_entity, edge_endpoints);

            let mut edge_2d_scale_y = edge_2d_scale;
            if local_shape_q.get(edge_2d_entity).is_ok() {
                edge_2d_scale_y = Edge2dLocal::NORMAL_THICKNESS;
            }

            let Ok(mut edge_2d_transform) = transform_q.get_mut(edge_2d_entity) else {
                warn!("2d Edge entity {:?} has no transform", edge_2d_entity);
                continue;
            };
            edge_2d_transform.scale.y = edge_2d_scale_y;
        }
    }

    pub fn sync_2d_edge(transform_q: &mut Query<&mut Transform>, edge_2d_entity: &Entity, edge_endpoints: &Edge2dLocal) {
        let Ok(start_transform) = transform_q.get(edge_endpoints.start) else {
            warn!(
                    "2d Edge start entity {:?} has no transform",
                    edge_endpoints.start,
                );
            return;
        };

        let start_pos = start_transform.translation.truncate();

        let Ok(end_transform) = transform_q.get(edge_endpoints.end) else {
            warn!(
                    "2d Edge end entity {:?} has no transform",
                    edge_endpoints.end,
                );
            return;
        };

        let end_pos = end_transform.translation.truncate();
        let depth = (start_transform.translation.z + end_transform.translation.z) / 2.0;

        let Ok(mut edge_2d_transform) = transform_q.get_mut(*edge_2d_entity) else {
            warn!("2d Edge entity {:?} has no transform", edge_2d_entity);
            return;
        };

        set_2d_line_transform(&mut edge_2d_transform, start_pos, end_pos, depth);
    }

    pub fn sync_edge_angles(
        &self,
        file_ext: FileExtension,
        edge_angle_q: &Query<(Entity, &EdgeAngle)>,
        transform_q: &mut Query<&mut Transform>,
        visibility_q: &mut Query<&mut Visibility>,
        camera_3d_scale: f32,
    ) {
        let edge_angle_base_circle_scale =
            Edge2dLocal::EDGE_ANGLE_BASE_CIRCLE_RADIUS * camera_3d_scale;
        let edge_angle_end_circle_scale =
            Edge2dLocal::EDGE_ANGLE_END_CIRCLE_RADIUS * camera_3d_scale;
        let edge_angle_length = Edge2dLocal::EDGE_ANGLE_LENGTH * camera_3d_scale;
        let edge_angle_thickness = Edge2dLocal::EDGE_ANGLE_THICKNESS * camera_3d_scale;
        let edge_angles_visible = self.edge_angles_are_visible(file_ext);

        for (edge_3d_entity, edge_angle) in edge_angle_q.iter() {
            let Some(edge_3d_data) = self.edges_3d.get(&edge_3d_entity) else {
                continue;
            };
            let edge_2d_entity = edge_3d_data.entity_2d;

            // visibility
            let Ok(visibility) = visibility_q.get(edge_2d_entity) else {
                continue;
            };
            if !visibility.visible {
                continue;
            }

            let edge_angle = edge_angle.get_radians();
            let (base_circle_entity, angle_edge_entity, end_circle_entity) =
                edge_3d_data.angle_entities_opt.unwrap();

            for entity in [base_circle_entity, angle_edge_entity, end_circle_entity] {
                let Ok(mut visibility) = visibility_q.get_mut(entity) else {
                    warn!("Edge angle entity {:?} has no transform", entity);
                    continue;
                };
                visibility.visible = edge_angles_visible;
            }

            if edge_angles_visible {
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

    pub fn sync_3d_edges(
        file_ext: FileExtension,
        edge_3d_q: &Query<(Entity, &Edge3dLocal, Option<&EdgeAngle>)>,
        transform_q: &mut Query<&mut Transform>,
        visibility_q: &mut Query<&mut Visibility>,
        local_shape_q: &Query<&LocalShape>,
    ) {
        for (edge_entity, edge_endpoints, edge_angle_opt) in edge_3d_q.iter() {
            // check visibility
            let Ok(mut visibility) = visibility_q.get_mut(edge_entity) else {
                continue;
            };
            if !visibility.visible {
                continue;
            }
            match file_ext {
                FileExtension::Skin => {
                    if local_shape_q.get(edge_entity).is_err() {
                        visibility.visible = false;
                    }
                }
                _ => {}
            }

            Self::sync_3d_edge(transform_q, &edge_entity, edge_endpoints, edge_angle_opt);
        }
    }

    pub fn sync_3d_edge(transform_q: &mut Query<&mut Transform>, edge_entity: &Entity, edge_endpoints: &Edge3dLocal, edge_angle_opt: Option<&EdgeAngle>) {
        let edge_angle_opt = edge_angle_opt.map(|e| e.get_radians());
        let edge_angle = edge_angle_opt.unwrap_or_default();

        let edge_start_entity = edge_endpoints.start;
        let edge_end_entity = edge_endpoints.end;

        let Ok(start_transform) = transform_q.get(edge_start_entity) else {
            warn!(
                "3d Edge start entity {:?} has no transform",
                edge_start_entity,
            );
            return;
        };
        let start_pos = start_transform.translation;
        let Ok(end_transform) = transform_q.get(edge_end_entity) else {
            warn!("3d Edge end entity {:?} has no transform", edge_end_entity);
            return;
        };
        let end_pos = end_transform.translation;
        let mut edge_transform = transform_q.get_mut(*edge_entity).unwrap();
        set_3d_line_transform(&mut edge_transform, start_pos, end_pos, edge_angle);
    }

    pub fn sync_local_3d_edges(
        edge_3d_q: &Query<(Entity, &Edge3dLocal, Option<&EdgeAngle>)>,
        transform_q: &mut Query<&mut Transform>,
        local_shape_q: &Query<&LocalShape>,
        camera_3d_scale: f32,
    ) {
        let local_shape_edge_3d_scale = LocalShape::EDGE_THICKNESS / camera_3d_scale;

        for (edge_3d_entity, edge_endpoints, _) in edge_3d_q.iter() {
            if local_shape_q.get(edge_3d_entity).is_err() {
                continue;
            }

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
            let mut edge_transform = transform_q.get_mut(edge_3d_entity).unwrap();
            set_3d_line_transform(&mut edge_transform, start_pos, end_pos, 0.0);

            edge_transform.scale.x = local_shape_edge_3d_scale;
            edge_transform.scale.y = local_shape_edge_3d_scale;
        }
    }

    pub fn reset_last_edge_dragged(&mut self) {
        self.last_edge_dragged = None;
    }

    pub fn update_last_edge_dragged(&mut self, edge_2d_entity: Entity, old_rot: f32, new_rot: f32) {
        if let Some((_, old_rot, _)) = self.last_edge_dragged {
            self.last_edge_dragged = Some((edge_2d_entity, old_rot, new_rot));
        } else {
            self.last_edge_dragged = Some((edge_2d_entity, old_rot, new_rot));
        }
    }

    pub fn take_last_edge_dragged(&mut self) -> Option<(Entity, f32, f32)> {
        self.last_edge_dragged.take()
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
        shape_color_resync_events: &mut EventWriter<ShapeColorResyncEvent>,
        parent_vertex_2d_entity: Entity,
        parent_vertex_3d_entity: Entity,
        child_vertex_2d_entity: Entity,
        child_vertex_3d_entity: Entity,
        file_entity: Entity,
        file_type: FileExtension,
        edge_angle: Option<f32>,
        entities_to_release: &mut Vec<Entity>,
    ) -> (Entity, Entity) {
        // create new 3d edge
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

        if file_type == FileExtension::Skel {
            let edge_angle_f32 = edge_angle.unwrap();
            commands
                .entity(new_edge_3d_entity)
                .insert(EdgeAngle::new(edge_angle_f32));
        }

        let default_draw = file_type == FileExtension::Mesh;

        // create new 2d edge, add local components to 3d edge
        let new_edge_2d_entity = self.edge_3d_postprocess(
            commands,
            meshes,
            materials,
            camera_manager,
            vertex_manager,
            face_manager,
            Some(shape_color_resync_events),
            new_edge_3d_entity,
            parent_vertex_2d_entity,
            parent_vertex_3d_entity,
            child_vertex_2d_entity,
            child_vertex_3d_entity,
            Some(file_entity),
            Vertex2d::ENABLED_COLOR,
            file_type == FileExtension::Skel,
            edge_angle,
            default_draw,
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
        shape_color_resync_events_opt: Option<&mut EventWriter<ShapeColorResyncEvent>>,
        edge_3d_entity: Entity,
        vertex_a_2d_entity: Entity,
        vertex_a_3d_entity: Entity,
        vertex_b_2d_entity: Entity,
        vertex_b_3d_entity: Entity,
        ownership_opt: Option<Entity>,
        color: Color,
        arrows_not_lines: bool,
        edge_angle_opt: Option<f32>,
        default_draw: bool,
    ) -> Entity {
        if let Some(shape_color_resync_events) = shape_color_resync_events_opt {
            // send shape color resync event
            shape_color_resync_events.send(ShapeColorResyncEvent);
        }

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

        if default_draw {
            commands.entity(edge_2d_entity).insert(DefaultDraw);
            commands.entity(edge_3d_entity).insert(DefaultDraw);
            //edge angle entities aren't here because theyre only in editor modes that DON'T use default draw ..
        }

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
            vertex_manager.vertex_add_edge(&vertex_3d_entity, edge_3d_entity);
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

    pub(crate) fn edge_entity_3d_to_2d(&self, entity_3d: &Entity) -> Option<Entity> {
        self.edges_3d.get(entity_3d).map(|data| data.entity_2d)
    }

    pub(crate) fn edge_connected_faces(&self, edge_3d_entity: &Entity) -> Option<Vec<FaceKey>> {
        self.edges_3d
            .get(edge_3d_entity)
            .map(|data| data.faces_3d.iter().copied().collect())
    }

    pub(crate) fn edge_add_face(&mut self, edge_3d_entity: &Entity, face_key: FaceKey) {
        self.edges_3d
            .get_mut(edge_3d_entity)
            .unwrap()
            .faces_3d
            .insert(face_key);
    }

    pub(crate) fn edge_remove_face(&mut self, edge_3d_entity: &Entity, face_key: &FaceKey) {
        self.edges_3d
            .get_mut(edge_3d_entity)
            .unwrap()
            .faces_3d
            .remove(face_key);
    }

    pub(crate) fn edge_get_endpoints(&self, edge_3d_entity: &Entity) -> (Entity, Entity) {
        let edge_data = self.edges_3d.get(edge_3d_entity).unwrap();
        (edge_data.vertex_a_3d_entity, edge_data.vertex_b_3d_entity)
    }

    pub(crate) fn edge_get_base_circle_entity(&self, edge_3d_entity: &Entity) -> Entity {
        self.edges_3d
            .get(edge_3d_entity)
            .unwrap()
            .angle_entities_opt
            .unwrap()
            .0
    }

    pub(crate) fn edge_angle_entities(
        &self,
        edge_3d_entity: &Entity,
    ) -> Option<(Entity, Entity, Entity)> {
        let data = self.edges_3d.get(edge_3d_entity)?;
        data.angle_entities_opt
    }

    pub fn edge_angle_visibility_toggle(
        &mut self,
        file_manager: &FileManager,
        tab_manager: &TabManager,
        canvas: &mut Canvas,
    ) {
        let current_file_type = FileManager::get_current_file_type(file_manager, tab_manager);
        if !Self::edge_angles_are_visible_for_filetype(current_file_type) {
            return;
        }

        self.edge_angle_visibility = !self.edge_angle_visibility;

        canvas.queue_resync_shapes();
    }

    fn edge_angles_are_visible_for_filetype(file_type: FileExtension) -> bool {
        match file_type {
            FileExtension::Skel | FileExtension::Anim => true,
            _ => false,
        }
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
                vertex_manager.vertex_remove_edge(&vertex_3d_entity, edge_3d_entity);
            }

            return Some(edge_2d_entity);
        }
        return None;
    }

    pub(crate) fn edge_3d_entity_from_vertices(
        &self,
        vertex_manager: &VertexManager,
        vertex_3d_a: Entity,
        vertex_3d_b: Entity,
    ) -> Option<Entity> {
        let vertex_a_edges = vertex_manager.vertex_get_edges(&vertex_3d_a)?;
        let vertex_b_edges = vertex_manager.vertex_get_edges(&vertex_3d_b)?;
        let intersecting_edge_3d_entity = vertex_a_edges.intersection(&vertex_b_edges).next()?;
        Some(*intersecting_edge_3d_entity)
    }

    pub(crate) fn edge_2d_entity_from_vertices(
        &self,
        vertex_manager: &VertexManager,
        vertex_2d_a: Entity,
        vertex_2d_b: Entity,
    ) -> Option<Entity> {
        let intersecting_edge_3d_entity = self.edge_3d_entity_from_vertices(
            vertex_manager,
            vertex_manager.vertex_entity_2d_to_3d(&vertex_2d_a)?,
            vertex_manager.vertex_entity_2d_to_3d(&vertex_2d_b)?,
        )?;
        let edge_2d_entity = self.edge_entity_3d_to_2d(&intersecting_edge_3d_entity)?;
        Some(edge_2d_entity)
    }

    pub(crate) fn draw_edge_angles(
        &self,
        edge_3d_entity: &Entity,
        render_frame: &mut RenderFrame,
        objects_q: &Query<(&Handle<CpuMesh>, &Transform, Option<&RenderLayer>)>,
        materials_q: &Query<&Handle<CpuMaterial>>,
    ) {
        let edge_3d_data = self.edges_3d.get(edge_3d_entity).unwrap();
        let (base_circle_entity, angle_edge_entity, end_circle_entity) =
            edge_3d_data.angle_entities_opt.unwrap();

        for entity in [base_circle_entity, angle_edge_entity, end_circle_entity] {
            let (mesh_handle, transform, render_layer_opt) = objects_q.get(entity).unwrap();
            let mat_handle = materials_q.get(entity).unwrap();
            render_frame.draw_object(render_layer_opt, mesh_handle, &mat_handle, transform);
        }
    }

    pub(crate) fn edge_angles_are_visible(&self, file_ext: FileExtension) -> bool {
        self.edge_angle_visibility && Self::edge_angles_are_visible_for_filetype(file_ext)
    }
}

pub fn edge_is_enabled(shape_name_opt: Option<&ShapeName>) -> bool {
    if let Some(shape_name) = shape_name_opt {
        if shape_name.value.len() > 0 {
            true
        } else {
            false
        }
    } else {
        false
    }
}
