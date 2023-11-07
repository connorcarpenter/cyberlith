use std::collections::{HashMap, HashSet};

use bevy_ecs::{
    entity::Entity,
    system::{Commands, Query, Res, ResMut, Resource, SystemState},
    world::{Mut, World},
};
use bevy_log::{info, warn};

use input::{Input, Key};
use naia_bevy_client::{Client, CommandsExt, ReplicationConfig};

use math::{Vec2, Vec3};

use render_api::{
    base::{Color, CpuMaterial, CpuMesh, CpuTexture2D},
    components::{
        AmbientLight, Camera, CameraBundle, OrthographicProjection, Projection, RenderLayer,
        RenderLayers, RenderObjectBundle, RenderTarget, Transform, Viewport,
    },
    resources::RenderFrame,
    shapes::{set_2d_line_transform, HollowTriangle, Triangle},
    Assets, Handle,
};

use vortex_proto::components::{IconEdge, IconFace, IconVertex, OwnedByFile};

use crate::app::{
    components::{
        DefaultDraw, Edge2dLocal, Face3dLocal, FaceIcon2d, IconEdgeLocal, IconLocalFace,
        OwnedByFileLocal, SelectCircle, SelectLine, SelectTriangle, Vertex2d,
    },
    resources::{
        action::icon::IconAction,
        canvas::Canvas,
        icon_data::IconEdgeData,
        icon_data::{IconFaceData, IconFaceKey, IconVertexData},
        input::IconInputManager,
        shape_data::CanvasShape,
        tab_manager::TabManager,
    },
    shapes::create_2d_edge_line,
};

#[derive(Resource)]
pub struct IconManager {
    wireframe: bool,
    pub(crate) camera_entity: Entity,
    render_layer: RenderLayer,

    resync_hover: bool,
    pub(crate) hovered_entity: Option<(Entity, CanvasShape)>,
    pub(crate) selected_shape: Option<(Entity, CanvasShape)>,
    select_circle_entity: Entity,
    select_triangle_entity: Entity,
    select_line_entity: Entity,

    // vertices
    vertices: HashMap<Entity, IconVertexData>,

    drags: Vec<(Entity, Vec2, Vec2)>,
    dragging_entity: Option<Entity>,
    dragging_start: Option<Vec2>,
    dragging_end: Option<Vec2>,

    // edges
    edges: HashMap<Entity, IconEdgeData>,

    // faces
    face_keys: HashMap<IconFaceKey, Option<IconFaceData>>,
    // net face entity -> face key
    net_faces: HashMap<Entity, IconFaceKey>,
    // local face entity -> face key
    local_faces: HashMap<Entity, IconFaceKey>,
    // queue of new faces to process
    new_face_keys: Vec<(IconFaceKey, Entity)>,
}

impl Default for IconManager {
    fn default() -> Self {
        Self {
            wireframe: true,
            camera_entity: Entity::PLACEHOLDER,
            render_layer: RenderLayer::default(),

            resync_hover: false,
            hovered_entity: None,
            selected_shape: None,
            select_circle_entity: Entity::PLACEHOLDER,
            select_triangle_entity: Entity::PLACEHOLDER,
            select_line_entity: Entity::PLACEHOLDER,

            // vertices
            vertices: HashMap::new(),
            drags: Vec::new(),
            dragging_entity: None,
            dragging_start: None,
            dragging_end: None,

            // edges
            edges: HashMap::new(),

            // faces
            new_face_keys: Vec::new(),
            face_keys: HashMap::new(),
            local_faces: HashMap::new(),
            net_faces: HashMap::new(),
        }
    }
}

impl IconManager {
    pub fn draw(&self, world: &mut World, current_file_entity: &Entity) {
        {
            let mut system_state: SystemState<(
                ResMut<RenderFrame>,
                Res<Canvas>,
                Res<TabManager>,
                Res<Input>,
                Query<(Entity, &IconVertex, &OwnedByFileLocal)>,
                Query<&IconLocalFace>,
                Query<(&Handle<CpuMesh>, &Handle<CpuMaterial>, Option<&RenderLayer>)>,
                Query<&mut Transform>,
            )> = SystemState::new(world);
            let (
                mut render_frame,
                canvas,
                tab_manager,
                input,
                vertex_q,
                face_q,
                object_q,
                mut transform_q
            ) = system_state.get_mut(world);

            // camera
            let camera_state = tab_manager.current_tab_camera_state().unwrap();
            let Ok(mut camera_transform) = transform_q.get_mut(self.camera_entity) else {
                return;
            };
            camera_transform.translation.x = 0.0 - camera_state.camera_3d_offset().x;
            camera_transform.translation.y = 0.0 - camera_state.camera_3d_offset().y;
            camera_transform.translation.z = 1.0;
            let camera_scale = 1.0 / camera_state.camera_3d_scale();
            camera_transform.scale = Vec3::new(camera_scale, camera_scale, 1.0);
            let camera_transform = *camera_transform;

            let mut edge_entities = HashSet::new();
            let mut face_keys = HashSet::new();

            // draw vertices, collect edges
            for (vertex_entity, vertex, owned_by_file) in vertex_q.iter() {
                if owned_by_file.file_entity != *current_file_entity {
                    continue;
                }

                // draw vertex
                let Some(data) = self.get_vertex_data(&vertex_entity) else {
                    continue;
                };

                let (mesh_handle, mat_handle, render_layer_opt) =
                    object_q.get(vertex_entity).unwrap();
                let mut transform = transform_q.get_mut(vertex_entity).unwrap();

                transform.translation.x = vertex.x() as f32;
                transform.translation.y = vertex.y() as f32;

                render_frame.draw_object(render_layer_opt, mesh_handle, mat_handle, &transform);

                for edge_entity in data.edges.iter() {
                    edge_entities.insert(*edge_entity);
                }
                for face_key in data.faces.iter() {
                    face_keys.insert(*face_key);
                }
            }

            // draw edges
            for edge_entity in edge_entities.iter() {

                let (start, end) = self.edge_get_endpoints(edge_entity);

                // sync
                let Ok(start_transform) = transform_q.get(start) else {
                    warn!(
                        "Edge start entity {:?} has no transform",
                        start,
                    );
                    continue;
                };

                let start_pos = start_transform.translation.truncate();

                let Ok(end_transform) = transform_q.get(end) else {
                    warn!(
                        "2d Edge end entity {:?} has no transform",
                        end,
                    );
                    continue;
                };

                let end_pos = end_transform.translation.truncate();
                let depth = (start_transform.translation.z + end_transform.translation.z) / 2.0;

                let Ok(mut edge_transform) = transform_q.get_mut(*edge_entity) else {
                    warn!("2d Edge entity {:?} has no transform", edge_entity);
                    return;
                };

                set_2d_line_transform(&mut edge_transform, start_pos, end_pos, depth);

                // draw

                let (mesh_handle, mat_handle, render_layer_opt) =
                    object_q.get(*edge_entity).unwrap();
                render_frame.draw_object(render_layer_opt, mesh_handle, mat_handle, &edge_transform);
            }

            // draw local faces
            for face_key in face_keys.iter() {
                let face_entity = self.local_face_entity_from_face_key(face_key).unwrap();

                let Ok(face_icon) = face_q.get(face_entity) else {
                    warn!("Face entity {:?} has no face icon", face_entity);
                    continue;
                };

                let Ok(vertex_a_transform) = transform_q.get(face_icon.vertex_a()) else {
                    warn!("Face entity {:?}'s vertex_a has no transform", face_entity);
                    continue;
                };
                let Ok(vertex_b_transform) = transform_q.get(face_icon.vertex_b()) else {
                    warn!("Face entity {:?}'s vertex_b has no transform", face_entity);
                    continue;
                };
                let Ok(vertex_c_transform) = transform_q.get(face_icon.vertex_c()) else {
                    warn!("Face entity {:?}'s vertex_c has no transform", face_entity);
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

                let (mesh_handle, mat_handle, render_layer_opt) =
                    object_q.get(face_entity).unwrap();
                let mut face_transform = transform_q.get_mut(face_entity).unwrap();
                face_transform.translation = center_translation;

                render_frame.draw_object(render_layer_opt, mesh_handle, mat_handle, &face_transform);
            }

            // draw select line & circle
            match self.selected_shape() {
                Some((vertex_entity, CanvasShape::Vertex)) => {

                    // draw select circle
                    let vertex_translation = transform_q.get(vertex_entity).unwrap().translation;

                    let (mesh_handle, mat_handle, render_layer_opt) =
                        object_q.get(self.select_circle_entity).unwrap();
                    let mut transform = transform_q.get_mut(self.select_circle_entity).unwrap();
                    transform.translation = vertex_translation;

                    render_frame.draw_object(render_layer_opt, mesh_handle, &mat_handle, &transform);

                    // draw select line
                    let screen_mouse_position = input.mouse_position();
                    let view_mouse_position = Self::screen_to_view(&canvas, &camera_transform, screen_mouse_position);
                    let (mesh_handle, mat_handle, render_layer_opt) =
                        object_q.get(self.select_line_entity).unwrap();

                    let mut transform = transform_q.get_mut(self.select_line_entity).unwrap();
                    set_2d_line_transform(&mut transform, vertex_translation.truncate(), view_mouse_position, vertex_translation.z + 1.0);

                    render_frame.draw_object(render_layer_opt, mesh_handle, &mat_handle, &transform);
                }
                Some((edge_entity, CanvasShape::Edge)) => {

                    let edge_transform = *transform_q.get(edge_entity).unwrap();

                    // draw select line
                    let (mesh_handle, mat_handle, render_layer_opt) =
                        object_q.get(self.select_line_entity).unwrap();

                    let mut transform = transform_q.get_mut(self.select_line_entity).unwrap();
                    transform.translation.x = edge_transform.translation.x;
                    transform.translation.y = edge_transform.translation.y;
                    transform.translation.z = edge_transform.translation.z + 1.0;
                    transform.rotation = edge_transform.rotation;
                    transform.scale.x = edge_transform.scale.x;
                    transform.scale.y = edge_transform.scale.y + 2.0;

                    render_frame.draw_object(render_layer_opt, mesh_handle, &mat_handle, &transform);
                }
                Some((face_entity, CanvasShape::Face)) => {

                    let face_translation = transform_q.get(face_entity).unwrap().translation;

                    // draw select triangle
                    let (mesh_handle, mat_handle, render_layer_opt) =
                        object_q.get(self.select_triangle_entity).unwrap();

                    let mut transform = transform_q.get_mut(self.select_triangle_entity).unwrap();
                    transform.translation = face_translation;

                    render_frame.draw_object(render_layer_opt, mesh_handle, &mat_handle, &transform);
                }
                _ => {}
            }
        }
    }

    pub fn setup_scene(
        &mut self,
        commands: &mut Commands,
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
        ambient_lights: &mut Assets<AmbientLight>,
        texture_size: &Vec2,
        canvas_texture_handle: Handle<CpuTexture2D>,
    ) {
        self.render_layer = RenderLayers::layer(4);

        // light
        {
            commands
                .spawn(ambient_lights.add(AmbientLight::new(1.0, Color::WHITE)))
                .insert(self.render_layer);
        }

        // camera
        {
            let mut camera_bundle = CameraBundle::new_2d(&Viewport::new_at_origin(
                texture_size.x as u32,
                texture_size.y as u32,
            ));
            camera_bundle.camera.target = RenderTarget::Image(canvas_texture_handle);
            camera_bundle.camera.is_active = false;
            camera_bundle.camera.order = 2;
            camera_bundle.transform = Transform::from_xyz(0.0, 0.0, 1.0)
                .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::NEG_Y);
            self.camera_entity = commands.spawn(camera_bundle).insert(self.render_layer).id();
        }

        // select circle
        {
            let mut select_circle_components = RenderObjectBundle::circle(
                meshes,
                materials,
                Vec2::ZERO,
                SelectCircle::RADIUS,
                Vertex2d::SUBDIVISIONS,
                Color::WHITE,
                Some(1),
            );
            select_circle_components.visibility.visible = false;
            let select_circle_entity = commands
                .spawn(select_circle_components)
                .insert(self.render_layer)
                .insert(SelectCircle)
                .insert(DefaultDraw)
                .id();
            self.select_circle_entity = select_circle_entity;
        }

        // select triangle
        {
            let mut select_triangle_components = RenderObjectBundle::equilateral_triangle(
                meshes,
                materials,
                Vec2::ZERO,
                SelectTriangle::SIZE,
                Color::WHITE,
                Some(1),
            );
            select_triangle_components.visibility.visible = false;
            let select_triangle_entity = commands
                .spawn(select_triangle_components)
                .insert(self.render_layer)
                .insert(SelectTriangle)
                .insert(DefaultDraw)
                .id();
            self.select_triangle_entity = select_triangle_entity;
        }

        // select line
        {
            let mut select_line_components = create_2d_edge_line(
                meshes,
                materials,
                Vec2::ZERO,
                Vec2::X,
                0.0,
                Color::WHITE,
                SelectLine::THICKNESS,
            );
            select_line_components.visibility.visible = false;
            let select_line_entity = commands
                .spawn(select_line_components)
                .insert(self.render_layer)
                .insert(SelectLine)
                .insert(DefaultDraw)
                .id();
            self.select_line_entity = select_line_entity;
        }
    }

    pub fn update_camera_viewport(
        &self,
        texture_size: Vec2,
        camera_query: &mut Query<(&mut Camera, &mut Transform, &mut Projection)>,
    ) {
        let Ok((mut camera, _, mut projection)) = camera_query.get_mut(self.camera_entity) else {
            return;
        };
        camera.viewport = Some(Viewport::new_at_origin(
            texture_size.x as u32,
            texture_size.y as u32,
        ));

        *projection =
            Projection::Orthographic(OrthographicProjection::new(texture_size.y, 0.0, 10.0));
    }

    pub fn enable_camera(
        &self,
        camera_q: &mut Query<(&mut Camera, &mut Projection, &mut Transform)>,
    ) {
        let Ok((mut camera, _, _)) = camera_q.get_mut(self.camera_entity) else {
            return;
        };
        camera.is_active = true;
    }

    pub fn disable_camera(
        &self,
        camera_q: &mut Query<(&mut Camera, &mut Projection, &mut Transform)>,
    ) {
        let Ok((mut camera, _, _)) = camera_q.get_mut(self.camera_entity) else {
            return;
        };
        camera.is_active = false;
    }

    pub fn handle_keypress_camera_controls(&mut self, key: Key) {
        match key {
            Key::S => {
                self.wireframe = false;
            }
            Key::W => {
                self.wireframe = true;
            }
            _ => panic!("Unexpected key: {:?}", key),
        }
    }

    pub fn queue_resync_hover_ui(&mut self) {
        self.resync_hover = true;
    }

    pub(crate) fn sync_mouse_hover_ui(
        &mut self,
        world: &mut World,
        current_file_entity: &Entity,
        screen_mouse_position: &Vec2,
    ) {
        if !self.resync_hover {
            return;
        }
        self.resync_hover = false;

        let mut system_state: SystemState<(Res<Canvas>, Query<&Transform>)> =
            SystemState::new(world);
        let (canvas, transform_q) = system_state.get_mut(world);

        let Ok(camera_transform) = transform_q.get(self.camera_entity) else {
            return;
        };
        let view_mouse_position = Self::screen_to_view(&canvas, camera_transform, screen_mouse_position);

        // sync to hover
        let new_hover_entity = IconInputManager::sync_mouse_hover_ui(world, current_file_entity, &view_mouse_position);
        if new_hover_entity == self.hovered_entity {
            return;
        }

        // reset scale of old hovered entity
        if let Some((hover_entity, hover_shape)) = self.hovered_entity {
            self.sync_hover_shape_scale(world, hover_entity, hover_shape, false);
        }

        // set hovered entity to new entity
        self.hovered_entity = new_hover_entity;

        // set scale of new hovered entity
        if let Some((hover_entity, hover_shape)) = self.hovered_entity {
            let visually_hovering = self.hovered_entity != self.selected_shape;
            self.sync_hover_shape_scale(world, hover_entity, hover_shape, visually_hovering);
        }
    }

    pub fn screen_to_view(canvas: &Canvas, camera_transform: &Transform, pos: &Vec2) -> Vec2 {

        // get canvas size
        let canvas_size = canvas.texture_size();

        let vx = (((pos.x / canvas_size.x) - 0.5)
            * camera_transform.scale.x
            * canvas_size.x)
            + camera_transform.translation.x;
        let vy = (((pos.y / canvas_size.y) - 0.5)
            * camera_transform.scale.y
            * canvas_size.y)
            + camera_transform.translation.y;
        Vec2::new(vx, vy)
    }

    fn sync_hover_shape_scale(&mut self, world: &mut World, hover_entity: Entity, hover_shape: CanvasShape, hovering: bool) {

        let mut system_state: SystemState<Query<&mut Transform>> = SystemState::new(world);
        let mut transform_q = system_state.get_mut(world);

        match hover_shape {
            CanvasShape::Vertex => {
                let scale = if hovering { Vertex2d::HOVER_RADIUS } else { Vertex2d::RADIUS };
                let mut hover_vert_transform = transform_q.get_mut(hover_entity).unwrap();
                hover_vert_transform.scale.x = scale;
                hover_vert_transform.scale.y = scale;
            }
            CanvasShape::Edge => {
                let scale = if hovering { Edge2dLocal::HOVER_THICKNESS } else { Edge2dLocal::NORMAL_THICKNESS };
                let mut hover_edge_transform = transform_q.get_mut(hover_entity).unwrap();
                hover_edge_transform.scale.y = scale;
            }
            CanvasShape::Face => {
                let scale = if hovering { FaceIcon2d::HOVER_SIZE } else { FaceIcon2d::SIZE };
                let mut hover_face_transform = transform_q.get_mut(hover_entity).unwrap();
                hover_face_transform.scale.x = scale;
                hover_face_transform.scale.y = scale;
            }
            _ => panic!(""),
        }
    }

    pub fn select_shape(&mut self, entity: &Entity, shape: CanvasShape) {
        if self.selected_shape.is_some() {
            panic!("must deselect before selecting");
        }
        self.selected_shape = Some((*entity, shape));
    }

    pub fn deselect_shape(&mut self) {
        self.selected_shape = None;
    }

    pub fn selected_shape(&self) -> Option<(Entity, CanvasShape)> {
        self.selected_shape
    }

    // Vertices

    pub(crate) fn handle_delete_vertex_action(
        &mut self,
        world: &mut World,
        vertex_entity: &Entity,
    ) {
        let mut system_state: SystemState<(Commands, Client)> = SystemState::new(world);
        let (mut commands, mut client) = system_state.get_mut(world);

        // delete vertex

        // check whether we can delete vertex
        let auth_status = commands.entity(*vertex_entity).authority(&client).unwrap();
        if !auth_status.is_granted() && !auth_status.is_available() {
            // do nothing, vertex is not available
            // TODO: queue for deletion? check before this?
            warn!("Vertex {:?} is not available for deletion!", vertex_entity);
            return;
        }

        let auth_status = commands.entity(*vertex_entity).authority(&client).unwrap();
        if !auth_status.is_granted() {
            // request authority if needed
            commands
                .entity(*vertex_entity)
                .request_authority(&mut client);
        }

        world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
            tab_manager.current_tab_execute_icon_action(
                world,
                self,
                IconAction::DeleteVertex(*vertex_entity, None),
            );
        });

        self.selected_shape = None;
    }

    pub(crate) fn reset_last_dragged_vertex(&mut self, world: &mut World) {
        // reset last dragged vertex
        if let Some(drags) = self.take_drags() {
            world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                for (vertex_entity, old_pos, new_pos) in drags {
                    tab_manager.current_tab_execute_icon_action(
                        world,
                        self,
                        IconAction::MoveVertex(vertex_entity, old_pos, new_pos, true),
                    );
                }
            });
        }
    }

    pub(crate) fn has_vertex_entity(&self, entity: &Entity) -> bool {
        self.vertices.contains_key(entity)
    }

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
        vertex_entity: Entity,
        ownership_opt: Option<Entity>,
        color: Color,
    ) {
        commands
            .entity(vertex_entity)
            .insert(RenderObjectBundle::circle(
                meshes,
                materials,
                Vec2::ZERO,
                Vertex2d::RADIUS,
                Vertex2d::SUBDIVISIONS,
                color,
                None,
            ))
            .insert(self.render_layer);

        if let Some(file_entity) = ownership_opt {
            commands
                .entity(vertex_entity)
                .insert(OwnedByFileLocal::new(file_entity));
        }

        self.register_vertex(vertex_entity);
    }

    pub fn register_vertex(&mut self, entity: Entity) {
        self.vertices.insert(entity, IconVertexData::new());
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

    pub fn cleanup_deleted_vertex(&mut self, vertex_entity: &Entity) {
        // unregister vertex
        self.unregister_vertex(vertex_entity);

        if self.hovered_entity == Some((*vertex_entity, CanvasShape::Vertex)) {
            self.hovered_entity = None;
        }
    }

    pub(crate) fn vertex_get_edges(&self, vertex_entity: &Entity) -> Option<&HashSet<Entity>> {
        self.vertices.get(vertex_entity).map(|data| &data.edges)
    }

    pub(crate) fn vertex_add_edge(&mut self, vertex_entity: &Entity, edge_entity: Entity) {
        let Some(vertex_data) = self.vertices.get_mut(&vertex_entity) else {
            panic!("Vertex entity: `{:?}` has not been registered", vertex_entity);
        };
        vertex_data.add_edge(edge_entity);
    }

    pub(crate) fn vertex_remove_edge(&mut self, vertex_entity: &Entity, edge_entity: &Entity) {
        // at this point, vertex_entity may have already been deregistered
        if let Some(vertex_data) = self.vertices.get_mut(vertex_entity) {
            vertex_data.remove_edge(edge_entity);
        }
    }

    pub(crate) fn vertex_get_faces(&self, vertex_entity: &Entity) -> Option<&HashSet<IconFaceKey>> {
        self.vertices.get(vertex_entity).map(|data| &data.faces)
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

    fn unregister_vertex(&mut self, entity: &Entity) {
        self.vertices.remove(entity);
    }

    pub(crate) fn get_vertex_data(&self, entity: &Entity) -> Option<&IconVertexData> {
        self.vertices.get(entity)
    }

    pub(crate) fn get_connected_vertices(&self, vertex_entity: Entity) -> HashSet<Entity> {
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
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
        vertex_entity_a: Entity,
        vertex_entity_b: Entity,
        file_entity: Entity,
        entities_to_release: &mut Vec<Entity>,
    ) -> Entity {
        // create new edge
        let mut new_edge_component = IconEdge::new();
        new_edge_component.start.set(client, &vertex_entity_a);
        new_edge_component.end.set(client, &vertex_entity_b);
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
        edge_entity: Entity,
        vertex_entity_a: Entity,
        vertex_entity_b: Entity,
        ownership_opt: Option<Entity>,
        color: Color,
    ) {
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
            .insert(self.render_layer)
            .insert(IconEdgeLocal::new(vertex_entity_a, vertex_entity_b));
        if let Some(file_entity) = ownership_opt {
            commands
                .entity(edge_entity)
                .insert(OwnedByFileLocal::new(file_entity));
        }

        // register edge
        self.register_edge(edge_entity, vertex_entity_a, vertex_entity_b, ownership_opt);
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
            IconEdgeData::new(vertex_entity_a, vertex_entity_b),
        );

        let file_entity = ownership_opt.unwrap();
        self.check_for_new_faces(file_entity, vertex_entity_a, vertex_entity_b);
    }

    // returns (deleted edge entity, Vec<deleted face entity>
    pub fn cleanup_deleted_edge(
        &mut self,
        commands: &mut Commands,
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
                    let local_face_entity = self.cleanup_deleted_face_key(commands, &face_key);
                    deleted_local_face_entities.push(local_face_entity);
                }
            }
        }

        // unregister edge
        self.unregister_edge(edge_entity);

        if self.hovered_entity == Some((*edge_entity, CanvasShape::Edge)) {
            self.hovered_entity = None;
        }

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

    fn unregister_edge(&mut self, edge_entity: &Entity) {
        if let Some(edge_data) = self.edges.remove(edge_entity) {
            // remove edge from vertices
            for vertex_entity in [edge_data.vertex_entity_a, edge_data.vertex_entity_b] {
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

    pub fn process_new_local_faces(
        &mut self,
        commands: &mut Commands,
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
    ) {
        if self.new_face_keys.is_empty() {
            return;
        }

        let keys = std::mem::take(&mut self.new_face_keys);
        for (face_key, file_entity) in keys {
            self.process_new_local_face(commands, meshes, materials, file_entity, &face_key);
        }
    }

    // return local face entity
    pub fn process_new_local_face(
        &mut self,
        commands: &mut Commands,
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
        file_entity: Entity,
        face_key: &IconFaceKey,
    ) -> Entity {
        if self.has_local_face(face_key) {
            panic!("face key already registered! `{:?}`", face_key);
        }
        info!("processing new face: `{:?}`", face_key);
        let vertex_a = face_key.vertex_a;
        let vertex_b = face_key.vertex_b;
        let vertex_c = face_key.vertex_c;

        // local face needs to have it's own button mesh, matching the vertices

        let new_entity = commands
            .spawn_empty()
            .insert(IconLocalFace::new(vertex_a, vertex_b, vertex_c))
            .insert(RenderObjectBundle::equilateral_triangle(
                meshes,
                materials,
                Vec2::ZERO,
                FaceIcon2d::SIZE,
                FaceIcon2d::COLOR,
                Some(1),
            ))
            .insert(self.render_layer)
            .id();

        info!("spawned face entity: {:?}", new_entity);

        info!(
            "adding OwnedByFile({:?}) to entity {:?}",
            file_entity, new_entity
        );
        commands
            .entity(new_entity)
            .insert(OwnedByFileLocal::new(file_entity));

        // add face to vertex data
        for vertex_entity in [&vertex_a, &vertex_b, &vertex_c] {
            self.vertex_add_face(vertex_entity, *face_key)
        }

        // add face to edge data
        let mut edge_entities = Vec::new();
        for (vert_a, vert_b) in [
            (&vertex_a, &vertex_b),
            (&vertex_b, &vertex_c),
            (&vertex_c, &vertex_a),
        ] {
            // find edge in common
            let vertex_a_edges = self.vertex_get_edges(vert_a).unwrap();
            let vertex_b_edges = self.vertex_get_edges(vert_b).unwrap();
            let intersection = vertex_a_edges.intersection(vertex_b_edges);
            let mut found_edge = None;
            for edge_entity in intersection {
                if found_edge.is_some() {
                    panic!("should only be one edge between any two vertices!");
                }
                found_edge = Some(*edge_entity);
            }

            if let Some(edge_entity) = found_edge {
                self.edge_add_face(&edge_entity, *face_key);

                edge_entities.push(edge_entity);
            }
        }

        // register face data
        self.face_keys.insert(
            *face_key,
            Some(IconFaceData::new(
                file_entity,
                new_entity,
                edge_entities[0],
                edge_entities[1],
                edge_entities[2],
            )),
        );
        self.local_faces.insert(new_entity, *face_key);

        new_entity
    }


    pub fn create_networked_face_from_world(
        &mut self,
        world: &mut World,
        local_face_entity: Entity,
    ) {
        let Some(face_key) = self.face_key_from_local_entity(&local_face_entity) else {
            panic!(
                "LocalFace entity: `{:?}` has no corresponding FaceKey",
                local_face_entity
            );
        };
        let Some(Some(face_data)) = self.face_keys.get(&face_key) else {
            panic!(
                "NetFace entity: `{:?}` has not been registered",
                face_key
            );
        };
        if face_data.net_entity.is_some() {
            panic!("already created net face entity! cannot do this twice!");
        }

        let mut system_state: SystemState<(
            Commands,
            Client,
            ResMut<Assets<CpuMesh>>,
            ResMut<Assets<CpuMaterial>>,
            Query<&Transform>,
        )> = SystemState::new(world);
        let (mut commands, mut client, mut meshes, mut materials, transform_q) =
            system_state.get_mut(world);

        self.create_networked_face(
            &mut commands,
            &mut client,
            &mut meshes,
            &mut materials,
            &transform_q,
            &face_key,
            [face_data.edge_a, face_data.edge_b, face_data.edge_c],
            face_data.file_entity,
        );

        system_state.apply(world);
    }

    pub fn create_networked_face(
        &mut self,
        commands: &mut Commands,
        client: &mut Client,
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
        transform_q: &Query<&Transform>,
        face_key: &IconFaceKey,
        edge_entities: [Entity; 3],
        file_entity: Entity,
    ) {
        // get vertex entities & positions
        let mut positions = [Vec3::ZERO, Vec3::ZERO, Vec3::ZERO];
        let mut vertex_entities = [
            Entity::PLACEHOLDER,
            Entity::PLACEHOLDER,
            Entity::PLACEHOLDER,
        ];

        for (index, vertex_entity) in [face_key.vertex_a, face_key.vertex_b, face_key.vertex_c]
            .iter()
            .enumerate()
        {
            let vertex_transform = transform_q.get(*vertex_entity).unwrap();
            positions[index] = vertex_transform.translation;
            vertex_entities[index] = *vertex_entity;
        }

        // possibly reorder vertices to be counter-clockwise with respect to camera
        let camera_transform = transform_q.get(self.camera_entity).unwrap();
        if math::reorder_triangle_winding(&mut positions, camera_transform.translation, true) {
            vertex_entities.swap(1, 2);
        }

        // set up networked face component
        let mut face_component = IconFace::new();
        face_component.vertex_a.set(client, &vertex_entities[0]);
        face_component.vertex_b.set(client, &vertex_entities[1]);
        face_component.vertex_c.set(client, &vertex_entities[2]);
        face_component.edge_a.set(client, &edge_entities[0]);
        face_component.edge_b.set(client, &edge_entities[1]);
        face_component.edge_c.set(client, &edge_entities[2]);

        // get owned_by_file component
        let mut owned_by_file_component = OwnedByFile::new();
        owned_by_file_component
            .file_entity
            .set(client, &file_entity);

        // set up net entity
        let face_net_entity = commands
            .spawn_empty()
            .enable_replication(client)
            .configure_replication(ReplicationConfig::Delegated)
            .insert(owned_by_file_component)
            .insert(OwnedByFileLocal::new(file_entity))
            .insert(face_component)
            .id();

        let positions = positions.map(|vec3| vec3.truncate());
        self.net_face_postprocess(
            commands,
            meshes,
            materials,
            face_key,
            face_net_entity,
            positions,
        );
    }

    pub fn net_face_postprocess(
        &mut self,
        commands: &mut Commands,
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
        face_key: &IconFaceKey,
        net_face_entity: Entity,
        positions: [Vec2; 3],
    ) {
        let positions = positions.map(|vec2| vec2.extend(0.0));
        commands
            .entity(net_face_entity)
            .insert(RenderObjectBundle::world_triangle(
                meshes,
                materials,
                positions,
                Face3dLocal::COLOR,
            ))
            .insert(self.render_layer);

        self.register_net_face(net_face_entity, face_key);

        // change local face to use non-hollow triangle
        let local_face_entity = self.local_face_entity_from_face_key(&face_key).unwrap();
        commands
            .entity(local_face_entity)
            .insert(meshes.add(Triangle::new_2d_equilateral()));
    }

    pub fn register_net_face(&mut self, net_face_entity: Entity, face_key: &IconFaceKey) {
        self.net_faces.insert(net_face_entity, *face_key);

        let Some(Some(face_data)) = self.face_keys.get_mut(face_key) else {
            panic!("FaceKey: `{:?}` has not been registered", face_key);
        };
        face_data.net_entity = Some(net_face_entity);
    }

    pub fn remove_new_face_key(&mut self, face_key: &IconFaceKey) {
        self.new_face_keys.retain(|(key, _)| key != face_key);
    }

    pub(crate) fn cleanup_deleted_net_face(
        &mut self,
        commands: &mut Commands,
        meshes: &mut Assets<CpuMesh>,
        net_face_entity: &Entity,
    ) {
        // unregister face
        if let Some(local_face_entity) = self.unregister_net_face(net_face_entity) {
            commands
                .entity(local_face_entity)
                .insert(meshes.add(HollowTriangle::new_2d_equilateral()));
        }
    }

    // returns local face entity
    pub(crate) fn cleanup_deleted_face_key(
        &mut self,
        commands: &mut Commands,
        face_key: &IconFaceKey,
    ) -> Entity {
        // unregister face
        let Some(local_face_entity) = self.unregister_face_key(face_key) else {
            panic!(
                "FaceKey: `{:?}` has no corresponding local Face entity",
                face_key
            );
        };

        // despawn local face
        info!("despawn local face {:?}", local_face_entity);
        commands.entity(local_face_entity).despawn();

        if self.hovered_entity == Some((local_face_entity, CanvasShape::Face)) {
            self.hovered_entity = None;
        }

        local_face_entity
    }

    pub(crate) fn has_local_face(&self, face_key: &IconFaceKey) -> bool {
        if let Some(Some(_)) = self.face_keys.get(face_key) {
            return true;
        }
        return false;
    }

    pub(crate) fn face_entity_local_to_net(&self, local_entity: &Entity) -> Option<Entity> {
        let Some(face_key) = self.local_faces.get(local_entity) else {
            return None;
        };
        let Some(Some(face_data)) = self.face_keys.get(face_key) else {
            return None;
        };
        face_data.net_entity
    }

    pub(crate) fn face_entity_net_to_local(&self, net_entity: &Entity) -> Option<Entity> {
        let Some(face_key) = self.net_faces.get(net_entity) else {
            return None;
        };
        self.local_face_entity_from_face_key(face_key)
    }

    pub(crate) fn local_face_entity_from_face_key(&self, face_key: &IconFaceKey) -> Option<Entity> {
        let Some(Some(face_data)) = self.face_keys.get(face_key) else {
            return None;
        };
        Some(face_data.local_entity)
    }

    fn face_key_from_local_entity(&self, local_entity: &Entity) -> Option<IconFaceKey> {
        self.local_faces.get(local_entity).copied()
    }

    pub(crate) fn net_face_entity_from_face_key(&self, face_key: &IconFaceKey) -> Option<Entity> {
        let Some(Some(face_data)) = self.face_keys.get(face_key) else {
            return None;
        };
        face_data.net_entity
    }

    // returns local face entity
    fn unregister_face_key(&mut self, face_key: &IconFaceKey) -> Option<Entity> {
        info!("unregistering face key: `{:?}`", face_key);
        if let Some(Some(face_data)) = self.face_keys.remove(&face_key) {
            let local_entity = face_data.local_entity;
            self.local_faces.remove(&local_entity);

            // remove face from vertices
            for vertex_entity in [face_key.vertex_a, face_key.vertex_b, face_key.vertex_c] {
                self.vertex_remove_face(&vertex_entity, face_key);
            }

            // remove face from edges
            for edge_entity in [face_data.edge_a, face_data.edge_b, face_data.edge_c] {
                self.edge_remove_face(&edge_entity, face_key);
            }

            return Some(local_entity);
        } else {
            return None;
        }
    }

    // returns local face entity
    fn unregister_net_face(&mut self, net_entity: &Entity) -> Option<Entity> {
        info!("unregistering net face entity: `{:?}`", net_entity);
        let Some(face_key) = self.net_faces.remove(net_entity) else {
            panic!("no net face found for entity {:?}", net_entity);
        };

        if let Some(Some(face_data)) = self.face_keys.get_mut(&face_key) {
            face_data.net_entity = None;
            info!("remove net entity: `{:?}` from face data", net_entity);

            let local_face_entity = face_data.local_entity;
            return Some(local_face_entity);
        }

        return None;
    }

    pub(crate) fn check_for_new_faces(
        &mut self,
        file_entity: Entity,
        vertex_entity_a: Entity,
        vertex_entity_b: Entity,
    ) {
        let vertex_a_connected_vertices = self.get_connected_vertices(vertex_entity_a);
        let vertex_b_connected_vertices = self.get_connected_vertices(vertex_entity_b);

        let common_vertices =
            vertex_a_connected_vertices.intersection(&vertex_b_connected_vertices);
        for common_vertex in common_vertices {
            let face_key = IconFaceKey::new(vertex_entity_a, vertex_entity_b, *common_vertex);
            if !self.face_keys.contains_key(&face_key) {
                self.face_keys.insert(face_key, None);
                self.new_face_keys.push((face_key, file_entity));
            }
        }
    }
}
