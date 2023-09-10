use std::collections::{HashMap, HashSet};

use bevy_ecs::{
    entity::Entity,
    query::{With, Without},
    system::{Commands, Query, Res, ResMut, Resource, SystemState},
    world::World,
};
use bevy_log::{info, warn};

use input::MouseButton;
use naia_bevy_client::{Client, CommandsExt, Replicate, ReplicationConfig};

use math::{convert_2d_to_3d, convert_3d_to_2d, Vec2, Vec3};
use render_api::{
    base::{Color, CpuMaterial, CpuMesh},
    components::{Camera, CameraProjection, Projection, RenderObjectBundle, Transform, Visibility},
    shapes::{
        distance_to_2d_line, get_2d_line_transform_endpoint, set_2d_line_transform, HollowTriangle,
        Triangle,
    },
    Assets, Handle,
};

use vortex_proto::components::{EdgeAngle, Face3d, FileTypeValue, OwnedByFile, Vertex3d, VertexRoot};

use crate::app::{
    components::{
        Compass, Edge2dLocal, Edge3dLocal, Face3dLocal, FaceIcon2d, OwnedByFileLocal, SelectCircle,
        SelectTriangle, Vertex2d, VertexTypeData,
    },
    resources::{
        canvas::Canvas,
        action::{ActionStack, ShapeAction},
        camera_manager::{CameraAngle, CameraManager},
        camera_state::CameraState,
        input_manager::AppInputAction,
        tab_manager::TabState,
    },
    set_3d_line_transform,
    shapes::{
        create_2d_edge_arrow, create_2d_edge_line, create_3d_edge_diamond, create_3d_edge_line,
    },
};
use crate::app::components::EdgeAngleLocal;

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum CanvasShape {
    RootVertex,
    Vertex,
    Edge,
    Face,
}

struct Vertex3dData {
    entity_2d: Entity,
    edges_3d: HashSet<Entity>,
    faces_3d: HashSet<FaceKey>,
}

impl Vertex3dData {
    fn new(entity_2d: Entity) -> Self {
        Self {
            entity_2d,
            edges_3d: HashSet::new(),
            faces_3d: HashSet::new(),
        }
    }

    fn add_edge(&mut self, edge_3d_entity: Entity) {
        self.edges_3d.insert(edge_3d_entity);
    }

    fn remove_edge(&mut self, edge_3d_entity: &Entity) {
        self.edges_3d.remove(edge_3d_entity);
    }

    fn add_face(&mut self, face_key: FaceKey) {
        self.faces_3d.insert(face_key);
    }

    fn remove_face(&mut self, face_key: &FaceKey) {
        self.faces_3d.remove(face_key);
    }
}

struct Edge3dData {
    entity_2d: Entity,
    vertex_a_3d_entity: Entity,
    vertex_b_3d_entity: Entity,
    faces_3d: HashSet<FaceKey>,
    angle_entity_opt: Option<Entity>,
}

impl Edge3dData {
    fn new(entity_2d: Entity, vertex_a_3d_entity: Entity, vertex_b_3d_entity: Entity, angle_entity_opt: Option<Entity>) -> Self {
        Self {
            entity_2d,
            vertex_a_3d_entity,
            vertex_b_3d_entity,
            faces_3d: HashSet::new(),
            angle_entity_opt,
        }
    }

    fn add_face(&mut self, face_key: FaceKey) {
        self.faces_3d.insert(face_key);
    }

    fn remove_face(&mut self, face_key: &FaceKey) {
        self.faces_3d.remove(face_key);
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub struct FaceKey {
    pub vertex_3d_a: Entity,
    pub vertex_3d_b: Entity,
    pub vertex_3d_c: Entity,
}

impl FaceKey {
    pub fn new(vertex_a: Entity, vertex_b: Entity, vertex_c: Entity) -> Self {
        let mut vertices = vec![vertex_a, vertex_b, vertex_c];

        vertices.sort();

        Self {
            vertex_3d_a: vertices[0],
            vertex_3d_b: vertices[1],
            vertex_3d_c: vertices[2],
        }
    }
}

struct FaceData {
    entity_3d: Option<Entity>,
    entity_2d: Entity,
    file_entity: Entity,

    edge_3d_a: Entity,
    edge_3d_b: Entity,
    edge_3d_c: Entity,
}

impl FaceData {
    fn new(
        entity_2d: Entity,
        file_entity: Entity,
        edge_3d_a: Entity,
        edge_3d_b: Entity,
        edge_3d_c: Entity,
    ) -> Self {
        Self {
            entity_2d,
            entity_3d: None,
            file_entity,
            edge_3d_a,
            edge_3d_b,
            edge_3d_c,
        }
    }
}

#[derive(Resource)]
pub struct ShapeManager {
    current_file_type: FileTypeValue,

    // 3d vertex entity -> 3d vertex data
    vertices_3d: HashMap<Entity, Vertex3dData>,
    // 2d vertex entity -> 3d vertex entity
    vertices_2d: HashMap<Entity, Entity>,

    // 3d edge entity -> 3d edge data
    edges_3d: HashMap<Entity, Edge3dData>,
    // 2d edge entity -> 3d edge entity
    edges_2d: HashMap<Entity, Entity>,

    // 3d face key -> 3d face entity
    face_keys: HashMap<FaceKey, Option<FaceData>>,
    // 3d face entity -> 3d face data
    faces_3d: HashMap<Entity, FaceKey>,
    // 2d face entity -> 3d face entity
    faces_2d: HashMap<Entity, FaceKey>,
    // queue of new faces to process
    new_face_keys: Vec<(FaceKey, Entity)>,

    shapes_recalc: u8,
    selection_recalc: bool,
    hover_recalc: bool,

    // Option<(2d shape entity, shape type)>
    hovered_entity: Option<(Entity, CanvasShape)>,

    pub select_circle_entity: Option<Entity>,
    pub select_triangle_entity: Option<Entity>,
    pub select_line_entity: Option<Entity>,

    // Option<(2d shape entity, shape type)>
    selected_shape: Option<(Entity, CanvasShape)>,

    last_vertex_dragged: Option<(Entity, Vec3, Vec3)>,
    compass_vertices: Vec<Entity>,
}

impl ShapeManager {
    pub(crate) fn on_canvas_focus_changed(&mut self, new_focus: bool) {
        self.recalculate_selection();
        if !new_focus {
            self.last_vertex_dragged = None;
            self.hovered_entity = None;
        }
    }
}

impl Default for ShapeManager {
    fn default() -> Self {
        Self {
            current_file_type: FileTypeValue::Skel,

            vertices_3d: HashMap::new(),
            vertices_2d: HashMap::new(),

            edges_3d: HashMap::new(),
            edges_2d: HashMap::new(),

            new_face_keys: Vec::new(),
            face_keys: HashMap::new(),
            faces_2d: HashMap::new(),
            faces_3d: HashMap::new(),

            shapes_recalc: 0,
            selection_recalc: false,
            hover_recalc: false,

            hovered_entity: None,

            select_circle_entity: None,
            select_triangle_entity: None,
            select_line_entity: None,
            selected_shape: None,

            last_vertex_dragged: None,
            compass_vertices: Vec::new(),
        }
    }
}

impl ShapeManager {
    pub fn update_input(
        &mut self,

        // input
        input_actions: Vec<AppInputAction>,

        // resources
        commands: &mut Commands,
        client: &mut Client,
        camera_manager: &mut CameraManager,
        tab_state: &mut TabState,

        // queries
        transform_q: &mut Query<&mut Transform>,
        camera_q: &mut Query<(&mut Camera, &mut Projection)>,
        vertex_3d_q: &mut Query<&mut Vertex3d>,
    ) {
        let camera_state = &mut tab_state.camera_state;

        for input_action in &input_actions {
            match input_action {
                AppInputAction::MiddleMouseScroll(scroll_y) => {
                    camera_manager.camera_zoom(camera_state, *scroll_y);
                }
                AppInputAction::MouseMoved => {
                    self.recalculate_hover();
                    self.recalculate_selection();
                }
                AppInputAction::SwitchTo3dMode => {
                    // disable 2d camera, enable 3d camera
                    camera_state.set_3d_mode();
                    camera_manager.recalculate_3d_view();
                    self.recalculate_shapes();
                }
                AppInputAction::SwitchTo2dMode => {
                    // disable 3d camera, enable 2d camera
                    camera_state.set_2d_mode();
                    camera_manager.recalculate_3d_view();
                    self.recalculate_shapes();
                }
                AppInputAction::SetCameraAngleFixed(camera_angle) => match camera_angle {
                    CameraAngle::Side => {
                        camera_manager.set_camera_angle_side(camera_state);
                    }
                    CameraAngle::Front => {
                        camera_manager.set_camera_angle_front(camera_state);
                    }
                    CameraAngle::Top => {
                        camera_manager.set_camera_angle_top(camera_state);
                    }
                    CameraAngle::Ingame(angle_index) => {
                        camera_manager.set_camera_angle_ingame(camera_state, *angle_index);
                    }
                },
                AppInputAction::InsertKeyPress => {
                    self.handle_insert_key_press(&mut tab_state.action_stack);
                }
                AppInputAction::DeleteKeyPress => {
                    self.handle_delete_key_press(commands, client, &mut tab_state.action_stack);
                }
                AppInputAction::CameraAngleYawRotate(clockwise) => {
                    camera_manager.set_camera_angle_yaw_rotate(camera_state, *clockwise);
                }
                AppInputAction::MouseDragged(click_type, mouse_position, delta) => {
                    self.handle_mouse_drag(
                        commands,
                        client,
                        camera_manager,
                        camera_state,
                        *click_type,
                        *mouse_position,
                        *delta,
                        camera_q,
                        transform_q,
                        vertex_3d_q,
                    );
                }
                AppInputAction::MouseClick(click_type, mouse_position) => {
                    self.handle_mouse_click(
                        camera_manager,
                        &mut tab_state.action_stack,
                        *click_type,
                        mouse_position,
                        camera_q,
                        transform_q,
                    );
                }
                AppInputAction::MouseRelease(MouseButton::Left) => {
                    if let Some((vertex_2d_entity, old_pos, new_pos)) =
                        self.last_vertex_dragged.take()
                    {
                        tab_state
                            .action_stack
                            .buffer_action(ShapeAction::MoveVertex(
                                vertex_2d_entity,
                                old_pos,
                                new_pos,
                            ));
                    }
                }
                _ => {}
            }
        }
    }

    pub fn sync_shapes(
        &mut self,
        camera_manager: &CameraManager,
        camera_state: &CameraState,
        current_tab_file_entity: Entity,

        camera_q: &Query<(&Camera, &Projection)>,
        compass_q: &Query<&Compass>,

        transform_q: &mut Query<&mut Transform>,
        owned_by_q: &Query<&OwnedByFileLocal>,

        vertex_3d_q: &mut Query<(Entity, &mut Vertex3d)>,
        edge_2d_q: &Query<(Entity, &Edge2dLocal)>,
        edge_3d_q: &Query<(Entity, &Edge3dLocal, Option<&EdgeAngle>)>,
        face_2d_q: &Query<(Entity, &FaceIcon2d)>,
    ) {
        if self.shapes_recalc == 0 {
            return;
        }

        let Some(camera_3d) = camera_manager.camera_3d_entity() else {
            return;
        };

        let Ok(camera_transform) = transform_q.get(camera_3d) else {
            return;
        };

        let Ok((camera, camera_projection)) = camera_q.get(camera_3d) else {
            return;
        };

        self.shapes_recalc -= 1;

        self.compass_recalc(camera_state, vertex_3d_q, &camera_transform);

        self.recalculate_hover();
        self.recalculate_selection();

        let camera_viewport = camera.viewport.unwrap();
        let view_matrix = camera_transform.view_matrix();
        let projection_matrix = camera_projection.projection_matrix(&camera_viewport);

        let camera_3d_scale = camera_state.camera_3d_scale();

        let vertex_2d_scale = Vertex2d::RADIUS * camera_3d_scale;
        let hover_vertex_2d_scale = Vertex2d::HOVER_RADIUS * camera_3d_scale;
        let compass_vertex_3d_scale = Compass::VERTEX_RADIUS / camera_3d_scale;
        let compass_vertex_2d_scale = Vertex2d::RADIUS;

        let edge_2d_scale = Edge2dLocal::NORMAL_THICKNESS * camera_3d_scale;
        let hover_edge_2d_scale = Edge2dLocal::HOVER_THICKNESS * camera_3d_scale;
        let compass_edge_3d_scale = Compass::EDGE_THICKNESS / camera_3d_scale;
        let compass_edge_2d_scale = Edge2dLocal::NORMAL_THICKNESS;

        let face_2d_scale = FaceIcon2d::SIZE * camera_3d_scale;
        let hover_face_2d_scale = FaceIcon2d::HOVER_SIZE * camera_3d_scale;

        // update vertices
        for (vertex_3d_entity, vertex_3d) in vertex_3d_q.iter() {
            // check if vertex is owned by the current tab
            if !Self::is_owned_by_tab_or_unowned(
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

        // update 2d edges
        for (edge_2d_entity, edge_endpoints) in edge_2d_q.iter() {
            let Some(end_3d_entity) = self.vertex_entity_2d_to_3d(&edge_endpoints.end) else {
                warn!("Edge entity {:?} has no 3d endpoint entity", edge_2d_entity);
                continue;
            };

            // check if vertex is owned by the current tab
            if !Self::is_owned_by_tab_or_unowned(current_tab_file_entity, owned_by_q, end_3d_entity)
            {
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

            let Ok(mut edge_2d_transform) = transform_q.get_mut(edge_2d_entity) else {
                warn!("2d Edge entity {:?} has no transform", edge_2d_entity);
                continue;
            };

            set_2d_line_transform(&mut edge_2d_transform, start_pos, end_pos);

            if compass_q.get(edge_2d_entity).is_ok() {
                edge_2d_transform.scale.y = compass_edge_2d_scale;
            } else {
                edge_2d_transform.scale.y = edge_2d_scale;
            }
        }

        // update 3d edges
        for (edge_entity, edge_endpoints, edge_angle_opt) in edge_3d_q.iter() {
            // check if vertex is owned by the current tab
            if !Self::is_owned_by_tab_or_unowned(current_tab_file_entity, owned_by_q, edge_entity) {
                continue;
            }

            let edge_angle_opt = edge_angle_opt.map(|angle| angle.get());

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
        }

        // update 2d faces
        for (face_2d_entity, face_icon) in face_2d_q.iter() {
            // check if face is owned by the current tab
            if !Self::is_owned_by_tab_or_unowned(
                current_tab_file_entity,
                owned_by_q,
                face_2d_entity,
            ) {
                continue;
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

        // update hover circle / triangle
        if let Some((hover_entity, shape)) = self.hovered_entity {
            if self.hovered_entity != self.selected_shape {
                match shape {
                    CanvasShape::RootVertex | CanvasShape::Vertex => {
                        let mut hover_vert_transform = transform_q.get_mut(hover_entity).unwrap();
                        hover_vert_transform.scale.x = hover_vertex_2d_scale;
                        hover_vert_transform.scale.y = hover_vertex_2d_scale;
                    }
                    CanvasShape::Edge => {
                        let mut hover_edge_transform = transform_q.get_mut(hover_entity).unwrap();
                        hover_edge_transform.scale.y = hover_edge_2d_scale;
                    }
                    CanvasShape::Face => {
                        let mut hover_face_transform = transform_q.get_mut(hover_entity).unwrap();
                        hover_face_transform.scale.x = hover_face_2d_scale;
                        hover_face_transform.scale.y = hover_face_2d_scale;
                    }
                }
            }
        }
    }

    pub fn recalculate_shapes(&mut self) {
        self.shapes_recalc = 2;
    }

    pub fn recalculate_hover(&mut self) {
        self.hover_recalc = true;
    }

    pub fn recalculate_selection(&mut self) {
        self.selection_recalc = true;
    }

    pub fn select_shape(&mut self, entity: &Entity, shape: CanvasShape) {
        self.selected_shape = Some((*entity, shape));
        self.recalculate_shapes();
    }

    pub fn deselect_shape(&mut self) {
        self.selected_shape = None;
        self.recalculate_shapes();
    }

    pub fn selected_shape_2d(&self) -> Option<(Entity, CanvasShape)> {
        self.selected_shape
    }

    pub fn register_3d_vertex(&mut self, entity_3d: Entity, entity_2d: Entity) {
        self.vertices_3d
            .insert(entity_3d, Vertex3dData::new(entity_2d));
        self.vertices_2d.insert(entity_2d, entity_3d);
    }

    pub fn register_3d_edge(
        &mut self,
        edge_3d_entity: Entity,
        edge_2d_entity: Entity,
        vertex_a_3d_entity: Entity,
        vertex_b_3d_entity: Entity,
        ownership_opt: Option<Entity>,
        angle_entity_opt: Option<Entity>,
    ) {
        for vertex_3d_entity in [vertex_a_3d_entity, vertex_b_3d_entity] {
            let Some(vertex_3d_data) = self.vertices_3d.get_mut(&vertex_3d_entity) else {
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
            Edge3dData::new(edge_2d_entity, vertex_a_3d_entity, vertex_b_3d_entity, angle_entity_opt),
        );
        self.edges_2d.insert(edge_2d_entity, edge_3d_entity);

        if let Some(file_entity) = ownership_opt {
            self.check_for_new_faces(vertex_a_3d_entity, vertex_b_3d_entity, file_entity);
        }
    }

    pub fn register_3d_face(&mut self, entity_3d: Entity, face_key: &FaceKey) {
        self.faces_3d.insert(entity_3d, *face_key);

        let Some(Some(face_3d_data)) = self.face_keys.get_mut(face_key) else {
            panic!("Face3d key: `{:?}` has not been registered", face_key);
        };
        face_3d_data.entity_3d = Some(entity_3d);
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

    // returns 2d edge entity
    fn unregister_3d_edge(&mut self, edge_3d_entity: &Entity) -> Option<Entity> {
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
                if let Some(vertex_3d_data) = self.vertices_3d.get_mut(&vertex_3d_entity) {
                    vertex_3d_data.remove_edge(edge_3d_entity);
                }
            }

            return Some(edge_2d_entity);
        }
        return None;
    }

    // returns 2d face entity
    fn unregister_face_key(&mut self, face_key: &FaceKey) -> Option<Entity> {
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
                if let Some(vertex_3d_data) = self.vertices_3d.get_mut(&vertex_3d_entity) {
                    vertex_3d_data.remove_face(face_key);
                }
            }

            // remove face from edges
            for edge_3d_entity in [
                face_3d_data.edge_3d_a,
                face_3d_data.edge_3d_b,
                face_3d_data.edge_3d_c,
            ] {
                if let Some(edge_3d_data) = self.edges_3d.get_mut(&edge_3d_entity) {
                    edge_3d_data.remove_face(face_key);
                }
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

    fn check_for_new_faces(
        &mut self,
        vertex_a_3d_entity: Entity,
        vertex_b_3d_entity: Entity,
        file_entity: Entity,
    ) {
        let vertex_a_connected_vertices = self.get_connected_vertices(vertex_a_3d_entity);
        let vertex_b_connected_vertices = self.get_connected_vertices(vertex_b_3d_entity);

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

    fn get_connected_vertices(&self, vertex_3d_entity: Entity) -> HashSet<Entity> {
        let mut set = HashSet::new();

        let Some(vertex_data) = self.vertices_3d.get(&vertex_3d_entity) else {
            panic!("Vertex3d entity: `{:?}` has not been registered", vertex_3d_entity);
        };
        let edges = &vertex_data.edges_3d;
        for edge_entity in edges {
            let edge_data = self.edges_3d.get(edge_entity).unwrap();
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

    pub fn process_new_faces(
        &mut self,
        commands: &mut Commands,
        camera_manager: &CameraManager,
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
                meshes,
                materials,
                file_entity,
                &face_key,
            );
        }

        self.recalculate_shapes();
    }

    pub fn remove_new_face_key(&mut self, face_key: &FaceKey) {
        self.new_face_keys.retain(|(key, _)| key != face_key);
    }

    // return face 2d entity
    pub fn process_new_face(
        &mut self,
        commands: &mut Commands,
        camera_manager: &CameraManager,
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
        let vertex_2d_a = self.vertex_entity_3d_to_2d(&vertex_3d_a).unwrap();
        let vertex_2d_b = self.vertex_entity_3d_to_2d(&vertex_3d_b).unwrap();
        let vertex_2d_c = self.vertex_entity_3d_to_2d(&vertex_3d_c).unwrap();

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
            let vertex_3d_data = self.vertices_3d.get_mut(vertex_3d_entity).unwrap();
            vertex_3d_data.add_face(*face_key);
        }

        // add face to edge data
        let mut edge_entities = Vec::new();
        for (vert_a, vert_b) in [
            (&vertex_3d_a, &vertex_3d_b),
            (&vertex_3d_b, &vertex_3d_c),
            (&vertex_3d_c, &vertex_3d_a),
        ] {
            // find edge in common
            let vertex_a_edges = &self.vertices_3d.get(vert_a).unwrap().edges_3d;
            let vertex_b_edges = &self.vertices_3d.get(vert_b).unwrap().edges_3d;
            let intersection = vertex_a_edges.intersection(vertex_b_edges);
            let mut found_edge = false;
            for edge_entity in intersection {
                if found_edge {
                    panic!("should only be one edge between any two vertices!");
                }
                found_edge = true;

                let Some(edge_3d_data) = self.edges_3d.get_mut(edge_entity) else {
                    panic!("Edge3d entity: `{:?}` has not been registered", edge_entity);
                };
                edge_3d_data.add_face(*face_key);

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

    pub fn create_networked_face_outer(&mut self, world: &mut World, face_2d_entity: Entity) {
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

        self.create_networked_face_inner(
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

    pub fn create_networked_face_inner(
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
            .insert(Face3dLocal);

        self.register_3d_face(face_3d_entity, face_key);

        // change 2d icon to use non-hollow triangle
        let face_2d_entity = self.face_2d_entity_from_face_key(&face_key).unwrap();
        commands
            .entity(face_2d_entity)
            .insert(meshes.add(Triangle::new_2d_equilateral()));
    }

    pub fn on_vertex_3d_moved(
        &self,
        client: &Client,
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
            let Some(Some(face_3d_data)) = self.face_keys.get(face_3d_key) else {
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

        if self.hovered_entity == Some((vertex_2d_entity, CanvasShape::Vertex)) {
            self.hovered_entity = None;
        }

        self.recalculate_shapes();

        vertex_2d_entity
    }

    // returns (deleted edge entity 2d, Vec<(deleted face entity 2d, deleted face entity 3d)>
    pub fn cleanup_deleted_edge(
        &mut self,
        commands: &mut Commands,
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
                let face_2d_entity = self.cleanup_deleted_face_key(commands, &face_3d_key);
                deleted_face_2d_entities.push(face_2d_entity);
            }
        }

        // unregister edge
        let Some(edge_2d_entity) = self.unregister_3d_edge(entity_3d) else {
            panic!(
                "Edge3d entity: `{:?}` has no corresponding Edge2d entity",
                entity_3d
            );
        };

        // despawn 2d edge
        info!("despawn 2d edge {:?}", edge_2d_entity);
        commands.entity(edge_2d_entity).despawn();

        if self.hovered_entity == Some((edge_2d_entity, CanvasShape::Edge)) {
            self.hovered_entity = None;
        }

        self.recalculate_shapes();

        (edge_2d_entity, deleted_face_2d_entities)
    }

    // returns face 2d entity
    pub(crate) fn cleanup_deleted_face_key(
        &mut self,
        commands: &mut Commands,
        face_key: &FaceKey,
    ) -> Entity {
        // unregister face
        let Some(face_2d_entity) = self.unregister_face_key(face_key) else {
            panic!(
                "FaceKey: `{:?}` has no corresponding Face2d entity",
                face_key
            );
        };

        // despawn 2d face
        info!("despawn 2d face {:?}", face_2d_entity);
        commands.entity(face_2d_entity).despawn();

        if self.hovered_entity == Some((face_2d_entity, CanvasShape::Face)) {
            self.hovered_entity = None;
        }

        self.recalculate_shapes();

        face_2d_entity
    }

    // returns 2d face entity
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

    pub(crate) fn has_vertex_entity_3d(&self, entity_3d: &Entity) -> bool {
        self.vertices_3d.contains_key(entity_3d)
    }

    pub(crate) fn has_edge_entity_3d(&self, entity_3d: &Entity) -> bool {
        self.edges_3d.contains_key(entity_3d)
    }

    pub(crate) fn has_2d_face(&self, face_key: &FaceKey) -> bool {
        if let Some(Some(_)) = self.face_keys.get(face_key) {
            return true;
        }
        return false;
    }

    pub(crate) fn has_shape_entity_3d(&self, entity_3d: &Entity) -> bool {
        self.faces_3d.contains_key(entity_3d)
            || self.edges_3d.contains_key(entity_3d)
            || self.vertices_3d.contains_key(entity_3d)
    }

    pub(crate) fn shape_entity_2d_to_3d(
        &self,
        entity_2d: &Entity,
        shape_type: CanvasShape,
    ) -> Option<Entity> {
        match shape_type {
            CanvasShape::RootVertex | CanvasShape::Vertex => self.vertex_entity_2d_to_3d(entity_2d),
            CanvasShape::Edge => {
                let output = self.edge_entity_2d_to_3d(entity_2d);
                info!("edge entity 2d `{:?}` to 3d: `{:?}`", entity_2d, output);
                output
            }
            CanvasShape::Face => self.face_entity_2d_to_3d(entity_2d),
        }
    }

    fn shape_type_from_3d_entity(&self, entity_3d: &Entity) -> Option<CanvasShape> {
        if self.vertices_3d.contains_key(entity_3d) {
            Some(CanvasShape::Vertex)
        } else if self.edges_3d.contains_key(entity_3d) {
            Some(CanvasShape::Edge)
        } else if self.faces_3d.contains_key(entity_3d) {
            Some(CanvasShape::Face)
        } else {
            None
        }
    }

    pub(crate) fn shape_entity_3d_to_2d(&self, entity_3d: &Entity) -> Option<Entity> {
        let shape_type = self.shape_type_from_3d_entity(entity_3d).unwrap();

        match shape_type {
            CanvasShape::RootVertex | CanvasShape::Vertex => self.vertex_entity_3d_to_2d(entity_3d),
            CanvasShape::Edge => self.edge_entity_3d_to_2d(entity_3d),
            CanvasShape::Face => self.face_entity_3d_to_2d(entity_3d),
        }
    }

    pub(crate) fn vertex_entity_3d_to_2d(&self, entity_3d: &Entity) -> Option<Entity> {
        self.vertices_3d.get(entity_3d).map(|data| data.entity_2d)
    }

    pub(crate) fn vertex_entity_2d_to_3d(&self, entity_2d: &Entity) -> Option<Entity> {
        self.vertices_2d.get(entity_2d).copied()
    }

    pub(crate) fn edge_entity_2d_to_3d(&self, entity_2d: &Entity) -> Option<Entity> {
        self.edges_2d.get(entity_2d).copied()
    }

    pub(crate) fn edge_entity_3d_to_2d(&self, entity_2d: &Entity) -> Option<Entity> {
        self.edges_3d.get(entity_2d).map(|data| data.entity_2d)
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

    pub(crate) fn edge_connected_faces(&self, edge_3d_entity: &Entity) -> Option<Vec<FaceKey>> {
        self.edges_3d
            .get(edge_3d_entity)
            .map(|data| data.faces_3d.iter().copied().collect())
    }

    fn face_key_from_2d_entity(&self, entity_2d: &Entity) -> Option<FaceKey> {
        self.faces_2d.get(entity_2d).copied()
    }

    pub(crate) fn face_2d_entity_from_face_key(&self, face_key: &FaceKey) -> Option<Entity> {
        let Some(Some(face_3d_data)) = self.face_keys.get(face_key) else {
            return None;
        };
        Some(face_3d_data.entity_2d)
    }

    pub(crate) fn face_3d_entity_from_face_key(&self, face_key: &FaceKey) -> Option<Entity> {
        let Some(Some(face_3d_data)) = self.face_keys.get(face_key) else {
            return None;
        };
        face_3d_data.entity_3d
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

    pub fn edge_3d_postprocess(
        &mut self,
        commands: &mut Commands,
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
        camera_manager: &CameraManager,
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
            create_3d_edge_diamond(meshes, materials, Vec3::ZERO, Vec3::X, color, Edge3dLocal::NORMAL_THICKNESS)
        } else {
            create_3d_edge_line(meshes, materials, Vec3::ZERO, Vec3::X, color, Edge3dLocal::NORMAL_THICKNESS)
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
                    color,
                    Edge2dLocal::NORMAL_THICKNESS,
                )
            } else {
                create_2d_edge_line(
                    meshes,
                    materials,
                    Vec2::ZERO,
                    Vec2::X,
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
        let edge_angle_entity_opt = if let Some(edge_angle) = edge_angle_opt {
            let shape_components = create_2d_edge_arrow(
                meshes,
                materials,
                Vec2::ZERO,
                Vec2::X,
                color,
                Edge2dLocal::NORMAL_THICKNESS,
            );
            let edge_angle_entity = commands.spawn_empty()
                .insert(shape_components)
                .insert(camera_manager.layer_2d)
                .insert(EdgeAngleLocal::new(edge_angle))
                .id();
            if let Some(file_entity) = ownership_opt {
                commands
                    .entity(edge_angle_entity)
                    .insert(OwnedByFileLocal::new(file_entity));
            }
            Some(edge_angle_entity)
        } else {
            None
        };

        // register 3d & 2d edges together
        self.register_3d_edge(
            edge_3d_entity,
            edge_2d_entity,
            vertex_a_3d_entity,
            vertex_b_3d_entity,
            ownership_opt,
            edge_angle_entity_opt,
        );

        edge_2d_entity
    }

    pub fn set_current_file_type(&mut self, file_type: FileTypeValue) {
        self.current_file_type = file_type;
    }

    fn handle_insert_key_press(&mut self, action_stack: &mut ActionStack<ShapeAction>) {
        if self.selected_shape.is_some() {
            return;
        }

        if self.current_file_type != FileTypeValue::Mesh {
            return;
        }

        action_stack.buffer_action(ShapeAction::CreateVertex(
            VertexTypeData::Mesh(Vec::new(), Vec::new()),
            Vec3::ZERO,
            None,
        ));
    }

    fn handle_delete_key_press(
        &mut self,
        commands: &mut Commands,
        client: &mut Client,
        action_stack: &mut ActionStack<ShapeAction>,
    ) {
        match self.selected_shape {
            Some((vertex_2d_entity, CanvasShape::Vertex)) => {
                // delete vertex
                let vertex_3d_entity = self.vertex_entity_2d_to_3d(&vertex_2d_entity).unwrap();

                // check whether we can delete vertex
                let auth_status = commands.entity(vertex_3d_entity).authority(client).unwrap();
                if !auth_status.is_granted() && !auth_status.is_available() {
                    // do nothing, vertex is not available
                    // TODO: queue for deletion? check before this?
                    warn!(
                        "Vertex {:?} is not available for deletion!",
                        vertex_3d_entity
                    );
                    return;
                }

                let auth_status = commands.entity(vertex_3d_entity).authority(client).unwrap();
                if !auth_status.is_granted() {
                    // request authority if needed
                    commands.entity(vertex_3d_entity).request_authority(client);
                }

                action_stack.buffer_action(ShapeAction::DeleteVertex(vertex_2d_entity, None));

                self.selected_shape = None;
            }
            Some((edge_2d_entity, CanvasShape::Edge)) => {
                if self.current_file_type == FileTypeValue::Skel {
                    return;
                }
                // delete edge
                let edge_3d_entity = self.edge_entity_2d_to_3d(&edge_2d_entity).unwrap();

                // check whether we can delete edge
                let auth_status = commands.entity(edge_3d_entity).authority(client).unwrap();
                if !auth_status.is_granted() && !auth_status.is_available() {
                    // do nothing, edge is not available
                    // TODO: queue for deletion? check before this?
                    warn!("Edge {:?} is not available for deletion!", edge_3d_entity);
                    return;
                }

                let auth_status = commands.entity(edge_3d_entity).authority(client).unwrap();
                if !auth_status.is_granted() {
                    // request authority if needed
                    commands.entity(edge_3d_entity).request_authority(client);
                }

                action_stack.buffer_action(ShapeAction::DeleteEdge(edge_2d_entity, None));

                self.selected_shape = None;
            }
            Some((face_2d_entity, CanvasShape::Face)) => {
                let face_key = self.face_key_from_2d_entity(&face_2d_entity).unwrap();
                let face_3d_entity = self.face_3d_entity_from_face_key(&face_key).unwrap();

                // check whether we can delete edge
                let auth_status = commands.entity(face_3d_entity).authority(client).unwrap();
                if !auth_status.is_granted() && !auth_status.is_available() {
                    // do nothing, face is not available
                    // TODO: queue for deletion? check before this?
                    warn!("Face {:?} is not available for deletion!", face_key);
                    return;
                }

                let auth_status = commands.entity(face_3d_entity).authority(client).unwrap();
                if !auth_status.is_granted() {
                    // request authority if needed
                    commands.entity(face_3d_entity).request_authority(client);
                }

                action_stack.buffer_action(ShapeAction::DeleteFace(face_2d_entity));

                self.selected_shape = None;
            }
            _ => {}
        }
    }

    pub(crate) fn update_mouse_hover(
        &mut self,
        current_tab_file_entity: Entity,
        mouse_position: &Vec2,
        camera_state: &CameraState,
        transform_q: &mut Query<(&mut Transform, Option<&Compass>)>,
        owned_by_q: &Query<&OwnedByFileLocal>,
        vertex_2d_q: &Query<(Entity, Option<&VertexRoot>), (With<Vertex2d>, Without<Compass>)>,
        edge_2d_q: &Query<(Entity, &Edge2dLocal), Without<Compass>>,
        face_2d_q: &Query<(Entity, &FaceIcon2d)>,
    ) {
        if !self.hover_recalc {
            return;
        }
        self.hover_recalc = false;

        let camera_3d_scale = camera_state.camera_3d_scale();

        let mut least_distance = f32::MAX;
        let mut least_entity = None;

        // check for vertices
        for (vertex_entity, root_opt) in vertex_2d_q.iter() {
            // check tab ownership, skip vertices from other tabs
            if !Self::is_owned_by_tab(current_tab_file_entity, owned_by_q, vertex_entity) {
                continue;
            }

            let (vertex_transform, _) = transform_q.get(vertex_entity).unwrap();
            let vertex_position = vertex_transform.translation.truncate();
            let distance = vertex_position.distance(*mouse_position);
            if distance < least_distance {
                least_distance = distance;

                let shape = match root_opt {
                    Some(_) => CanvasShape::RootVertex,
                    None => CanvasShape::Vertex,
                };

                least_entity = Some((vertex_entity, shape));
            }
        }

        let mut is_hovering = least_distance <= (Vertex2d::DETECT_RADIUS * camera_3d_scale);

        // check for edges
        if !is_hovering {
            for (edge_entity, _) in edge_2d_q.iter() {
                // check tab ownership, skip edges from other tabs
                if !Self::is_owned_by_tab(current_tab_file_entity, owned_by_q, edge_entity) {
                    continue;
                }

                let (edge_transform, _) = transform_q.get(edge_entity).unwrap();
                let edge_start = edge_transform.translation.truncate();
                let edge_end = get_2d_line_transform_endpoint(&edge_transform);

                let distance = distance_to_2d_line(*mouse_position, edge_start, edge_end);
                if distance < least_distance {
                    least_distance = distance;
                    least_entity = Some((edge_entity, CanvasShape::Edge));
                }
            }

            is_hovering = least_distance <= (Edge2dLocal::DETECT_THICKNESS * camera_3d_scale);
        }

        // check for faces
        if !is_hovering {
            for (face_entity, _) in face_2d_q.iter() {
                // check tab ownership, skip faces from other tabs
                if !Self::is_owned_by_tab(current_tab_file_entity, owned_by_q, face_entity) {
                    continue;
                }

                let (face_transform, _) = transform_q.get(face_entity).unwrap();
                let face_position = face_transform.translation.truncate();
                let distance = face_position.distance(*mouse_position);
                if distance < least_distance {
                    least_distance = distance;

                    least_entity = Some((face_entity, CanvasShape::Face));
                }
            }

            is_hovering = least_distance <= (FaceIcon2d::DETECT_RADIUS * camera_3d_scale);
        }

        // define old and new hovered states
        let old_hovered_entity = self.hovered_entity;
        let next_hovered_entity = if is_hovering { least_entity } else { None };

        // hover state did not change
        if old_hovered_entity == next_hovered_entity {
            return;
        }

        // apply
        self.hovered_entity = next_hovered_entity;
        self.shapes_recalc = 1;
    }

    pub(crate) fn update_select_line(
        &mut self,
        mouse_position: &Vec2,
        canvas: &Canvas,
        camera_state: &CameraState,
        transform_q: &mut Query<&mut Transform>,
        visibility_q: &mut Query<&mut Visibility>,
    ) {
        if !self.selection_recalc {
            return;
        }
        self.selection_recalc = false;

        let camera_3d_scale = camera_state.camera_3d_scale();

        // update selected vertex line
        let select_line_entity = self.select_line_entity.unwrap();
        let select_circle_entity = self.select_circle_entity.unwrap();
        let select_triangle_entity = self.select_triangle_entity.unwrap();

        //

        // update selected vertex circle & line
        let Ok(mut select_shape_visibilities) = visibility_q.get_many_mut([select_circle_entity, select_triangle_entity, select_line_entity]) else {
            panic!("Select shape entities has no Visibility");
        };

        match self.selected_shape {
            Some((selected_vertex_entity, CanvasShape::Vertex))
            | Some((selected_vertex_entity, CanvasShape::RootVertex)) => {
                let vertex_transform = {
                    let Ok(vertex_transform) = transform_q.get(selected_vertex_entity) else {
                        return;
                    };
                    *vertex_transform
                };

                // sync select line transform
                {
                    let Ok(mut select_line_transform) = transform_q.get_mut(select_line_entity) else {
                        panic!("Select line entity has no Transform");
                    };

                    set_2d_line_transform(
                        &mut select_line_transform,
                        vertex_transform.translation.truncate(),
                        *mouse_position,
                    );
                    select_line_transform.scale.y = camera_3d_scale;
                }

                // sync select circle transform
                {
                    let Ok(mut select_circle_transform) = transform_q.get_mut(select_circle_entity) else {
                        panic!("Select shape entities has no Transform");
                    };

                    select_circle_transform.translation = vertex_transform.translation;
                    select_circle_transform.scale =
                        Vec3::splat(SelectCircle::RADIUS * camera_3d_scale);
                }

                select_shape_visibilities[0].visible = true; // select circle is visible
                select_shape_visibilities[1].visible = false; // no select triangle visible
                select_shape_visibilities[2].visible = canvas.has_focus(); // select line is visible
            }
            Some((selected_edge_entity, CanvasShape::Edge)) => {
                let selected_edge_transform = {
                    let Ok(selected_edge_transform) = transform_q.get(selected_edge_entity) else {
                        return;
                    };
                    *selected_edge_transform
                };

                // sync select line transform
                {
                    let Ok(mut select_line_transform) = transform_q.get_mut(select_line_entity) else {
                        panic!("Select line entity has no Transform");
                    };

                    select_line_transform.mirror(&selected_edge_transform);

                    select_line_transform.scale.y = 3.0 * camera_3d_scale;
                    select_line_transform.translation.z += 1.0;
                }

                select_shape_visibilities[0].visible = false; // no select circle visible
                select_shape_visibilities[1].visible = false; // no select triangle visible
                select_shape_visibilities[2].visible = true; // select line is visible
            }
            Some((selected_face_entity, CanvasShape::Face)) => {
                let face_icon_transform = {
                    let Ok(face_icon_transform) = transform_q.get(selected_face_entity) else {
                        return;
                    };
                    *face_icon_transform
                };

                // sync select triangle transform
                {
                    let Ok(mut select_triangle_transform) = transform_q.get_mut(select_triangle_entity) else {
                        panic!("Select shape entities has no Transform");
                    };

                    select_triangle_transform.translation = face_icon_transform.translation;
                    select_triangle_transform.scale =
                        Vec3::splat(SelectTriangle::SIZE * camera_3d_scale);
                }

                select_shape_visibilities[0].visible = false; // select circle is not visible
                select_shape_visibilities[1].visible = true; // select triangle is visible
                select_shape_visibilities[2].visible = false; // select line is not visible
            }
            None => {
                select_shape_visibilities[0].visible = false; // no select circle visible
                select_shape_visibilities[1].visible = false; // no select triangle visible
                select_shape_visibilities[2].visible = false; // no select line visible
            }
        }
    }

    fn handle_mouse_click(
        &mut self,
        camera_manager: &CameraManager,
        action_stack: &mut ActionStack<ShapeAction>,
        click_type: MouseButton,
        mouse_position: &Vec2,
        camera_q: &Query<(&mut Camera, &mut Projection)>,
        transform_q: &Query<&mut Transform>,
    ) {
        let cursor_is_hovering = self.hovered_entity.is_some();
        let shape_is_selected = self.selected_shape.is_some();

        if shape_is_selected {
            match click_type {
                MouseButton::Left => {
                    match self.selected_shape.unwrap() {
                        (_, CanvasShape::Edge) | (_, CanvasShape::Face) => {
                            // should not ever be able to attach something to an edge or face?
                            // select hovered entity
                            action_stack
                                .buffer_action(ShapeAction::SelectShape(self.hovered_entity));
                            return;
                        }
                        _ => {}
                    }

                    if cursor_is_hovering {
                        if self.current_file_type == FileTypeValue::Skel {
                            // skel file type does nothing when trying to connect vertices together
                            // needs to always be a new vertex
                            // select hovered entity
                            action_stack
                                .buffer_action(ShapeAction::SelectShape(self.hovered_entity));
                            return;
                        } else {
                            match self.hovered_entity.unwrap() {
                                (_, CanvasShape::Edge) | (_, CanvasShape::Face) => {
                                    // should not ever be able to attach something to an edge or face?
                                    // select hovered entity
                                    action_stack
                                        .buffer_action(ShapeAction::SelectShape(self.hovered_entity));
                                    return;
                                }
                                _ => {}
                            }
                        }

                        // at this point, filetype is Mesh, and we are trying to connect vertices together

                        // link vertices together
                        let (vertex_2d_entity_a, _) = self.selected_shape.unwrap();
                        let (vertex_2d_entity_b, _) = self.hovered_entity.unwrap();
                        if vertex_2d_entity_a == vertex_2d_entity_b {
                            return;
                        }

                        // check if edge already exists
                        if self
                            .edge_2d_entity_from_vertices(vertex_2d_entity_a, vertex_2d_entity_b)
                            .is_some()
                        {
                            // select edge
                            action_stack.buffer_action(ShapeAction::SelectShape(Some((
                                vertex_2d_entity_b,
                                CanvasShape::Vertex,
                            ))));
                            return;
                        } else {
                            // create edge
                            action_stack.buffer_action(ShapeAction::CreateEdge(
                                vertex_2d_entity_a,
                                vertex_2d_entity_b,
                                (vertex_2d_entity_b, CanvasShape::Vertex),
                                None,
                                None,
                            ));
                            return;
                        }
                    } else {
                        // create new vertex

                        // get camera
                        let camera_3d = camera_manager.camera_3d_entity().unwrap();
                        let camera_transform: Transform = *transform_q.get(camera_3d).unwrap();
                        let (camera, camera_projection) = camera_q.get(camera_3d).unwrap();

                        let camera_viewport = camera.viewport.unwrap();
                        let view_matrix = camera_transform.view_matrix();
                        let projection_matrix =
                            camera_projection.projection_matrix(&camera_viewport);

                        // get 2d vertex transform
                        let (vertex_2d_entity, _) = self.selected_shape.unwrap();
                        if let Ok(vertex_2d_transform) = transform_q.get(vertex_2d_entity) {
                            // convert 2d to 3d
                            let new_3d_position = convert_2d_to_3d(
                                &view_matrix,
                                &projection_matrix,
                                &camera_viewport.size_vec2(),
                                &mouse_position,
                                vertex_2d_transform.translation.z,
                            );

                            // spawn new vertex
                            action_stack.buffer_action(ShapeAction::CreateVertex(
                                match self.current_file_type {
                                    FileTypeValue::Skel => {
                                        VertexTypeData::Skel(vertex_2d_entity, 0.0,None)
                                    }
                                    FileTypeValue::Mesh => VertexTypeData::Mesh(
                                        vec![(vertex_2d_entity, None)],
                                        Vec::new(),
                                    ),
                                    FileTypeValue::Anim => {
                                        panic!("");
                                    }
                                },
                                new_3d_position,
                                None,
                            ));
                        } else {
                            warn!(
                                "Selected vertex entity: {:?} has no Transform",
                                vertex_2d_entity
                            );
                        }
                    }
                }
                MouseButton::Right => {
                    // deselect vertex
                    action_stack.buffer_action(ShapeAction::SelectShape(None));
                }
                _ => {}
            }
        } else {
            if cursor_is_hovering {
                match (self.hovered_entity.map(|(_, s)| s).unwrap(), click_type) {
                    (CanvasShape::Vertex, MouseButton::Left)
                    | (CanvasShape::RootVertex, MouseButton::Left) => {
                        action_stack.buffer_action(ShapeAction::SelectShape(self.hovered_entity));
                    }
                    (CanvasShape::Edge, MouseButton::Left) => {
                        action_stack
                            .buffer_action(ShapeAction::SelectShape(self.hovered_entity));
                    }
                    (CanvasShape::Face, MouseButton::Left) => {
                        if self.current_file_type == FileTypeValue::Mesh {
                            action_stack
                                .buffer_action(ShapeAction::SelectShape(self.hovered_entity));
                        } else {
                            panic!("shouldn't be possible");
                        }
                    }
                    (CanvasShape::Vertex, MouseButton::Right)
                    | (CanvasShape::RootVertex, MouseButton::Right) => {
                        // do nothing, vertex deselection happens above
                    }
                    (CanvasShape::Edge, MouseButton::Right) => {
                        // TODO: delete edge?
                    }
                    (CanvasShape::Face, MouseButton::Right) => {
                        // TODO: delete face?
                    }
                    _ => {}
                }
            }
        }
    }

    fn handle_mouse_drag(
        &mut self,
        commands: &mut Commands,
        client: &Client,
        camera_manager: &mut CameraManager,
        camera_state: &mut CameraState,
        click_type: MouseButton,
        mouse_position: Vec2,
        delta: Vec2,
        camera_q: &Query<(&mut Camera, &mut Projection)>,
        transform_q: &Query<&mut Transform>,
        vertex_3d_q: &mut Query<&mut Vertex3d>,
    ) {
        let vertex_is_selected = self.selected_shape.is_some();
        let shape_can_drag =
            vertex_is_selected && self.selected_shape.unwrap().1 == CanvasShape::Vertex;

        if vertex_is_selected && shape_can_drag {
            match click_type {
                MouseButton::Left => {
                    // move vertex
                    let (vertex_2d_entity, _) = self.selected_shape.unwrap();

                    if let Some(vertex_3d_entity) = self.vertex_entity_2d_to_3d(&vertex_2d_entity) {
                        let auth_status =
                            commands.entity(vertex_3d_entity).authority(client).unwrap();
                        if !(auth_status.is_requested() || auth_status.is_granted()) {
                            // only continue to mutate if requested or granted authority over vertex
                            info!("No authority over vertex, skipping..");
                            return;
                        }

                        // get camera
                        let camera_3d = camera_manager.camera_3d_entity().unwrap();
                        let camera_transform: Transform = *transform_q.get(camera_3d).unwrap();
                        let (camera, camera_projection) = camera_q.get(camera_3d).unwrap();

                        let camera_viewport = camera.viewport.unwrap();
                        let view_matrix = camera_transform.view_matrix();
                        let projection_matrix =
                            camera_projection.projection_matrix(&camera_viewport);

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

                        // set networked 3d vertex position
                        let mut vertex_3d = vertex_3d_q.get_mut(vertex_3d_entity).unwrap();

                        if let Some((_, old_3d_position, _)) = self.last_vertex_dragged {
                            self.last_vertex_dragged =
                                Some((vertex_2d_entity, old_3d_position, new_3d_position));
                        } else {
                            let old_3d_position = vertex_3d.as_vec3();
                            self.last_vertex_dragged =
                                Some((vertex_2d_entity, old_3d_position, new_3d_position));
                        }

                        vertex_3d.set_x(new_3d_position.x as i16);
                        vertex_3d.set_y(new_3d_position.y as i16);
                        vertex_3d.set_z(new_3d_position.z as i16);

                        // redraw
                        self.recalculate_shapes();
                    } else {
                        warn!(
                            "Selected vertex entity: {:?} has no 3d counterpart",
                            vertex_2d_entity
                        );
                    }
                }
                MouseButton::Right => {
                    // TODO: dunno if this is possible? shouldn't the vertex be deselected?
                }
                _ => {}
            }
        } else {
            match click_type {
                MouseButton::Left => {
                    camera_manager.camera_pan(camera_state, delta);
                }
                MouseButton::Right => {
                    camera_manager.camera_orbit(camera_state, delta);
                }
                _ => {}
            }
        }
    }

    pub(crate) fn setup_compass(
        &mut self,
        commands: &mut Commands,
        camera_manager: &mut CameraManager,
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
    ) {
        let (root_vertex_2d_entity, vertex_3d_entity, _, _) = self.new_local_vertex(
            commands,
            camera_manager,
            meshes,
            materials,
            None,
            Vec3::ZERO,
            Color::WHITE,
        );
        self.compass_vertices.push(vertex_3d_entity);
        commands.entity(root_vertex_2d_entity).insert(Compass);
        commands.entity(vertex_3d_entity).insert(Compass);

        self.new_compass_arm(
            commands,
            camera_manager,
            meshes,
            materials,
            root_vertex_2d_entity,
            Vec3::new(100.0, 0.0, 0.0),
            Color::RED,
        );

        self.new_compass_arm(
            commands,
            camera_manager,
            meshes,
            materials,
            root_vertex_2d_entity,
            Vec3::new(0.0, 100.0, 0.0),
            Color::GREEN,
        );

        self.new_compass_arm(
            commands,
            camera_manager,
            meshes,
            materials,
            root_vertex_2d_entity,
            Vec3::new(0.0, 0.0, 100.0),
            Color::LIGHT_BLUE,
        );
    }

    fn new_compass_arm(
        &mut self,
        commands: &mut Commands,
        camera_manager: &mut CameraManager,
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
        root_vertex_2d_entity: Entity,
        position: Vec3,
        color: Color,
    ) {
        let (vertex_2d_entity, vertex_3d_entity, Some(edge_2d_entity), Some(edge_3d_entity)) = self.new_local_vertex(
            commands,
            camera_manager,
            meshes,
            materials,
            Some(root_vertex_2d_entity),
            position,
            color,
        ) else {
            panic!("No edges?");
        };
        self.compass_vertices.push(vertex_3d_entity);
        commands.entity(vertex_2d_entity).insert(Compass);
        commands.entity(vertex_3d_entity).insert(Compass);
        commands.entity(edge_2d_entity).insert(Compass);
        commands.entity(edge_3d_entity).insert(Compass);
    }

    pub(crate) fn setup_grid(
        &mut self,
        commands: &mut Commands,
        camera_manager: &mut CameraManager,
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
    ) {
        self.new_grid_corner(
            commands,
            camera_manager,
            meshes,
            materials,
            true,
            true,
            true,
        );
        self.new_grid_corner(
            commands,
            camera_manager,
            meshes,
            materials,
            true,
            false,
            false,
        );

        self.new_grid_corner(
            commands,
            camera_manager,
            meshes,
            materials,
            false,
            true,
            false,
        );
        self.new_grid_corner(
            commands,
            camera_manager,
            meshes,
            materials,
            false,
            false,
            true,
        );
    }

    fn new_grid_corner(
        &mut self,
        commands: &mut Commands,
        camera_manager: &mut CameraManager,
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
        x: bool,
        y: bool,
        z: bool,
    ) {
        let xf = if x { 1.0 } else { -1.0 };
        let yf = if y { 1.0 } else { -1.0 };
        let zf = if z { 1.0 } else { -1.0 };

        let grid_size: f32 = 100.0;
        let neg_grid_size: f32 = -grid_size;

        let (root_vertex_2d_entity, root_vertex_3d_entity, _, _) = self.new_local_vertex(
            commands,
            camera_manager,
            meshes,
            materials,
            None,
            Vec3::new(grid_size * xf, (grid_size * yf) + grid_size, grid_size * zf),
            Color::DARK_GRAY,
        );
        commands.entity(root_vertex_2d_entity).insert(Compass);
        commands.entity(root_vertex_3d_entity).insert(Compass);

        self.new_grid_vertex(
            commands,
            camera_manager,
            meshes,
            materials,
            root_vertex_2d_entity,
            Vec3::new(
                neg_grid_size * xf,
                (grid_size * yf) + grid_size,
                grid_size * zf,
            ),
        );
        self.new_grid_vertex(
            commands,
            camera_manager,
            meshes,
            materials,
            root_vertex_2d_entity,
            Vec3::new(
                grid_size * xf,
                (neg_grid_size * yf) + grid_size,
                grid_size * zf,
            ),
        );
        self.new_grid_vertex(
            commands,
            camera_manager,
            meshes,
            materials,
            root_vertex_2d_entity,
            Vec3::new(
                grid_size * xf,
                (grid_size * yf) + grid_size,
                neg_grid_size * zf,
            ),
        );
    }

    fn new_grid_vertex(
        &mut self,
        commands: &mut Commands,
        camera_manager: &mut CameraManager,
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
        parent_vertex_2d_entity: Entity,
        position: Vec3,
    ) {
        let (vertex_2d_entity, vertex_3d_entity, Some(edge_2d_entity), Some(edge_3d_entity)) = self.new_local_vertex(
            commands,
            camera_manager,
            meshes,
            materials,
            Some(parent_vertex_2d_entity),
            position,
            Color::DARK_GRAY,
        ) else {
            panic!("No edges?");
        };
        commands.entity(vertex_2d_entity).insert(Compass);
        commands.entity(edge_2d_entity).insert(Compass);
        commands.entity(vertex_3d_entity).insert(Compass);
        commands.entity(edge_3d_entity).insert(Compass);
    }

    fn new_local_vertex(
        &mut self,
        commands: &mut Commands,
        camera_manager: &mut CameraManager,
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
            let new_edge_2d_entity = self.edge_3d_postprocess(
                commands,
                meshes,
                materials,
                camera_manager,
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

    fn compass_recalc(
        &self,
        camera_state: &CameraState,
        vertex_3d_q: &mut Query<(Entity, &mut Vertex3d)>,
        camera_transform: &Transform,
    ) {
        let Ok((_, mut vertex_3d)) = vertex_3d_q.get_mut(self.compass_vertices[0]) else {
            return;
        };

        let right = camera_transform.right_direction();
        let up = right.cross(camera_transform.view_direction());

        let unit_length = 1.0 / camera_state.camera_3d_scale();
        const COMPASS_POS: Vec2 = Vec2::new(530.0, 300.0);
        let offset_2d = camera_state.camera_3d_offset().round()
            + Vec2::new(
                unit_length * -1.0 * COMPASS_POS.x,
                unit_length * COMPASS_POS.y,
            );
        let offset_3d = (right * offset_2d.x) + (up * offset_2d.y);

        let vert_offset_3d = Vec3::ZERO + offset_3d;
        vertex_3d.set_vec3(&vert_offset_3d);

        let compass_length = unit_length * 25.0;
        let vert_offset_3d = Vec3::new(compass_length, 0.0, 0.0) + offset_3d;
        let (_, mut vertex_3d) = vertex_3d_q.get_mut(self.compass_vertices[1]).unwrap();
        vertex_3d.set_vec3(&vert_offset_3d);

        let vert_offset_3d = Vec3::new(0.0, compass_length, 0.0) + offset_3d;
        let (_, mut vertex_3d) = vertex_3d_q.get_mut(self.compass_vertices[2]).unwrap();
        vertex_3d.set_vec3(&vert_offset_3d);

        let vert_offset_3d = Vec3::new(0.0, 0.0, compass_length) + offset_3d;
        let (_, mut vertex_3d) = vertex_3d_q.get_mut(self.compass_vertices[3]).unwrap();
        vertex_3d.set_vec3(&vert_offset_3d);
    }

    // returns true if vertex is owned by tab or unowned
    fn is_owned_by_tab_or_unowned(
        current_tab_file_entity: Entity,
        owned_by_tab_q: &Query<&OwnedByFileLocal>,
        entity: Entity,
    ) -> bool {
        if let Ok(owned_by_tab) = owned_by_tab_q.get(entity) {
            if owned_by_tab.file_entity == current_tab_file_entity {
                return true;
            }
        } else {
            return true;
        }
        return false;
    }

    // returns true if vertex is owned by tab
    fn is_owned_by_tab(
        current_tab_file_entity: Entity,
        owned_by_tab_q: &Query<&OwnedByFileLocal>,
        entity: Entity,
    ) -> bool {
        if let Ok(owned_by_tab) = owned_by_tab_q.get(entity) {
            if owned_by_tab.file_entity == current_tab_file_entity {
                return true;
            }
        }
        return false;
    }
    fn edge_2d_entity_from_vertices(
        &self,
        vertex_2d_a: Entity,
        vertex_2d_b: Entity,
    ) -> Option<Entity> {
        let vertex_3d_a = self.vertex_entity_2d_to_3d(&vertex_2d_a)?;
        let vertex_3d_b = self.vertex_entity_2d_to_3d(&vertex_2d_b)?;
        let vertex_a_data = self.vertices_3d.get(&vertex_3d_a)?;
        let vertex_b_data = self.vertices_3d.get(&vertex_3d_b)?;
        let intersecting_edge_3d_entity = vertex_a_data
            .edges_3d
            .intersection(&vertex_b_data.edges_3d)
            .next()?;
        let edge_2d_entity = self.edge_entity_3d_to_2d(&intersecting_edge_3d_entity)?;
        Some(edge_2d_entity)
    }
}
