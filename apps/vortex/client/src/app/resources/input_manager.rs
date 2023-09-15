use std::f32::consts::FRAC_PI_2;

use bevy_ecs::{
    entity::Entity,
    query::{With, Without},
    system::{Commands, Query, Resource},
};
use bevy_log::{info, warn};

use naia_bevy_client::{Client, CommandsExt};

use input::{InputAction, Key, MouseButton};
use math::{convert_2d_to_3d, Vec2, Vec3};
use render_api::{
    components::{Camera, CameraProjection, Projection, Transform, Visibility},
    shapes::{
        angle_between, distance_to_2d_line, get_2d_line_transform_endpoint, normalize_angle,
        set_2d_line_transform,
    },
};

use vortex_proto::components::{EdgeAngle, FileExtension, Vertex3d, VertexRoot};

use crate::app::{
    components::{
        Edge2dLocal, FaceIcon2d, LocalShape, OwnedByFileLocal, SelectCircle, SelectTriangle,
        Vertex2d, VertexTypeData,
    },
    resources::{
        action::ActionStack, action::ShapeAction, animation_manager::AnimationManager,
        camera_manager::CameraAngle, camera_manager::CameraManager, camera_state::CameraState,
        canvas::Canvas, edge_manager::EdgeManager, face_manager::FaceManager,
        key_action_map::KeyActionMap, shape_data::CanvasShape, shape_manager::ShapeManager,
        tab_manager::TabState, vertex_manager::VertexManager,
    },
};

#[derive(Clone, Copy)]
pub enum AppInputAction {
    MiddleMouseScroll(f32),
    MouseMoved,
    MouseDragged(MouseButton, Vec2, Vec2),
    MouseClick(MouseButton, Vec2),
    MouseRelease(MouseButton),

    SwitchTo3dMode,
    SwitchTo2dMode,
    SetCameraAngleFixed(CameraAngle),
    CameraAngleYawRotate(bool),
    DeleteKeyPress,
    InsertKeyPress,
}

#[derive(Resource)]
pub struct InputManager {
    key_action_map: KeyActionMap<AppInputAction>,

    //// hover
    resync_hover: bool,
    // Option<(2d shape entity, shape type)>
    pub(crate) hovered_entity: Option<(Entity, CanvasShape)>,

    //// selection
    resync_selection: bool,
    // Option<(2d shape entity, shape type)>
    selected_shape: Option<(Entity, CanvasShape)>,
    pub select_circle_entity: Option<Entity>,
    pub select_triangle_entity: Option<Entity>,
    pub select_line_entity: Option<Entity>,
}

impl Default for InputManager {
    fn default() -> Self {
        let key_state = KeyActionMap::init(vec![
            (Key::S, AppInputAction::SwitchTo3dMode),
            (Key::W, AppInputAction::SwitchTo2dMode),
            (
                Key::Num1,
                AppInputAction::SetCameraAngleFixed(CameraAngle::Ingame(1)),
            ),
            (
                Key::Num2,
                AppInputAction::SetCameraAngleFixed(CameraAngle::Ingame(2)),
            ),
            (
                Key::Num3,
                AppInputAction::SetCameraAngleFixed(CameraAngle::Ingame(3)),
            ),
            (
                Key::Num4,
                AppInputAction::SetCameraAngleFixed(CameraAngle::Ingame(4)),
            ),
            (
                Key::Num5,
                AppInputAction::SetCameraAngleFixed(CameraAngle::Ingame(5)),
            ),
            (
                Key::D,
                AppInputAction::SetCameraAngleFixed(CameraAngle::Side),
            ),
            (
                Key::T,
                AppInputAction::SetCameraAngleFixed(CameraAngle::Top),
            ),
            (
                Key::F,
                AppInputAction::SetCameraAngleFixed(CameraAngle::Front),
            ),
            (Key::PageUp, AppInputAction::CameraAngleYawRotate(true)),
            (Key::PageDown, AppInputAction::CameraAngleYawRotate(false)),
            (Key::Insert, AppInputAction::InsertKeyPress),
            (Key::Delete, AppInputAction::DeleteKeyPress),
        ]);

        Self {
            key_action_map: key_state,
            resync_selection: false,
            resync_hover: false,

            hovered_entity: None,

            select_circle_entity: None,
            select_triangle_entity: None,
            select_line_entity: None,
            selected_shape: None,
        }
    }
}

impl InputManager {
    pub fn update_input(
        &mut self,

        // input
        input_actions: Vec<InputAction>,

        // resources
        commands: &mut Commands,
        client: &mut Client,
        canvas: &mut Canvas,
        camera_manager: &mut CameraManager,
        animation_manager: &mut AnimationManager,
        tab_state: &mut TabState,
        vertex_manager: &mut VertexManager,
        edge_manager: &mut EdgeManager,
        face_manager: &FaceManager,

        // queries
        transform_q: &mut Query<&mut Transform>,
        camera_q: &mut Query<(&mut Camera, &mut Projection)>,
        vertex_3d_q: &mut Query<&mut Vertex3d>,
        edge_angle_q: &mut Query<&mut EdgeAngle>,
    ) {
        let camera_state = &mut tab_state.camera_state;

        let mut app_actions = Vec::new();

        for action in input_actions {
            match action {
                InputAction::MiddleMouseScroll(scroll_amount) => {
                    app_actions.push(AppInputAction::MiddleMouseScroll(scroll_amount))
                }
                InputAction::MouseMoved => app_actions.push(AppInputAction::MouseMoved),
                InputAction::MouseDragged(click_type, mouse_position, delta) => {
                    if canvas.has_focus() {
                        app_actions.push(AppInputAction::MouseDragged(
                            click_type,
                            mouse_position,
                            delta,
                        ));
                    }
                }
                InputAction::MouseClick(click_type, mouse_position) => {
                    // check if mouse position is outside of canvas
                    if !canvas.is_position_inside(mouse_position) {
                        continue;
                    }

                    app_actions.push(AppInputAction::MouseClick(click_type, mouse_position))
                }
                InputAction::MouseRelease(click_type) => {
                    app_actions.push(AppInputAction::MouseRelease(click_type))
                }
                InputAction::KeyPress(key) => {
                    if let Some(action) = self.key_action_map.key_to_action(key) {
                        app_actions.push(action);
                    }
                }
                _ => {}
            }
        }

        // TODO: unify input_actions and app_actions!

        for input_action in app_actions {
            match input_action {
                AppInputAction::MiddleMouseScroll(scroll_y) => {
                    camera_manager.camera_zoom(camera_state, scroll_y);
                }
                AppInputAction::MouseMoved => {
                    self.queue_resync_hover_ui();
                    self.queue_resync_selection_ui();
                }
                AppInputAction::SwitchTo3dMode => {
                    // disable 2d camera, enable 3d camera
                    camera_state.set_3d_mode();
                    camera_manager.recalculate_3d_view();
                    canvas.queue_resync_shapes();
                }
                AppInputAction::SwitchTo2dMode => {
                    // disable 3d camera, enable 2d camera
                    camera_state.set_2d_mode();
                    camera_manager.recalculate_3d_view();
                    canvas.queue_resync_shapes();
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
                        camera_manager.set_camera_angle_ingame(camera_state, angle_index);
                    }
                },
                AppInputAction::InsertKeyPress => {
                    self.handle_insert_key_press(&canvas, &mut tab_state.action_stack);
                }
                AppInputAction::DeleteKeyPress => {
                    self.handle_delete_key_press(
                        commands,
                        client,
                        &canvas,
                        &mut tab_state.action_stack,
                        &vertex_manager,
                        &edge_manager,
                        &face_manager,
                    );
                }
                AppInputAction::CameraAngleYawRotate(clockwise) => {
                    camera_manager.set_camera_angle_yaw_rotate(camera_state, clockwise);
                }
                AppInputAction::MouseDragged(click_type, mouse_position, delta) => {
                    self.handle_mouse_drag(
                        commands,
                        client,
                        canvas,
                        camera_manager,
                        camera_state,
                        vertex_manager,
                        edge_manager,
                        animation_manager,
                        click_type,
                        mouse_position,
                        delta,
                        camera_q,
                        transform_q,
                        vertex_3d_q,
                        edge_angle_q,
                    );
                }
                AppInputAction::MouseClick(click_type, mouse_position) => {
                    self.handle_mouse_click(
                        canvas,
                        camera_manager,
                        vertex_manager,
                        edge_manager,
                        &mut tab_state.action_stack,
                        click_type,
                        &mouse_position,
                        camera_q,
                        transform_q,
                    );
                }
                AppInputAction::MouseRelease(MouseButton::Left) => {
                    if let Some((vertex_2d_entity, old_pos, new_pos)) =
                        vertex_manager.last_vertex_dragged.take()
                    {
                        tab_state
                            .action_stack
                            .buffer_action(ShapeAction::MoveVertex(
                                vertex_2d_entity,
                                old_pos,
                                new_pos,
                            ));
                    }
                    if let Some((edge_2d_entity, old_angle, new_angle)) =
                        edge_manager.last_edge_dragged.take()
                    {
                        tab_state
                            .action_stack
                            .buffer_action(ShapeAction::RotateEdge(
                                edge_2d_entity,
                                old_angle,
                                new_angle,
                            ));
                    }
                }
                _ => {}
            }
        }
    }

    // HOVER
    pub fn queue_resync_hover_ui(&mut self) {
        self.resync_hover = true;
    }

    pub(crate) fn sync_mouse_hover_ui(
        &mut self,
        canvas: &mut Canvas,
        current_tab_file_entity: Entity,
        mouse_position: &Vec2,
        camera_state: &CameraState,
        transform_q: &mut Query<(&mut Transform, Option<&LocalShape>)>,
        owned_by_q: &Query<&OwnedByFileLocal>,
        vertex_2d_q: &Query<(Entity, Option<&VertexRoot>), (With<Vertex2d>, Without<LocalShape>)>,
        edge_2d_q: &Query<(Entity, &Edge2dLocal), Without<LocalShape>>,
        face_2d_q: &Query<(Entity, &FaceIcon2d)>,
    ) {
        if !self.resync_hover {
            return;
        }
        self.resync_hover = false;

        let camera_3d_scale = camera_state.camera_3d_scale();

        let mut least_distance = f32::MAX;
        let mut least_entity = None;

        // check for vertices
        for (vertex_entity, root_opt) in vertex_2d_q.iter() {
            // check tab ownership, skip vertices from other tabs
            if !ShapeManager::is_owned_by_tab(current_tab_file_entity, owned_by_q, vertex_entity) {
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
                if !ShapeManager::is_owned_by_tab(current_tab_file_entity, owned_by_q, edge_entity)
                {
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
                if !ShapeManager::is_owned_by_tab(current_tab_file_entity, owned_by_q, face_entity)
                {
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
        canvas.queue_resync_shapes_light();
    }

    // SELECTION
    pub fn select_shape(&mut self, canvas: &mut Canvas, entity: &Entity, shape: CanvasShape) {
        self.selected_shape = Some((*entity, shape));
        canvas.queue_resync_shapes();
    }

    pub fn deselect_shape(&mut self, canvas: &mut Canvas) {
        self.selected_shape = None;
        canvas.queue_resync_shapes();
    }

    pub fn selected_shape_2d(&self) -> Option<(Entity, CanvasShape)> {
        self.selected_shape
    }

    pub fn queue_resync_selection_ui(&mut self) {
        self.resync_selection = true;
    }

    pub(crate) fn sync_selection_ui(
        &mut self,
        mouse_position: &Vec2,
        canvas: &Canvas,
        camera_state: &CameraState,
        transform_q: &mut Query<&mut Transform>,
        visibility_q: &mut Query<&mut Visibility>,
    ) {
        if !self.resync_selection {
            return;
        }
        self.resync_selection = false;

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
                        0.0,
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
                select_shape_visibilities[2].visible =
                    !canvas.current_file_type_equals(FileExtension::Anim) && canvas.has_focus();
                // select line is visible
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

    pub(crate) fn handle_insert_key_press(
        &mut self,
        canvas: &Canvas,
        action_stack: &mut ActionStack<ShapeAction>,
    ) {
        if !canvas.current_file_type_equals(FileExtension::Mesh) {
            return;
        }

        if self.selected_shape.is_some() {
            return;
        }

        action_stack.buffer_action(ShapeAction::CreateVertex(
            VertexTypeData::Mesh(Vec::new(), Vec::new()),
            Vec3::ZERO,
            None,
        ));
    }

    pub(crate) fn handle_delete_key_press(
        &mut self,
        commands: &mut Commands,
        client: &mut Client,
        canvas: &Canvas,
        action_stack: &mut ActionStack<ShapeAction>,
        vertex_manager: &VertexManager,
        edge_manager: &EdgeManager,
        face_manager: &FaceManager,
    ) {
        if canvas.current_file_type_equals(FileExtension::Anim)  {
            return;
        }

        match self.selected_shape {
            Some((vertex_2d_entity, CanvasShape::Vertex)) => {
                // delete vertex
                let vertex_3d_entity = vertex_manager
                    .vertex_entity_2d_to_3d(&vertex_2d_entity)
                    .unwrap();

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
                if canvas.current_file_type_equals(FileExtension::Skel) {
                    return;
                }
                // delete edge
                let edge_3d_entity = edge_manager.edge_entity_2d_to_3d(&edge_2d_entity).unwrap();

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
                let face_3d_entity = face_manager.face_entity_2d_to_3d(&face_2d_entity).unwrap();

                // check whether we can delete edge
                let auth_status = commands.entity(face_3d_entity).authority(client).unwrap();
                if !auth_status.is_granted() && !auth_status.is_available() {
                    // do nothing, face is not available
                    // TODO: queue for deletion? check before this?
                    warn!("Face `{:?}` is not available for deletion!", face_3d_entity);
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

    pub(crate) fn handle_mouse_click(
        &mut self,
        canvas: &Canvas,
        camera_manager: &CameraManager,
        vertex_manager: &VertexManager,
        edge_manager: &EdgeManager,
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
                        if canvas.current_file_type_equals(FileExtension::Skel) {
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
                                    action_stack.buffer_action(ShapeAction::SelectShape(
                                        self.hovered_entity,
                                    ));
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
                        if edge_manager
                            .edge_2d_entity_from_vertices(
                                vertex_manager,
                                vertex_2d_entity_a,
                                vertex_2d_entity_b,
                            )
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
                                match canvas.get_current_file_type() {
                                    FileExtension::Skel => {
                                        VertexTypeData::Skel(vertex_2d_entity, 0.0, None)
                                    }
                                    FileExtension::Mesh => VertexTypeData::Mesh(
                                        vec![(vertex_2d_entity, None)],
                                        Vec::new(),
                                    ),
                                    FileExtension::Anim | FileExtension::Unknown => {
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
                        action_stack.buffer_action(ShapeAction::SelectShape(self.hovered_entity));
                    }
                    (CanvasShape::Face, MouseButton::Left) => {
                        if canvas.current_file_type_equals(FileExtension::Mesh) {
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

    pub(crate) fn handle_mouse_drag(
        &mut self,
        commands: &mut Commands,
        client: &Client,
        canvas: &mut Canvas,
        camera_manager: &mut CameraManager,
        camera_state: &mut CameraState,
        vertex_manager: &mut VertexManager,
        edge_manager: &mut EdgeManager,
        animation_manager: &mut AnimationManager,
        click_type: MouseButton,
        mouse_position: Vec2,
        delta: Vec2,
        camera_q: &Query<(&mut Camera, &mut Projection)>,
        transform_q: &Query<&mut Transform>,
        vertex_3d_q: &mut Query<&mut Vertex3d>,
        edge_angle_q: &mut Query<&mut EdgeAngle>,
    ) {
        let vertex_is_selected = self.selected_shape.is_some();
        let shape_can_drag = vertex_is_selected
            && match self.selected_shape.unwrap().1 {
                CanvasShape::RootVertex | CanvasShape::Vertex => true,
                CanvasShape::Edge => !canvas.current_file_type_equals(FileExtension::Mesh),
                _ => false,
            };

        if vertex_is_selected && shape_can_drag {
            match click_type {
                MouseButton::Left => {
                    match self.selected_shape.unwrap() {
                        (vertex_2d_entity, CanvasShape::Vertex) => {
                            // move vertex
                            let Some(vertex_3d_entity) = vertex_manager.vertex_entity_2d_to_3d(&vertex_2d_entity) else {
                                warn!(
                                    "Selected vertex entity: {:?} has no 3d counterpart",
                                    vertex_2d_entity
                                );
                                return;
                            };

                            if canvas.current_file_type_equals(FileExtension::Anim) {
                                animation_manager.drag_vertex(
                                    commands,
                                    client,
                                    vertex_3d_entity,
                                    mouse_position,
                                    delta,
                                );
                                return;
                            }

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

                            if let Some((_, old_3d_position, _)) =
                                vertex_manager.last_vertex_dragged
                            {
                                vertex_manager.last_vertex_dragged =
                                    Some((vertex_2d_entity, old_3d_position, new_3d_position));
                            } else {
                                let old_3d_position = vertex_3d.as_vec3();
                                vertex_manager.last_vertex_dragged =
                                    Some((vertex_2d_entity, old_3d_position, new_3d_position));
                            }

                            vertex_3d.set_x(new_3d_position.x as i16);
                            vertex_3d.set_y(new_3d_position.y as i16);
                            vertex_3d.set_z(new_3d_position.z as i16);

                            // redraw
                            canvas.queue_resync_shapes();
                        }
                        (edge_2d_entity, CanvasShape::Edge) => {
                            // rotate edge angle
                            let edge_3d_entity =
                                edge_manager.edge_entity_2d_to_3d(&edge_2d_entity).unwrap();

                            if canvas.current_file_type_equals(FileExtension::Anim) {
                                animation_manager.drag_edge(
                                    commands,
                                    client,
                                    edge_3d_entity,
                                    mouse_position,
                                    delta,
                                );
                                return;
                            }

                            let auth_status =
                                commands.entity(edge_3d_entity).authority(client).unwrap();
                            if !(auth_status.is_requested() || auth_status.is_granted()) {
                                // only continue to mutate if requested or granted authority over edge
                                info!("No authority over edge, skipping..");
                                return;
                            }

                            let edge_2d_transform = transform_q.get(edge_2d_entity).unwrap();
                            let start_pos = edge_2d_transform.translation.truncate();
                            let end_pos = get_2d_line_transform_endpoint(&edge_2d_transform);
                            let base_angle = angle_between(&start_pos, &end_pos);

                            let edge_angle_entity = edge_manager
                                .edges_3d
                                .get(&edge_3d_entity)
                                .unwrap()
                                .angle_entities_opt
                                .unwrap()
                                .0;
                            let edge_angle_pos = transform_q
                                .get(edge_angle_entity)
                                .unwrap()
                                .translation
                                .truncate();

                            let mut edge_angle = edge_angle_q.get_mut(edge_3d_entity).unwrap();
                            let new_angle = normalize_angle(
                                angle_between(&edge_angle_pos, &mouse_position)
                                    - FRAC_PI_2
                                    - base_angle,
                            );
                            if let Some((_, prev_angle, _)) = edge_manager.last_edge_dragged {
                                edge_manager.last_edge_dragged =
                                    Some((edge_2d_entity, prev_angle, new_angle));
                            } else {
                                let old_angle = edge_angle.get_radians();
                                edge_manager.last_edge_dragged =
                                    Some((edge_2d_entity, old_angle, new_angle));
                            }
                            edge_angle.set_radians(new_angle);

                            // redraw
                            canvas.queue_resync_shapes();
                        }
                        _ => {
                            panic!("Shouldn't be possible");
                        }
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

    pub(crate) fn sync_hover_shape_scale(
        &mut self,
        transform_q: &mut Query<&mut Transform>,
        camera_3d_scale: f32,
    ) {
        let hover_vertex_2d_scale = Vertex2d::HOVER_RADIUS * camera_3d_scale;
        let hover_edge_2d_scale = Edge2dLocal::HOVER_THICKNESS * camera_3d_scale;
        let hover_face_2d_scale = FaceIcon2d::HOVER_SIZE * camera_3d_scale;

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
}
