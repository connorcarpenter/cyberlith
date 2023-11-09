use std::collections::{HashMap, HashSet};

use bevy_ecs::{
    query::With,
    entity::Entity,
    system::{Commands, Query, Res, ResMut, Resource, SystemState},
    world::{Mut, World},
};
use bevy_log::{info, warn};

use input::{Input, Key};
use naia_bevy_client::{Client, CommandsExt, Instant, Replicate, ReplicationConfig};

use math::{Vec2, Vec3};

use render_api::{
    base::{Color, CpuMaterial, CpuMesh, CpuTexture2D},
    components::{
        AmbientLight, Camera, CameraBundle, OrthographicProjection, Projection, RenderLayer,
        RenderLayers, RenderObjectBundle, RenderTarget, Transform, Viewport,
    },
    resources::RenderFrame,
    shapes::{set_2d_line_transform, Circle, HollowTriangle, Triangle},
    Assets, Handle,
};

use vortex_proto::components::{IconEdge, IconFace, IconFrame, IconVertex, OwnedByFile};

use crate::app::{
    components::{
        DefaultDraw, Edge2dLocal, Face3dLocal, FaceIcon2d, IconEdgeLocal, IconLocalFace,
        OwnedByFileLocal, SelectCircle, SelectLine, SelectTriangle, Vertex2d, LocalShape,
    },
    resources::{
        action::icon::IconAction,
        canvas::Canvas,
        icon_data::{IconFaceData, IconEdgeData, IconFaceKey, IconVertexData},
        input::{IconInputManager, CardinalDirection},
        shape_data::CanvasShape,
        tab_manager::TabManager,
    },
    shapes::{create_2d_edge_line, Line2d},
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
    grid_vertices: Vec<Entity>,

    // framing
    meshing: bool,
    frame_size: Vec2,
    frame_buffer: Vec2,
    frame_hover: Option<usize>,
    resync_frame_order: HashSet<Entity>,
    current_frame_index: usize,
    framing_y: f32,
    frame_duration_ms: f32,

    // file_entity -> file_frame_data
    file_frame_data: HashMap<Entity, FileFrameData>,
    // frame entity -> file_entity
    frames: HashMap<Entity, Entity>,

    preview_playing: bool,
    last_preview_instant: Instant,
    preview_elapsed_ms: f32,
    preview_frame_index: usize,

    //doubleclick
    pub(crate) last_left_click_instant: Instant,
    pub(crate) last_frame_index_hover: usize, //TODO: move this to IconInputManager?

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
    // queue of new faces to process (FaceKey, File Entity, Frame Entity)
    new_face_keys: Vec<(IconFaceKey, Entity, Entity)>,
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
            grid_vertices: Vec::new(),

            // framing
            meshing: false,
            resync_frame_order: HashSet::new(),
            frame_size: Vec2::new(100.0, 100.0),
            frame_buffer: Vec2::new(12.0, 12.0),
            frame_hover: None,
            current_frame_index: 0,
            framing_y: 0.0,
            frame_duration_ms: 40.0,

            file_frame_data: HashMap::new(),
            frames: HashMap::new(),

            preview_playing: false,
            last_preview_instant: Instant::now(),
            preview_elapsed_ms: 0.0,
            preview_frame_index: 0,

            last_left_click_instant: Instant::now(),
            last_frame_index_hover: 0,

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

    pub fn draw(&mut self, world: &mut World, current_file_entity: &Entity) {
        if self.meshing {
            self.draw_meshing(world, current_file_entity);
        } else {
            self.draw_framing(world);
        }
    }

    fn draw_meshing(&self, world: &mut World, current_file_entity: &Entity) {

        let current_frame_entity = self.current_frame_entity(current_file_entity).unwrap();

        let mut system_state: SystemState<(
            ResMut<RenderFrame>,
            Res<Canvas>,
            Res<TabManager>,
            Res<Input>,
            Query<(Entity, &OwnedByFileLocal), With<IconVertex>>,
            Query<&IconVertex>,
            Query<&IconLocalFace>,
            Query<(&Handle<CpuMesh>, &Handle<CpuMaterial>)>,
            Query<&mut Transform>,
        )> = SystemState::new(world);
        let (
            mut render_frame,
            canvas,
            tab_manager,
            input,
            mesh_vertex_q,
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

        let mut vertex_entities = Vec::new();
        let mut edge_entities = HashSet::new();
        let mut face_keys = HashSet::new();

        // collect grid vertices
        for vertex_entity in self.grid_vertices.iter() {
            vertex_entities.push(*vertex_entity);
        }

        // collect mesh vertices
        for (vertex_entity, owned_by_file) in mesh_vertex_q.iter() {
            if owned_by_file.file_entity != *current_file_entity {
                continue;
            }

            // draw vertex
            let Some(data) = self.get_vertex_data(&vertex_entity) else {
                continue;
            };
            if data.frame_entity_opt.unwrap() != current_frame_entity {
                continue;
            }

            vertex_entities.push(vertex_entity);
        }

        // draw vertices, collect edges
        for vertex_entity in vertex_entities {
            let vertex = vertex_q.get(vertex_entity).unwrap();
            let data = self.get_vertex_data(&vertex_entity).unwrap();

            let (mesh_handle, mat_handle) =
                object_q.get(vertex_entity).unwrap();
            let mut transform = transform_q.get_mut(vertex_entity).unwrap();

            transform.translation.x = vertex.x() as f32;
            transform.translation.y = vertex.y() as f32;

            render_frame.draw_object(Some(&self.render_layer), mesh_handle, mat_handle, &transform);

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

            let (mesh_handle, mat_handle) =
                object_q.get(*edge_entity).unwrap();
            render_frame.draw_object(Some(&self.render_layer), mesh_handle, mat_handle, &edge_transform);
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

            let (mesh_handle, mat_handle) =
                object_q.get(face_entity).unwrap();
            let mut face_transform = transform_q.get_mut(face_entity).unwrap();
            face_transform.translation = center_translation;

            render_frame.draw_object(Some(&self.render_layer), mesh_handle, mat_handle, &face_transform);
        }

        // draw select line & circle
        match self.selected_shape() {
            Some((vertex_entity, CanvasShape::Vertex)) => {

                // draw select circle
                let vertex_translation = transform_q.get(vertex_entity).unwrap().translation;

                let (mesh_handle, mat_handle) =
                    object_q.get(self.select_circle_entity).unwrap();
                let mut transform = transform_q.get_mut(self.select_circle_entity).unwrap();
                transform.translation = vertex_translation;

                render_frame.draw_object(Some(&self.render_layer), mesh_handle, &mat_handle, &transform);

                // draw select line
                let screen_mouse_position = input.mouse_position();
                let view_mouse_position = Self::screen_to_view(&canvas, &camera_transform, screen_mouse_position);
                let (mesh_handle, mat_handle) =
                    object_q.get(self.select_line_entity).unwrap();

                let mut transform = transform_q.get_mut(self.select_line_entity).unwrap();
                set_2d_line_transform(&mut transform, vertex_translation.truncate(), view_mouse_position, vertex_translation.z + 1.0);

                render_frame.draw_object(Some(&self.render_layer), mesh_handle, &mat_handle, &transform);
            }
            Some((edge_entity, CanvasShape::Edge)) => {

                let edge_transform = *transform_q.get(edge_entity).unwrap();

                // draw select line
                let (mesh_handle, mat_handle) =
                    object_q.get(self.select_line_entity).unwrap();

                let mut transform = transform_q.get_mut(self.select_line_entity).unwrap();
                transform.translation.x = edge_transform.translation.x;
                transform.translation.y = edge_transform.translation.y;
                transform.translation.z = edge_transform.translation.z + 1.0;
                transform.rotation = edge_transform.rotation;
                transform.scale.x = edge_transform.scale.x;
                transform.scale.y = edge_transform.scale.y + 2.0;

                render_frame.draw_object(Some(&self.render_layer), mesh_handle, &mat_handle, &transform);
            }
            Some((face_entity, CanvasShape::Face)) => {

                let face_translation = transform_q.get(face_entity).unwrap().translation;

                // draw select triangle
                let (mesh_handle, mat_handle) =
                    object_q.get(self.select_triangle_entity).unwrap();

                let mut transform = transform_q.get_mut(self.select_triangle_entity).unwrap();
                transform.translation = face_translation;

                render_frame.draw_object(Some(&self.render_layer), mesh_handle, &mat_handle, &transform);
            }
            _ => {}
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

        // grid
        {
            self.setup_grid(
                commands,
                meshes,
                materials,
            );
        }
    }

    fn setup_grid(
        &mut self,
        commands: &mut Commands,
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
    ) {
        let grid_size: f32 = 100.0;
        let neg_grid_size: f32 = -grid_size;

        let vertex_entity_a = self.new_local_vertex(
            commands,
            meshes,
            materials,
            Vec2::new(neg_grid_size, neg_grid_size),
            Color::LIGHT_GRAY,
        );
        let vertex_entity_b = self.new_local_vertex(
            commands,
            meshes,
            materials,
            Vec2::new(grid_size, neg_grid_size),
            Color::LIGHT_GRAY,
        );
        let vertex_entity_c = self.new_local_vertex(
            commands,
            meshes,
            materials,
            Vec2::new(grid_size, grid_size),
            Color::LIGHT_GRAY,
        );
        let vertex_entity_d = self.new_local_vertex(
            commands,
            meshes,
            materials,
            Vec2::new(neg_grid_size, grid_size),
            Color::LIGHT_GRAY,
        );
        self.grid_vertices.push(vertex_entity_a);
        self.grid_vertices.push(vertex_entity_b);
        self.grid_vertices.push(vertex_entity_c);
        self.grid_vertices.push(vertex_entity_d);

        let edge_entity_a = self.new_local_edge(
            commands,
            meshes,
            materials,
            vertex_entity_a,
            vertex_entity_b,
            Color::LIGHT_GRAY,
        );
        let edge_entity_b = self.new_local_edge(
            commands,
            meshes,
            materials,
            vertex_entity_b,
            vertex_entity_c,
            Color::LIGHT_GRAY,
        );
        let edge_entity_c = self.new_local_edge(
            commands,
            meshes,
            materials,
            vertex_entity_c,
            vertex_entity_d,
            Color::LIGHT_GRAY,
        );
        let edge_entity_d = self.new_local_edge(
            commands,
            meshes,
            materials,
            vertex_entity_d,
            vertex_entity_a,
            Color::LIGHT_GRAY,
        );
    }

    fn new_local_vertex(
        &mut self,
        commands: &mut Commands,
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
        position: Vec2,
        color: Color,
    ) -> Entity {

        // vertex
        let mut vertex_component = IconVertex::new(0, 0);
        vertex_component.localize();
        vertex_component.set_vec2(&position);
        let new_vertex_entity = commands.spawn_empty().insert(vertex_component).id();

        self.vertex_postprocess(
            commands,
            meshes,
            materials,
            None,
            None,
            new_vertex_entity,
            color,
        );

        commands.entity(new_vertex_entity).insert(LocalShape);

        return new_vertex_entity;
    }

    fn new_local_edge(
        &mut self,
        commands: &mut Commands,
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
        vertex_entity_a: Entity,
        vertex_entity_b: Entity,
        color: Color,
    ) -> Entity {

        let new_edge_entity = commands
            .spawn_empty()
            .id();

        self.edge_postprocess(
            commands,
            meshes,
            materials,
            None,
            None,
            new_edge_entity,
            vertex_entity_a,
            vertex_entity_b,
            color,
        );

        commands.entity(new_edge_entity).insert(LocalShape);

        new_edge_entity
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

        let canvas_size = canvas.texture_size();
        let Ok(camera_transform) = transform_q.get(self.camera_entity) else {
            return;
        };
        let view_mouse_position = Self::screen_to_view(&canvas, camera_transform, screen_mouse_position);

        if self.meshing {
            self.sync_mouse_hover_ui_meshing(world, current_file_entity, &view_mouse_position);
        } else {
            self.sync_mouse_hover_ui_framing(current_file_entity, &canvas_size, &view_mouse_position);
        }
    }

    fn sync_mouse_hover_ui_meshing(
        &mut self,
        world: &mut World,
        current_file_entity: &Entity,
        view_mouse_position: &Vec2,
    ) {
        let frame_entity = self.current_frame_entity(current_file_entity).unwrap();

        // sync to hover
        let new_hover_entity = IconInputManager::sync_mouse_hover_ui(self, world, current_file_entity, &frame_entity, view_mouse_position);
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

    pub(crate) fn vertex_get_frame_entity(&self, vertex_entity: &Entity) -> Option<Entity> {
        self.vertices.get(vertex_entity).map(|data| data.frame_entity_opt.unwrap())
    }

    pub(crate) fn edge_get_frame_entity(&self, edge_entity: &Entity) -> Option<Entity> {
        self.edges.get(edge_entity).map(|data| data.frame_entity_opt.unwrap())
    }

    pub(crate) fn face_get_frame_entity(&self, local_face_entity: &Entity) -> Option<Entity> {
        let face_key = self.local_faces.get(local_face_entity)?;
        let face_data_opt = self.face_keys.get(face_key)?;
        face_data_opt.as_ref().map(|data| data.frame_entity)
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

    fn take_drags(&mut self) -> Option<Vec<(Entity, Vec2, Vec2)>> {
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
        file_entity: &Entity,
        frame_entity: &Entity,
        position: Vec2,
        entities_to_release: &mut Vec<Entity>,
    ) -> Entity {
        // create new vertex
        let mut owned_by_file_component = OwnedByFile::new();
        owned_by_file_component
            .file_entity
            .set(client, file_entity);
        let mut vertex_component = IconVertex::from_vec2(position);
        vertex_component.frame_entity.set(client, frame_entity);
        let new_vertex_entity = commands
            .spawn_empty()
            .enable_replication(client)
            .configure_replication(ReplicationConfig::Delegated)
            .insert(vertex_component)
            .insert(owned_by_file_component)
            .id();

        entities_to_release.push(new_vertex_entity);

        // add local components to vertex
        self.vertex_postprocess(
            commands,
            meshes,
            materials,
            Some(*file_entity),
            Some(*frame_entity),
            new_vertex_entity,
            Vertex2d::ENABLED_COLOR,
        );

        return new_vertex_entity;
    }

    pub fn vertex_postprocess(
        &mut self,
        commands: &mut Commands,
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
        file_entity_opt: Option<Entity>,
        frame_entity_opt: Option<Entity>,
        vertex_entity: Entity,
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

        if let Some(file_entity) = file_entity_opt {
            commands
                .entity(vertex_entity)
                .insert(OwnedByFileLocal::new(file_entity));
        }

        self.register_vertex(frame_entity_opt, vertex_entity);
    }

    fn register_vertex(&mut self, frame_entity_opt: Option<Entity>, vertex_entity: Entity) {
        self.vertices.insert(vertex_entity, IconVertexData::new(frame_entity_opt));
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

    fn vertex_add_edge(&mut self, vertex_entity: &Entity, edge_entity: Entity) {
        let Some(vertex_data) = self.vertices.get_mut(&vertex_entity) else {
            panic!("Vertex entity: `{:?}` has not been registered", vertex_entity);
        };
        vertex_data.add_edge(edge_entity);
    }

    fn vertex_remove_edge(&mut self, vertex_entity: &Entity, edge_entity: &Entity) {
        // at this point, vertex_entity may have already been deregistered
        if let Some(vertex_data) = self.vertices.get_mut(vertex_entity) {
            vertex_data.remove_edge(edge_entity);
        }
    }

    pub(crate) fn vertex_get_faces(&self, vertex_entity: &Entity) -> Option<&HashSet<IconFaceKey>> {
        self.vertices.get(vertex_entity).map(|data| &data.faces)
    }

    fn vertex_add_face(&mut self, vertex_entity: &Entity, face_key: IconFaceKey) {
        // at this point, vertex_entity may have already been deregistered
        if let Some(vertex_data) = self.vertices.get_mut(vertex_entity) {
            vertex_data.add_face(face_key);
        };
    }

    fn vertex_remove_face(&mut self, vertex_entity: &Entity, face_key: &IconFaceKey) {
        // at this point, vertex_entity may have already been deregistered
        if let Some(vertex_data) = self.vertices.get_mut(vertex_entity) {
            vertex_data.remove_face(face_key);
        };
    }

    fn unregister_vertex(&mut self, entity: &Entity) {
        self.vertices.remove(entity);
    }

    fn get_vertex_data(&self, entity: &Entity) -> Option<&IconVertexData> {
        self.vertices.get(entity)
    }

    fn get_connected_vertices(&self, vertex_entity: Entity) -> HashSet<Entity> {
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
        vertex_entity_a: &Entity,
        vertex_entity_b: &Entity,
        file_entity: &Entity,
        frame_entity: &Entity,
        entities_to_release: &mut Vec<Entity>,
    ) -> Entity {
        // create new edge
        let mut new_edge_component = IconEdge::new();
        new_edge_component.frame_entity.set(client, frame_entity);
        new_edge_component.start.set(client, vertex_entity_a);
        new_edge_component.end.set(client, vertex_entity_b);
        let mut owned_by_file_component = OwnedByFile::new();
        owned_by_file_component
            .file_entity
            .set(client, file_entity);
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
            Some(*file_entity),
            Some(*frame_entity),
            new_edge_entity,
            *vertex_entity_a,
            *vertex_entity_b,
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
        file_entity_opt: Option<Entity>,
        frame_entity_opt: Option<Entity>,
        edge_entity: Entity,
        vertex_entity_a: Entity,
        vertex_entity_b: Entity,
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
        if let Some(file_entity) = file_entity_opt {
            commands
                .entity(edge_entity)
                .insert(OwnedByFileLocal::new(file_entity));
        }

        // register edge
        self.register_edge(file_entity_opt, frame_entity_opt, edge_entity, vertex_entity_a, vertex_entity_b);
    }

    fn register_edge(
        &mut self,
        file_entity_opt: Option<Entity>,
        frame_entity_opt: Option<Entity>,
        edge_entity: Entity,
        vertex_entity_a: Entity,
        vertex_entity_b: Entity,
    ) {
        for vertex_entity in [vertex_entity_a, vertex_entity_b] {
            self.vertex_add_edge(&vertex_entity, edge_entity);
        }

        self.edges.insert(
            edge_entity,
            IconEdgeData::new(frame_entity_opt, vertex_entity_a, vertex_entity_b),
        );

        if let Some(file_entity) = file_entity_opt {
            let frame_entity = frame_entity_opt.unwrap();
            self.check_for_new_faces(file_entity, frame_entity, vertex_entity_a, vertex_entity_b);
        }
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

    fn edge_add_face(&mut self, edge_entity: &Entity, face_key: IconFaceKey) {
        self.edges
            .get_mut(edge_entity)
            .unwrap()
            .faces
            .insert(face_key);
    }

    fn edge_remove_face(&mut self, edge_entity: &Entity, face_key: &IconFaceKey) {
        self.edges
            .get_mut(edge_entity)
            .unwrap()
            .faces
            .remove(face_key);
    }

    fn edge_get_endpoints(&self, edge_entity: &Entity) -> (Entity, Entity) {
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
        for (face_key, file_entity, frame_entity) in keys {
            self.process_new_local_face(commands, meshes, materials, &file_entity, &frame_entity, &face_key);
        }
    }

    // return local face entity
    pub fn process_new_local_face(
        &mut self,
        commands: &mut Commands,
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
        file_entity: &Entity,
        frame_entity: &Entity,
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
            .insert(OwnedByFileLocal::new(*file_entity));

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
                *file_entity,
                *frame_entity,
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
        let file_entity = face_data.file_entity;
        let frame_entity = face_data.frame_entity;
        let edge_entity_a = face_data.edge_a;
        let edge_entity_b = face_data.edge_b;
        let edge_entity_c = face_data.edge_c;

        let mut system_state: SystemState<(
            Commands,
            Client,
            ResMut<Assets<CpuMesh>>,
            ResMut<Assets<CpuMaterial>>,
            Query<&Transform>,
        )> = SystemState::new(world);
        let (
            mut commands,
            mut client,
            mut meshes,
            mut materials,
            transform_q
        ) = system_state.get_mut(world);

        self.create_networked_face(
            &mut commands,
            &mut client,
            &mut meshes,
            &mut materials,
            &transform_q,
            &face_key,
            [edge_entity_a, edge_entity_b, edge_entity_c],
            &file_entity,
            &frame_entity,
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
        file_entity: &Entity,
        frame_entity: &Entity,
    ) {
        info!("creating networked face");

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
        face_component.frame_entity.set(client, frame_entity);
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
            .set(client, file_entity);

        // set up net entity
        let face_net_entity = commands
            .spawn_empty()
            .enable_replication(client)
            .configure_replication(ReplicationConfig::Delegated)
            .insert(owned_by_file_component)
            .insert(OwnedByFileLocal::new(*file_entity))
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

    fn register_net_face(&mut self, net_face_entity: Entity, face_key: &IconFaceKey) {
        self.net_faces.insert(net_face_entity, *face_key);

        let Some(Some(face_data)) = self.face_keys.get_mut(face_key) else {
            panic!("FaceKey: `{:?}` has not been registered", face_key);
        };
        face_data.net_entity = Some(net_face_entity);
    }

    pub fn remove_new_face_key(&mut self, face_key: &IconFaceKey) {
        self.new_face_keys.retain(|(key, _, _)| key != face_key);
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
    fn cleanup_deleted_face_key(
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

    fn check_for_new_faces(
        &mut self,
        file_entity: Entity,
        frame_entity: Entity,
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
                self.new_face_keys.push((face_key, file_entity, frame_entity));
            }
        }
    }

    // Framing

    pub(crate) fn current_frame_index(&self) -> usize {
        self.current_frame_index
    }

    pub fn set_current_frame_index(&mut self, frame_index: usize) {
        self.current_frame_index = frame_index;
    }

    pub fn preview_is_playing(&self) -> bool {
        self.preview_playing
    }

    pub fn preview_play(&mut self) {
        self.preview_playing = true;
        self.last_preview_instant = Instant::now();
    }

    pub fn preview_pause(&mut self) {
        self.preview_playing = false;
    }

    pub fn frame_index_hover(&self) -> Option<usize> {
        self.frame_hover
    }

    pub fn is_meshing(&self) -> bool {
        self.meshing
    }

    pub fn is_framing(&self) -> bool {
        !self.meshing
    }

    pub fn set_meshing(&mut self) {
        self.meshing = true;
    }

    pub fn set_framing(&mut self) {
        self.meshing = false;
    }

    pub fn get_frame_entity(&self, file_entity: &Entity, frame_index: usize) -> Option<Entity> {
        //info!("get_frame_entity({:?}, {:?})", file_entity, frame_index);
        let frame_data = self.file_frame_data.get(file_entity)?;
        //info!("frame list: {:?}", frame_data.frame_list);
        let entity_opt = frame_data.frame_list.get(frame_index)?.as_ref();
        let entity = entity_opt?;
        Some(*entity)
    }

    pub(crate) fn current_frame_entity(&self, file_entity: &Entity) -> Option<Entity> {
        let current_frame_index = self.current_frame_index;
        let frame_data = self.file_frame_data.get(file_entity)?; //&(*file_entity, current_frame_index)).copied()
        let entity_opt = frame_data.frame_list.get(current_frame_index)?.as_ref();
        let entity = entity_opt?;
        Some(*entity)
    }

    pub(crate) fn get_frame_count(&self, file_entity: &Entity) -> Option<usize> {
        let frame_data = self.file_frame_data.get(file_entity)?;
        Some(frame_data.frame_list.len())
    }

    pub(crate) fn register_frame(&mut self, file_entity: Entity, frame_entity: Entity) {
        if !self.file_frame_data.contains_key(&file_entity) {
            self.file_frame_data.insert(file_entity, FileFrameData::new());
        }
        let frame_data = self.file_frame_data.get_mut(&file_entity).unwrap();
        frame_data.register_frame(frame_entity);

        self.frames.insert(frame_entity, file_entity);

        self.framing_queue_resync_frame_order(&file_entity);
    }

    pub(crate) fn deregister_frame(&mut self, file_entity: &Entity, frame_entity: &Entity) {
        if !self.file_frame_data.contains_key(file_entity) {
            panic!("Frame data not found!");
        }

        let frame_data = self.file_frame_data.get_mut(file_entity).unwrap();
        frame_data.deregister_frame(frame_entity);

        if frame_data.frames.is_empty() {
            self.file_frame_data.remove(file_entity);
        }

        self.frames.remove(frame_entity);

        self.framing_queue_resync_frame_order(file_entity);

        // TODO: handle current selected frame ... harder to do because can we really suppose that
        // the current tab file entity is the same as the file entity here?
    }

    fn get_frame_positions(&mut self, canvas_size: &Vec2, frame_count: usize) -> Vec<Vec2> {
        let canvas_size = *canvas_size * 0.5;
        let mut positions = Vec::new();
        let mut cursor_position = self.frame_buffer - canvas_size;

        for _ in 0..=frame_count {
            positions.push(cursor_position);
            let next_x = cursor_position.x + self.frame_size.x + self.frame_buffer.x;
            if next_x + self.frame_size.x > canvas_size.x {
                cursor_position.x = self.frame_buffer.x - canvas_size.x;
                cursor_position.y += self.frame_size.y + self.frame_buffer.y;
            } else {
                cursor_position.x = next_x;
            }
        }

        let last_y = cursor_position.y + self.frame_size.y + self.frame_buffer.y;
        let y_diff = last_y - canvas_size.y;
        if y_diff <= 0.0 {
            self.framing_y = 0.0;
        } else {
            if self.framing_y > 0.0 {
                self.framing_y = 0.0;
            }
            if self.framing_y < -y_diff {
                self.framing_y = -y_diff;
            }
        }

        for position in positions.iter_mut() {
            position.y += self.framing_y;
        }

        positions
    }

    pub fn framing_queue_resync_frame_order(&mut self, file_entity: &Entity) {
        info!(
            "framing_queue_resync_frame_order for entity: `{:?}`",
            file_entity
        );
        self.resync_frame_order.insert(*file_entity);
    }

    pub fn framing_resync_frame_order(
        &mut self,
        client: &Client,
        frame_q: &Query<(Entity, &IconFrame)>,
    ) {
        if self.resync_frame_order.is_empty() {
            return;
        }
        let resync_frame_order = std::mem::take(&mut self.resync_frame_order);
        for file_entity in resync_frame_order {
            // info!("resync_frame_order for entity: `{:?}`", file_entity);
            self.framing_recalc_order(client, &file_entity, frame_q);
        }
    }

    pub fn framing_insert_frame(
        &mut self,
        commands: &mut Commands,
        client: &mut Client,
        file_entity: Entity,
        frame_index: usize,
    ) -> Entity {
        let mut frame_component = IconFrame::new(frame_index as u8);
        frame_component.file_entity.set(client, &file_entity);
        let entity_id = commands
            .spawn_empty()
            .enable_replication(client)
            .configure_replication(ReplicationConfig::Delegated)
            .insert(frame_component)
            .id();

        // create new 2d vertex, add local components to 3d vertex
        self.frame_postprocess(file_entity, entity_id);

        entity_id
    }

    fn framing_recalc_order(
        &mut self,
        client: &Client,
        file_entity: &Entity,
        frame_q: &Query<(Entity, &IconFrame)>,
    ) {
        let Some(frame_data) = self.file_frame_data.get_mut(&file_entity) else {
            return;
        };

        let mut new_frame_list = Vec::new();

        for (frame_entity, frame) in frame_q.iter() {
            let frames_file_entity = frame.file_entity.get(client).unwrap();
            if frames_file_entity != *file_entity {
                continue;
            }
            let frame_index = frame.get_order() as usize;
            // resize if necessary
            if frame_index >= new_frame_list.len() {
                new_frame_list.resize(frame_index + 1, None);
            }
            if new_frame_list[frame_index].is_some() {
                warn!("Duplicate frame order! {:?}", frame_index);
            }
            new_frame_list[frame_index] = Some(frame_entity);
        }

        for (index, frame_entity_opt) in new_frame_list.iter().enumerate() {
            info!("frame order: {:?} -> {:?}", index, frame_entity_opt);
        }

        frame_data.frame_list = new_frame_list;
    }

    pub(crate) fn frame_postprocess(&mut self, file_entity: Entity, frame_entity: Entity) {
        self.register_frame(file_entity, frame_entity);
    }

    pub(crate) fn preview_update(
        &mut self,
        current_file_entity: &Entity,
    ) {
        if !self.preview_playing {
            return;
        }

        let ms_elapsed = self.last_preview_instant.elapsed().as_millis() as f32;
        self.last_preview_instant = Instant::now();

        let Some(preview_frame_count) = self.get_frame_count(current_file_entity) else {
            return;
        };

        const FRAME_DURATION: f32 = 40.0; // TODO: move this somewhere more sensible ... or have variable animation speed ..

        self.preview_elapsed_ms += ms_elapsed / 10.0; // change this back to 1 for real speeds! maybe should be configurable..
        while self.preview_elapsed_ms > FRAME_DURATION {
            self.preview_elapsed_ms -= FRAME_DURATION;
            self.preview_frame_index += 1;
            if self.preview_frame_index >= preview_frame_count {
                self.preview_frame_index = 0;
            }
        }
    }

    pub(crate) fn handle_mouse_drag_framing(&mut self, delta_y: f32) {
        self.framing_y += delta_y;
    }

    pub fn framing_navigate(
        &mut self,
        current_file_entity: &Entity,
        dir: CardinalDirection,
    ) -> Option<(usize, usize)> {
        let mut current_index = self.current_frame_index;
        let Some(frame_data) = self.file_frame_data.get(current_file_entity) else {
            return None;
        };
        match dir {
            CardinalDirection::West => {
                if current_index <= 0 {
                    return None;
                }
                current_index -= 1;
                // if no frame entity, continue decrementing
                while frame_data.frame_list[current_index].is_none() {
                    if current_index <= 0 {
                        return None;
                    }
                    current_index -= 1;
                }
                return Some((self.current_frame_index, current_index));
            }
            CardinalDirection::East => {
                if current_index >= frame_data.frame_list.len() - 1 {
                    return None;
                }
                current_index += 1;
                // if no frame entity, continue incrementing
                while frame_data.frame_list[current_index].is_none() {
                    if current_index >= frame_data.frame_list.len() - 1 {
                        return None;
                    }
                    current_index += 1;
                }
                return Some((self.current_frame_index, current_index));
            }
            _ => {
                return None;
            }
        }
    }

    pub fn framing_handle_mouse_wheel(&mut self, scroll_y: f32) {
        let scroll_y = 0.8 + (((scroll_y + 24.0) / 48.0) * 0.4);
        self.frame_size *= scroll_y;
        if self.frame_size.x < 40.0 {
            self.frame_size = Vec2::new(40.0, 40.0);
        }
        if self.frame_size.x > 400.0 {
            self.frame_size = Vec2::new(400.0, 400.0);
        }
    }

    fn sync_mouse_hover_ui_framing(
        &mut self,
        current_file_entity: &Entity,
        canvas_size: &Vec2,
        view_mouse_position: &Vec2,
    ) {
        let Some(file_frame_data) = self.file_frame_data.get(current_file_entity) else {
            return;
        };

        let frame_count = file_frame_data.count();

        let frame_positions = self.get_frame_positions(canvas_size, frame_count);

        self.frame_hover = None;
        for (index, frame_position) in frame_positions.iter().enumerate() {
            // assign hover frame
            if view_mouse_position.x >= frame_position.x
                && view_mouse_position.x <= frame_position.x + self.frame_size.x
            {
                if view_mouse_position.y >= frame_position.y
                    && view_mouse_position.y <= frame_position.y + self.frame_size.y
                {
                    self.frame_hover = Some(index);
                    return;
                }
            }
        }
    }

    fn draw_framing(&mut self, world: &mut World) {

        let mut system_state: SystemState<(
            Res<TabManager>,
            Res<Canvas>,
            Query<&mut Transform>,
        )> = SystemState::new(world);
        let (
            mut tab_manager,
            canvas,
            mut transform_q,
        ) = system_state.get_mut(world);

        // get current file
        let Some(current_file_entity) = tab_manager.current_tab_entity() else {
            return;
        };
        let current_file_entity = *current_file_entity;

        // camera
        let Ok(mut camera_transform) = transform_q.get_mut(self.camera_entity) else {
            return;
        };
        camera_transform.translation = Vec3::new(0.0, 0.0, 1.0);
        camera_transform.scale = Vec3::new(1.0, 1.0, 1.0);

        // frames
        let Some(file_frame_data) = self.file_frame_data.get(&current_file_entity) else {
            return;
        };
        let frame_count = file_frame_data.count();
        let canvas_size = canvas.texture_size();
        let frame_rects = self.get_frame_positions(&canvas_size, frame_count);

        let file_frame_data = self.file_frame_data.get(&current_file_entity).unwrap();

        let (
            frame_rects,
            point_mesh_handle,
            line_mesh_handle,
            mat_handle_white,
            mat_handle_green,
        ) = {
            // draw
            let mut system_state: SystemState<(
                ResMut<RenderFrame>,
                ResMut<Assets<CpuMesh>>,
                ResMut<Assets<CpuMaterial>>,
            )> = SystemState::new(world);
            let (
                mut render_frame,
                mut meshes,
                mut materials,
            ) = system_state.get_mut(world);

            let render_layer = self.render_layer;
            let point_mesh_handle = meshes.add(Circle::new(Vertex2d::SUBDIVISIONS));
            let line_mesh_handle = meshes.add(Line2d);
            let mat_handle_white = materials.add(Color::WHITE);
            let mat_handle_gray = materials.add(Color::GRAY);
            let mat_handle_dark_gray = materials.add(Color::DARK_GRAY);
            let mat_handle_green = materials.add(Color::GREEN);

            for (frame_index, frame_pos) in frame_rects.iter().enumerate() {
                // frame_index 0 is preview frame
                let selected: bool = frame_index > 0 && self.current_frame_index == frame_index - 1;

                // set thickness to 4.0 if frame is hovered and not currently selected, otherwise 2.0
                let thickness = if !selected && Some(frame_index) == self.frame_hover {
                    4.0
                } else {
                    2.0
                };

                let mat = if frame_index == 0 {
                    mat_handle_dark_gray
                } else {
                    mat_handle_gray
                };

                draw_rectangle(
                    &mut render_frame,
                    &render_layer,
                    &line_mesh_handle,
                    &mat,
                    *frame_pos,
                    self.frame_size,
                    thickness,
                );

                if selected {
                    // draw white rectangle around selected frame
                    draw_rectangle(
                        &mut render_frame,
                        &render_layer,
                        &line_mesh_handle,
                        &mat_handle_white,
                        *frame_pos + Vec2::new(-4.0, -4.0),
                        self.frame_size + Vec2::new(8.0, 8.0),
                        thickness,
                    );
                }
            }

            (
                frame_rects,
                point_mesh_handle,
                line_mesh_handle,
                mat_handle_white,
                mat_handle_green,
            )
        };

        let mut frame_index = 0;

        {
            // draw preview frame
            if let Some(Some(preview_current_frame_entity)) =
                file_frame_data.frame_list.get(self.preview_frame_index)
            {
                let frame_pos = frame_rects[frame_index] + (self.frame_size * 0.5);

                self.draw_frame_contents(
                    world,
                    &current_file_entity,
                    &preview_current_frame_entity,
                    &frame_pos,
                    &point_mesh_handle,
                    &line_mesh_handle,
                    &mat_handle_green,
                );
            }

            frame_index += 1;
        }

        for frame_opt in file_frame_data.frame_list.iter() {
            if frame_opt.is_none() {
                continue;
            }
            let frame_entity = frame_opt.unwrap();

            let frame_pos = frame_rects[frame_index] + (self.frame_size * 0.5);

            self.draw_frame_contents(
                world,
                &current_file_entity,
                &frame_entity,
                &frame_pos,
                &point_mesh_handle,
                &line_mesh_handle,
                &mat_handle_green,
            );

            frame_index += 1;
        }

        self.draw_preview_time_line(
            world,
            &current_file_entity,
            &line_mesh_handle,
            &mat_handle_white,
            &frame_rects,
        );
    }

    fn draw_frame_contents(
        &self,
        world: &mut World,
        file_entity: &Entity,
        frame_entity: &Entity,
        frame_pos: &Vec2,
        point_mesh_handle: &Handle<CpuMesh>,
        line_mesh_handle: &Handle<CpuMesh>,
        mat_handle_green: &Handle<CpuMaterial>,
    ) {
        let mut system_state: SystemState<(
            Client,
            ResMut<RenderFrame>,
            Query<(Entity, &IconVertex)>,
            Query<&OwnedByFileLocal>,
        )> = SystemState::new(world);
        let (
            client,
            mut render_frame,
            vertex_q,
            owned_by_file_q,
        ) = system_state.get_mut(world);

        let mut edge_entities = HashSet::new();

        let size_ratio = (self.frame_size * 0.5) / 100.0;

        // draw vertices, collect edges
        for (vertex_entity, vertex) in vertex_q.iter() {
            let Ok(owned_by_file) = owned_by_file_q.get(vertex_entity) else {
                continue;
            };
            if owned_by_file.file_entity != *file_entity {
                continue;
            }
            let vertex_frame_entity = vertex.frame_entity.get(&client).unwrap();
            if vertex_frame_entity != *frame_entity {
                continue;
            }

            // draw vertex
            let Some(data) = self.get_vertex_data(&vertex_entity) else {
                continue;
            };

            let mut vertex_pos = vertex.as_vec2();
            vertex_pos.x *= size_ratio.x;
            vertex_pos.y *= size_ratio.y;
            let vertex_pos = *frame_pos + vertex_pos;
            let transform = Transform::from_translation_2d(vertex_pos);
            render_frame.draw_object(Some(&self.render_layer), point_mesh_handle, mat_handle_green, &transform);

            for edge_entity in data.edges.iter() {
                edge_entities.insert(*edge_entity);
            }
        }

        // draw edges
        for edge_entity in edge_entities.iter() {

            let (start, end) = self.edge_get_endpoints(edge_entity);

            // sync
            let (_, start_vertex) = vertex_q.get(start).unwrap();

            let mut start_pos = start_vertex.as_vec2();
            start_pos.x *= size_ratio.x;
            start_pos.y *= size_ratio.y;
            let start_pos = *frame_pos + start_pos;

            let (_, end_vertex) = vertex_q.get(end).unwrap();

            let mut end_pos = end_vertex.as_vec2();
            end_pos.x *= size_ratio.x;
            end_pos.y *= size_ratio.y;
            let end_pos = *frame_pos + end_pos;

            let mut edge_transform = Transform::default();
            set_2d_line_transform(&mut edge_transform, start_pos, end_pos, 1.0);

            // draw
            render_frame.draw_object(Some(&self.render_layer), line_mesh_handle, mat_handle_green, &edge_transform);
        }
    }

    fn draw_preview_time_line(
        &self,
        world: &mut World,
        current_file_entity: &Entity,
        line_mesh_handle: &Handle<CpuMesh>,
        mat_handle_white: &Handle<CpuMaterial>,
        frame_positions: &Vec<Vec2>,
    ) {
        let Some(_frame_entity) = self.get_frame_entity(current_file_entity, self.preview_frame_index) else {
            return;
        };
        let complete = self.preview_elapsed_ms / self.frame_duration_ms;
        let frame_width = self.frame_size.x + self.frame_buffer.x;
        let frame_count = frame_positions.len();

        let mut start: Vec2;

        let mut preview_frame_index = self.preview_frame_index + 1;
        if preview_frame_index >= frame_count {
            preview_frame_index -= frame_count - 1;
        }

        start = frame_positions[preview_frame_index];

        start.x += frame_width * complete;

        start.y -= self.frame_buffer.y;

        let mut end = start;
        end.y += self.frame_size.y + (self.frame_buffer.y * 2.0);

        let mut render_frame = world.get_resource_mut::<RenderFrame>().unwrap();
        draw_line(
            &mut render_frame,
            &self.render_layer,
            line_mesh_handle,
            mat_handle_white,
            start,
            end,
            2.0,
        );
    }
}

struct FileFrameData {
    frames: HashSet<Entity>,
    frame_list: Vec<Option<Entity>>,
}

impl FileFrameData {
    pub fn new() -> Self {
        Self {
            frames: HashSet::new(),
            frame_list: Vec::new(),
        }
    }

    pub fn register_frame(&mut self, frame_entity: Entity) {
        self.frames.insert(frame_entity);
    }

    pub fn deregister_frame(&mut self, frame_entity: &Entity) {
        self.frames.remove(frame_entity);
    }

    pub fn count(&self) -> usize {
        let mut count = 0;
        for val_opt in &self.frame_list {
            if val_opt.is_some() {
                count += 1;
            }
        }
        count
    }
}

fn draw_rectangle(
    render_frame: &mut RenderFrame,
    render_layer: &RenderLayer,
    mesh_handle: &Handle<CpuMesh>,
    mat_handle: &Handle<CpuMaterial>,
    position: Vec2,
    size: Vec2,
    thickness: f32,
) {
    // top
    let start = position;
    let mut end = position;
    end.x += size.x;
    draw_line(
        render_frame,
        render_layer,
        mesh_handle,
        mat_handle,
        start,
        end,
        thickness,
    );

    // bottom
    let mut start = position;
    start.y += size.y;
    let mut end = start;
    end.x += size.x;
    draw_line(
        render_frame,
        render_layer,
        mesh_handle,
        mat_handle,
        start,
        end,
        thickness,
    );

    // left
    let start = position;
    let mut end = position;
    end.y += size.y;
    draw_line(
        render_frame,
        render_layer,
        mesh_handle,
        mat_handle,
        start,
        end,
        thickness,
    );

    // right
    let mut start = position;
    start.x += size.x;
    let mut end = start;
    end.y += size.y;
    draw_line(
        render_frame,
        render_layer,
        mesh_handle,
        mat_handle,
        start,
        end,
        thickness,
    );
}

fn draw_line(
    render_frame: &mut RenderFrame,
    render_layer: &RenderLayer,
    mesh_handle: &Handle<CpuMesh>,
    mat_handle: &Handle<CpuMaterial>,
    start: Vec2,
    end: Vec2,
    thickness: f32,
) {
    let mut transform = Transform::default();
    transform.scale.y = thickness;
    set_2d_line_transform(&mut transform, start, end, 0.0);
    render_frame.draw_object(Some(render_layer), mesh_handle, mat_handle, &transform);
}