use std::f32::consts::FRAC_PI_2;

use bevy_ecs::{
    entity::Entity,
    query::{With, Without},
    system::{Commands, Query, Res, ResMut, Resource, SystemState},
    world::{Mut, World},
};
use bevy_log::{info, warn};

use naia_bevy_client::{Client, CommandsExt, Instant};

use input::{InputAction, Key, MouseButton};
use math::{convert_2d_to_3d, Vec2, Vec3};
use render_api::{
    components::{Camera, CameraProjection, Projection, Transform, Visibility},
    shapes::{
        angle_between, distance_to_2d_line, get_2d_line_transform_endpoint, normalize_angle,
        set_2d_line_transform,
    },
};

use vortex_proto::components::{EdgeAngle, FileExtension, ShapeName, Vertex3d, VertexRoot};

use crate::app::{
    components::{
        Edge2dLocal, FaceIcon2d, LocalShape, SelectCircle, SelectTriangle, Vertex2d, VertexTypeData,
    },
    resources::{
        action::AnimAction,
        action::ShapeAction,
        animation_manager::{AnimationManager, anim_file_play_pause, anim_file_delete_frame, anim_file_insert_frame},
        camera_manager::CameraAngle,
        camera_manager::CameraManager,
        camera_state::CameraState,
        canvas::Canvas,
        edge_manager::EdgeManager,
        face_manager::FaceManager,
        file_manager::FileManager,
        shape_data::CanvasShape,
        tab_manager::TabManager,
        vertex_manager::VertexManager,
    },
    ui::widgets::naming_bar_visibility_toggle,
};

#[derive(Clone, Copy)]
pub enum CardinalDirection {
    North,
    East,
    South,
    West,
}

#[derive(Resource)]
pub struct InputManager {
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

    //doubleclick
    last_left_click_instant: Instant,
    last_frame_index_hover: usize,
}

impl Default for InputManager {
    fn default() -> Self {
        Self {
            resync_selection: false,
            resync_hover: false,

            hovered_entity: None,

            select_circle_entity: None,
            select_triangle_entity: None,
            select_line_entity: None,
            selected_shape: None,

            last_left_click_instant: Instant::now(),
            last_frame_index_hover: 0,
        }
    }
}

impl InputManager {
    pub fn update_input(&mut self, input_actions: Vec<InputAction>, world: &mut World) {
        let current_file_entity = world
            .get_resource::<TabManager>()
            .unwrap()
            .current_tab_entity()
            .unwrap();
        let current_file_type = world
            .get_resource::<FileManager>()
            .unwrap()
            .get_file_type(current_file_entity);
        match current_file_type {
            FileExtension::Skel => self.update_input_skel(input_actions, world),
            FileExtension::Mesh => self.update_input_mesh(input_actions, world),
            FileExtension::Anim => self.update_input_anim(input_actions, world),
            _ => {}
        }
    }

    fn update_input_skel(&mut self, input_actions: Vec<InputAction>, world: &mut World) {
        for action in input_actions {
            match action {
                InputAction::MouseClick(click_type, mouse_position) => self
                    .handle_mouse_click_skelmesh(
                        world,
                        FileExtension::Skel,
                        click_type,
                        &mouse_position,
                    ),
                InputAction::MouseDragged(click_type, mouse_position, delta) => self
                    .handle_mouse_drag_skelmesh(
                        world,
                        FileExtension::Skel,
                        click_type,
                        mouse_position,
                        delta,
                    ),
                InputAction::MiddleMouseScroll(scroll_y) => {
                    Self::handle_mouse_scroll_wheel(world, scroll_y)
                }
                InputAction::MouseMoved => {
                    self.queue_resync_hover_ui();
                    self.queue_resync_selection_ui();
                }
                InputAction::MouseRelease(MouseButton::Left) => {
                    self.reset_last_dragged_vertex(world);
                    self.reset_last_dragged_edge(world);
                }
                InputAction::KeyPress(key) => match key {
                    Key::S
                    | Key::W
                    | Key::D
                    | Key::T
                    | Key::F
                    | Key::Num1
                    | Key::Num2
                    | Key::Num3
                    | Key::Num4
                    | Key::Num5
                    | Key::PageUp
                    | Key::PageDown => Self::handle_keypress_camera_controls(world, key),
                    Key::Delete => self.handle_delete_key_press_skel(world),
                    Key::N => naming_bar_visibility_toggle(world, self),
                    Key::E => Self::handle_edge_angle_visibility_toggle(world),
                    _ => {}
                },
                _ => {}
            }
        }
    }

    fn update_input_mesh(&mut self, input_actions: Vec<InputAction>, world: &mut World) {
        for action in input_actions {
            match action {
                InputAction::MouseClick(click_type, mouse_position) => self
                    .handle_mouse_click_skelmesh(
                        world,
                        FileExtension::Mesh,
                        click_type,
                        &mouse_position,
                    ),
                InputAction::MouseDragged(click_type, mouse_position, delta) => self
                    .handle_mouse_drag_skelmesh(
                        world,
                        FileExtension::Mesh,
                        click_type,
                        mouse_position,
                        delta,
                    ),
                InputAction::MiddleMouseScroll(scroll_y) => {
                    Self::handle_mouse_scroll_wheel(world, scroll_y)
                }
                InputAction::MouseMoved => {
                    self.queue_resync_hover_ui();
                    self.queue_resync_selection_ui();
                }
                InputAction::MouseRelease(MouseButton::Left) => {
                    self.reset_last_dragged_vertex(world)
                }
                InputAction::KeyPress(key) => match key {
                    Key::S
                    | Key::W
                    | Key::D
                    | Key::T
                    | Key::F
                    | Key::Num1
                    | Key::Num2
                    | Key::Num3
                    | Key::Num4
                    | Key::Num5
                    | Key::PageUp
                    | Key::PageDown => Self::handle_keypress_camera_controls(world, key),
                    Key::Delete => self.handle_delete_key_press_mesh(world),
                    Key::Insert => self.handle_insert_key_press_mesh(world),
                    _ => {}
                },
                _ => {}
            }
        }
    }

    fn update_input_anim(&mut self, input_actions: Vec<InputAction>, world: &mut World) {
        let animation_manager = world.get_resource_mut::<AnimationManager>().unwrap();
        if animation_manager.is_posing() {
            self.update_input_anim_posing(input_actions, world);
        } else {
            self.update_input_anim_framing(input_actions, world);
        }
    }

    fn update_input_anim_framing(&mut self, input_actions: Vec<InputAction>, world: &mut World) {
        for action in input_actions {
            match action {
                InputAction::MouseClick(click_type, mouse_position) => {
                    self.handle_mouse_click_anim_framing(world, click_type, &mouse_position)
                }
                InputAction::MouseDragged(click_type, mouse_position, delta) => {
                    self.handle_mouse_drag_anim_framing(world, click_type, mouse_position, delta)
                }
                InputAction::MiddleMouseScroll(scroll_y) => {
                    Self::handle_mouse_scroll_anim_framing(world, scroll_y)
                }
                InputAction::MouseMoved => {
                    let mut animation_manager =
                        world.get_resource_mut::<AnimationManager>().unwrap();
                    animation_manager.framing_queue_resync_hover_ui();
                }
                InputAction::KeyPress(key) => match key {
                    Key::Delete => anim_file_delete_frame(self, world),
                    Key::Insert => anim_file_insert_frame(self, world),
                    Key::Space => anim_file_play_pause(world),
                    Key::Enter => {
                        let mut system_state: SystemState<(
                            ResMut<Canvas>,
                            ResMut<AnimationManager>,
                        )> = SystemState::new(world);
                        let (mut canvas, mut animation_manager) = system_state.get_mut(world);
                        animation_manager.set_posing(&mut canvas);
                    }
                    Key::ArrowLeft | Key::ArrowRight | Key::ArrowUp | Key::ArrowDown => {
                        let dir = match key {
                            Key::ArrowLeft => CardinalDirection::West,
                            Key::ArrowRight => CardinalDirection::East,
                            Key::ArrowUp => CardinalDirection::North,
                            Key::ArrowDown => CardinalDirection::South,
                            _ => panic!("Unexpected key: {:?}", key),
                        };
                        let current_file_entity = *world
                            .get_resource::<TabManager>()
                            .unwrap()
                            .current_tab_entity()
                            .unwrap();
                        let mut animation_manager =
                            world.get_resource_mut::<AnimationManager>().unwrap();
                        if let Some((prev_index, next_index)) =
                            animation_manager.framing_navigate(&current_file_entity, dir)
                        {
                            world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                                tab_manager.current_tab_execute_anim_action(
                                    world,
                                    self,
                                    AnimAction::SelectFrame(
                                        current_file_entity,
                                        next_index,
                                        prev_index,
                                    ),
                                );
                            });
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    fn update_input_anim_posing(&mut self, input_actions: Vec<InputAction>, world: &mut World) {
        for action in input_actions {
            match action {
                InputAction::MouseClick(click_type, mouse_position) => {
                    self.handle_mouse_click_anim_posing(world, click_type, &mouse_position)
                }
                InputAction::MouseDragged(click_type, mouse_position, delta) => {
                    self.handle_mouse_drag_anim_posing(world, click_type, mouse_position, delta)
                }
                InputAction::MiddleMouseScroll(scroll_y) => {
                    Self::handle_mouse_scroll_wheel(world, scroll_y)
                }
                InputAction::MouseMoved => {
                    self.queue_resync_hover_ui();
                    self.queue_resync_selection_ui();
                }
                InputAction::MouseRelease(MouseButton::Left) => {
                    self.reset_last_dragged_rotation(world)
                }
                InputAction::KeyPress(key) => match key {
                    Key::S
                    | Key::W
                    | Key::D
                    | Key::T
                    | Key::F
                    | Key::Num1
                    | Key::Num2
                    | Key::Num3
                    | Key::Num4
                    | Key::Num5
                    | Key::PageUp
                    | Key::PageDown => Self::handle_keypress_camera_controls(world, key),
                    Key::E => Self::handle_edge_angle_visibility_toggle(world),
                    Key::Escape => {
                        let mut animation_manager =
                            world.get_resource_mut::<AnimationManager>().unwrap();
                        animation_manager.set_framing();
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    fn reset_last_dragged_vertex(&mut self, world: &mut World) {
        // reset last dragged vertex
        if let Some(drags) = world
            .get_resource_mut::<VertexManager>()
            .unwrap()
            .take_drags()
        {
            world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                for (vertex_2d_entity, old_pos, new_pos) in drags {
                    tab_manager.current_tab_execute_shape_action(
                        world,
                        self,
                        ShapeAction::MoveVertex(vertex_2d_entity, old_pos, new_pos, true),
                    );
                }
            });
        }
    }

    fn reset_last_dragged_edge(&mut self, world: &mut World) {
        // reset last dragged edge
        if let Some((edge_2d_entity, old_angle, new_angle)) = world
            .get_resource_mut::<EdgeManager>()
            .unwrap()
            .take_last_edge_dragged()
        {
            world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                tab_manager.current_tab_execute_shape_action(
                    world,
                    self,
                    ShapeAction::RotateEdge(edge_2d_entity, old_angle, new_angle),
                );
            });
        }
    }

    fn reset_last_dragged_rotation(&mut self, world: &mut World) {
        // reset last dragged rotation
        if let Some((vertex_2d_entity, old_angle, new_angle)) = world
            .get_resource_mut::<AnimationManager>()
            .unwrap()
            .take_last_rotation_dragged()
        {
            world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                tab_manager.current_tab_execute_anim_action(
                    world,
                    self,
                    AnimAction::RotateVertex(vertex_2d_entity, old_angle, Some(new_angle)),
                );
            });
        }
    }

    fn handle_edge_angle_visibility_toggle(world: &mut World) {
        let mut system_state: SystemState<(
            ResMut<Canvas>,
            ResMut<EdgeManager>,
            Res<FileManager>,
            Res<TabManager>,
        )> = SystemState::new(world);
        let (mut canvas, mut edge_manager, file_manager, tab_manager) = system_state.get_mut(world);

        edge_manager.edge_angle_visibility_toggle(&file_manager, &tab_manager, &mut canvas);

        system_state.apply(world);
    }

    fn handle_keypress_camera_controls(world: &mut World, key: Key) {
        match key {
            Key::S => {
                // disable 2d camera, enable 3d camera
                world
                    .get_resource_mut::<TabManager>()
                    .unwrap()
                    .current_tab_camera_state_mut()
                    .unwrap()
                    .set_3d_mode();
                world
                    .get_resource_mut::<CameraManager>()
                    .unwrap()
                    .recalculate_3d_view();
                world
                    .get_resource_mut::<Canvas>()
                    .unwrap()
                    .queue_resync_shapes();
            }
            Key::W => {
                // disable 3d camera, enable 2d camera
                world
                    .get_resource_mut::<TabManager>()
                    .unwrap()
                    .current_tab_camera_state_mut()
                    .unwrap()
                    .set_2d_mode();
                world
                    .get_resource_mut::<CameraManager>()
                    .unwrap()
                    .recalculate_3d_view();
                world
                    .get_resource_mut::<Canvas>()
                    .unwrap()
                    .queue_resync_shapes();
            }
            Key::D
            | Key::T
            | Key::F
            | Key::Num1
            | Key::Num2
            | Key::Num3
            | Key::Num4
            | Key::Num5 => {
                let camera_angle = match key {
                    Key::D => CameraAngle::Side,
                    Key::T => CameraAngle::Top,
                    Key::F => CameraAngle::Front,
                    Key::Num1 => CameraAngle::Ingame(1),
                    Key::Num2 => CameraAngle::Ingame(2),
                    Key::Num3 => CameraAngle::Ingame(3),
                    Key::Num4 => CameraAngle::Ingame(4),
                    Key::Num5 => CameraAngle::Ingame(5),
                    _ => panic!("Unexpected key: {:?}", key),
                };

                let mut system_state: SystemState<(ResMut<TabManager>, ResMut<CameraManager>)> =
                    SystemState::new(world);
                let (mut tab_manager, mut camera_manager) = system_state.get_mut(world);

                match camera_angle {
                    CameraAngle::Side => camera_manager
                        .set_camera_angle_side(tab_manager.current_tab_camera_state_mut().unwrap()),
                    CameraAngle::Front => camera_manager.set_camera_angle_front(
                        tab_manager.current_tab_camera_state_mut().unwrap(),
                    ),
                    CameraAngle::Top => camera_manager
                        .set_camera_angle_top(tab_manager.current_tab_camera_state_mut().unwrap()),
                    CameraAngle::Ingame(angle_index) => camera_manager.set_camera_angle_ingame(
                        tab_manager.current_tab_camera_state_mut().unwrap(),
                        angle_index,
                    ),
                }
            }
            Key::PageUp | Key::PageDown => {
                let clockwise = match key {
                    Key::PageUp => true,
                    Key::PageDown => false,
                    _ => panic!("Unexpected key: {:?}", key),
                };
                let mut system_state: SystemState<(ResMut<TabManager>, ResMut<CameraManager>)> =
                    SystemState::new(world);
                let (mut tab_manager, mut camera_manager) = system_state.get_mut(world);

                camera_manager.set_camera_angle_yaw_rotate(
                    tab_manager.current_tab_camera_state_mut().unwrap(),
                    clockwise,
                );
            }
            _ => panic!("Unexpected key: {:?}", key),
        }
    }

    // HOVER
    pub fn queue_resync_hover_ui(&mut self) {
        self.resync_hover = true;
    }

    pub(crate) fn sync_mouse_hover_ui(
        &mut self,
        file_ext: FileExtension,
        canvas: &mut Canvas,
        vertex_manager: &VertexManager,
        edge_manager: &EdgeManager,
        transform_q: &mut Query<(&mut Transform, Option<&LocalShape>)>,
        visibility_q: &Query<&Visibility>,
        shape_name_q: &Query<&ShapeName>,
        vertex_2d_q: &Query<(Entity, Option<&VertexRoot>), (With<Vertex2d>, Without<LocalShape>)>,
        edge_2d_q: &Query<(Entity, &Edge2dLocal), Without<LocalShape>>,
        face_2d_q: &Query<(Entity, &FaceIcon2d)>,
        camera_state: &CameraState,
        mouse_position: &Vec2,
    ) {
        if !self.resync_hover {
            return;
        }
        self.resync_hover = false;

        let camera_3d_scale = camera_state.camera_3d_scale();

        let mut least_distance = f32::MAX;
        let mut least_entity = None;

        // check for vertices
        for (vertex_2d_entity, root_opt) in vertex_2d_q.iter() {
            let Ok(visibility) = visibility_q.get(vertex_2d_entity) else {
                panic!("Vertex entity has no Visibility");
            };
            if !visibility.visible {
                continue;
            }

            // don't hover over disabled vertices in Anim mode
            if file_ext == FileExtension::Anim {
                let vertex_3d_entity = vertex_manager
                    .vertex_entity_2d_to_3d(&vertex_2d_entity)
                    .unwrap();
                let Ok(shape_name) = shape_name_q.get(vertex_3d_entity) else { continue; };
                let shape_name = shape_name.value.as_str();
                if shape_name.len() == 0 {
                    continue;
                }
            }

            let (vertex_transform, _) = transform_q.get(vertex_2d_entity).unwrap();
            let vertex_position = vertex_transform.translation.truncate();
            let distance = vertex_position.distance(*mouse_position);
            if distance < least_distance {
                least_distance = distance;

                let shape = match root_opt {
                    Some(_) => CanvasShape::RootVertex,
                    None => CanvasShape::Vertex,
                };

                least_entity = Some((vertex_2d_entity, shape));
            }
        }

        let mut is_hovering = least_distance <= (Vertex2d::DETECT_RADIUS * camera_3d_scale);

        // check for edges
        if !is_hovering {
            for (edge_2d_entity, _) in edge_2d_q.iter() {
                // check visibility
                let Ok(visibility) = visibility_q.get(edge_2d_entity) else {
                    panic!("entity has no Visibility");
                };
                if !visibility.visible {
                    continue;
                }
                if file_ext == FileExtension::Anim {
                    let edge_3d_entity =
                        edge_manager.edge_entity_2d_to_3d(&edge_2d_entity).unwrap();
                    let (_, end_vertex_3d_entity) =
                        edge_manager.edge_get_endpoints(&edge_3d_entity);
                    let Ok(shape_name) = shape_name_q.get(end_vertex_3d_entity) else { continue; };
                    let shape_name = shape_name.value.as_str();
                    if shape_name.len() == 0 {
                        continue;
                    }
                }

                let (edge_transform, _) = transform_q.get(edge_2d_entity).unwrap();
                let edge_start = edge_transform.translation.truncate();
                let edge_end = get_2d_line_transform_endpoint(&edge_transform);

                let distance = distance_to_2d_line(*mouse_position, edge_start, edge_end);
                if distance < least_distance {
                    least_distance = distance;
                    least_entity = Some((edge_2d_entity, CanvasShape::Edge));
                }
            }

            is_hovering = least_distance <= (Edge2dLocal::DETECT_THICKNESS * camera_3d_scale);
        }

        // check for faces
        if !is_hovering {
            for (face_entity, _) in face_2d_q.iter() {
                // check tab ownership, skip faces from other tabs
                let Ok(visibility) = visibility_q.get(face_entity) else {
                    panic!("entity has no Visibility");
                };
                if !visibility.visible {
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

        self.sync_hover_shape_scale(transform_q, camera_3d_scale);

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
        if self.selected_shape.is_some() {
            panic!("must deselect before selecting");
        }
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
        canvas: &Canvas,
        file_manager: &FileManager,
        tab_manager: &TabManager,
        transform_q: &mut Query<&mut Transform>,
        visibility_q: &mut Query<&mut Visibility>,
        mouse_position: &Vec2,
    ) {
        if !self.resync_selection {
            return;
        }
        self.resync_selection = false;

        let Some(current_tab_state) = tab_manager.current_tab_state() else {
            return;
        };

        let current_tab_camera_state = &current_tab_state.camera_state;

        let camera_3d_scale = current_tab_camera_state.camera_3d_scale();

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

                let current_file_entity = tab_manager.current_tab_entity().unwrap();
                let current_file_type = file_manager.get_file_type(&current_file_entity);

                select_shape_visibilities[0].visible = true; // select circle is visible
                select_shape_visibilities[1].visible = false; // no select triangle visible
                select_shape_visibilities[2].visible =
                    current_file_type != FileExtension::Anim && canvas.has_focus();
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

    pub(crate) fn handle_insert_key_press_mesh(&mut self, world: &mut World) {
        if self.selected_shape.is_some() {
            return;
        }
        world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
            tab_manager.current_tab_execute_shape_action(
                world,
                self,
                ShapeAction::CreateVertex(
                    VertexTypeData::Mesh(Vec::new(), Vec::new()),
                    Vec3::ZERO,
                    None,
                ),
            );
        })
    }

    pub(crate) fn handle_delete_key_press_skel(&mut self, world: &mut World) {
        match self.selected_shape {
            Some((vertex_2d_entity, CanvasShape::Vertex)) => {
                self.handle_delete_vertex_action(world, &vertex_2d_entity)
            }
            _ => {}
        }
    }

    pub(crate) fn handle_delete_key_press_mesh(&mut self, world: &mut World) {
        match self.selected_shape {
            Some((vertex_2d_entity, CanvasShape::Vertex)) => {
                self.handle_delete_vertex_action(world, &vertex_2d_entity)
            }
            Some((edge_2d_entity, CanvasShape::Edge)) => {
                let mut system_state: SystemState<(Commands, Client, Res<EdgeManager>)> =
                    SystemState::new(world);
                let (mut commands, mut client, edge_manager) = system_state.get_mut(world);

                // delete edge
                let edge_3d_entity = edge_manager.edge_entity_2d_to_3d(&edge_2d_entity).unwrap();

                // check whether we can delete edge
                let auth_status = commands.entity(edge_3d_entity).authority(&client).unwrap();
                if !auth_status.is_granted() && !auth_status.is_available() {
                    // do nothing, edge is not available
                    // TODO: queue for deletion? check before this?
                    warn!("Edge {:?} is not available for deletion!", edge_3d_entity);
                    return;
                }

                let auth_status = commands.entity(edge_3d_entity).authority(&client).unwrap();
                if !auth_status.is_granted() {
                    // request authority if needed
                    commands
                        .entity(edge_3d_entity)
                        .request_authority(&mut client);
                }

                world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                    tab_manager.current_tab_execute_shape_action(
                        world,
                        self,
                        ShapeAction::DeleteEdge(edge_2d_entity, None),
                    );
                });

                self.selected_shape = None;
            }
            Some((face_2d_entity, CanvasShape::Face)) => {
                let mut system_state: SystemState<(Commands, Client, Res<FaceManager>)> =
                    SystemState::new(world);
                let (mut commands, mut client, face_manager) = system_state.get_mut(world);

                let face_3d_entity = face_manager.face_entity_2d_to_3d(&face_2d_entity).unwrap();

                // check whether we can delete edge
                let auth_status = commands.entity(face_3d_entity).authority(&client).unwrap();
                if !auth_status.is_granted() && !auth_status.is_available() {
                    // do nothing, face is not available
                    // TODO: queue for deletion? check before this?
                    warn!("Face `{:?}` is not available for deletion!", face_3d_entity);
                    return;
                }

                let auth_status = commands.entity(face_3d_entity).authority(&client).unwrap();
                if !auth_status.is_granted() {
                    // request authority if needed
                    commands
                        .entity(face_3d_entity)
                        .request_authority(&mut client);
                }

                world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                    tab_manager.current_tab_execute_shape_action(
                        world,
                        self,
                        ShapeAction::DeleteFace(face_2d_entity),
                    );
                });

                self.selected_shape = None;
            }
            _ => {}
        }
    }

    fn handle_delete_vertex_action(&mut self, world: &mut World, vertex_2d_entity: &Entity) {
        let mut system_state: SystemState<(Commands, Client, Res<VertexManager>)> =
            SystemState::new(world);
        let (mut commands, mut client, vertex_manager) = system_state.get_mut(world);

        // delete vertex
        let vertex_3d_entity = vertex_manager
            .vertex_entity_2d_to_3d(&vertex_2d_entity)
            .unwrap();

        // check whether we can delete vertex
        let auth_status = commands
            .entity(vertex_3d_entity)
            .authority(&client)
            .unwrap();
        if !auth_status.is_granted() && !auth_status.is_available() {
            // do nothing, vertex is not available
            // TODO: queue for deletion? check before this?
            warn!(
                "Vertex {:?} is not available for deletion!",
                vertex_3d_entity
            );
            return;
        }

        let auth_status = commands
            .entity(vertex_3d_entity)
            .authority(&client)
            .unwrap();
        if !auth_status.is_granted() {
            // request authority if needed
            commands
                .entity(vertex_3d_entity)
                .request_authority(&mut client);
        }

        world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
            tab_manager.current_tab_execute_shape_action(
                world,
                self,
                ShapeAction::DeleteVertex(*vertex_2d_entity, None),
            );
        });

        self.selected_shape = None;
    }

    pub(crate) fn handle_mouse_click_skelmesh(
        &mut self,
        world: &mut World,
        current_file_type: FileExtension,
        click_type: MouseButton,
        mouse_position: &Vec2,
    ) {
        // check if mouse position is outside of canvas
        if !world
            .get_resource::<Canvas>()
            .unwrap()
            .is_position_inside(*mouse_position)
        {
            return;
        }

        let mut system_state: SystemState<(
            Res<CameraManager>,
            Res<VertexManager>,
            Res<EdgeManager>,
            Query<(&Camera, &Projection)>,
            Query<&Transform>,
        )> = SystemState::new(world);
        let (camera_manager, vertex_manager, edge_manager, camera_q, transform_q) =
            system_state.get_mut(world);

        let selected_shape = self.selected_shape.map(|(_, shape)| shape);
        let hovered_shape = self.hovered_entity.map(|(_, shape)| shape);

        // click_type, selected_shape, hovered_shape, current_file_type
        match (click_type, selected_shape, hovered_shape, current_file_type) {
            (
                MouseButton::Left,
                Some(CanvasShape::Edge | CanvasShape::Face),
                _,
                FileExtension::Skel | FileExtension::Mesh,
            ) => {
                // should not ever be able to attach something to an edge or face?
                // select hovered entity
                world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                    tab_manager.current_tab_execute_shape_action(
                        world,
                        self,
                        ShapeAction::SelectShape(self.hovered_entity),
                    );
                });
                return;
            }
            (
                MouseButton::Left,
                Some(CanvasShape::Vertex | CanvasShape::RootVertex),
                Some(_),
                FileExtension::Skel,
            ) => {
                // skel file type does nothing when trying to connect vertices together
                // select hovered entity
                world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                    tab_manager.current_tab_execute_shape_action(
                        world,
                        self,
                        ShapeAction::SelectShape(self.hovered_entity),
                    );
                });
                return;
            }
            (
                MouseButton::Left,
                Some(_),
                Some(CanvasShape::Edge | CanvasShape::Face),
                FileExtension::Mesh,
            ) => {
                // should not ever be able to attach something to an edge or face?
                // select hovered entity
                world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                    tab_manager.current_tab_execute_shape_action(
                        world,
                        self,
                        ShapeAction::SelectShape(self.hovered_entity),
                    );
                });
                return;
            }
            (
                MouseButton::Left,
                Some(CanvasShape::Vertex | CanvasShape::RootVertex),
                Some(CanvasShape::Vertex | CanvasShape::RootVertex),
                FileExtension::Mesh,
            ) => {
                // link vertices together
                let (vertex_2d_entity_a, _) = self.selected_shape.unwrap();
                let (vertex_2d_entity_b, _) = self.hovered_entity.unwrap();
                if vertex_2d_entity_a == vertex_2d_entity_b {
                    return;
                }

                // check if edge already exists
                if edge_manager
                    .edge_2d_entity_from_vertices(
                        &vertex_manager,
                        vertex_2d_entity_a,
                        vertex_2d_entity_b,
                    )
                    .is_some()
                {
                    // select vertex
                    world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                        tab_manager.current_tab_execute_shape_action(
                            world,
                            self,
                            ShapeAction::SelectShape(Some((
                                vertex_2d_entity_b,
                                CanvasShape::Vertex,
                            ))),
                        );
                    });
                    return;
                } else {
                    // create edge
                    world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                        tab_manager.current_tab_execute_shape_action(
                            world,
                            self,
                            ShapeAction::CreateEdge(
                                vertex_2d_entity_a,
                                vertex_2d_entity_b,
                                (vertex_2d_entity_b, CanvasShape::Vertex),
                                None,
                                None,
                            ),
                        );
                    });
                    return;
                }
            }
            (
                MouseButton::Left,
                Some(CanvasShape::Vertex | CanvasShape::RootVertex),
                None,
                FileExtension::Skel | FileExtension::Mesh,
            ) => {
                // create new vertex

                // get camera
                let camera_3d = camera_manager.camera_3d_entity().unwrap();
                let camera_transform: Transform = *transform_q.get(camera_3d).unwrap();
                let (camera, camera_projection) = camera_q.get(camera_3d).unwrap();

                let camera_viewport = camera.viewport.unwrap();
                let view_matrix = camera_transform.view_matrix();
                let projection_matrix = camera_projection.projection_matrix(&camera_viewport);

                // get 2d vertex transform
                let (vertex_2d_entity, _) = self.selected_shape.unwrap();
                let Ok(vertex_2d_transform) = transform_q.get(vertex_2d_entity) else {
                    warn!(
                        "Selected vertex entity: {:?} has no Transform",
                        vertex_2d_entity
                    );
                    return;
                };
                // convert 2d to 3d
                let new_3d_position = convert_2d_to_3d(
                    &view_matrix,
                    &projection_matrix,
                    &camera_viewport.size_vec2(),
                    &mouse_position,
                    vertex_2d_transform.translation.z,
                );

                // spawn new vertex
                world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                    tab_manager.current_tab_execute_shape_action(
                        world,
                        self,
                        ShapeAction::CreateVertex(
                            match current_file_type {
                                FileExtension::Skel => {
                                    VertexTypeData::Skel(vertex_2d_entity, 0.0, None)
                                }
                                FileExtension::Mesh => {
                                    VertexTypeData::Mesh(vec![(vertex_2d_entity, None)], Vec::new())
                                }
                                _ => {
                                    panic!("");
                                }
                            },
                            new_3d_position,
                            None,
                        ),
                    );
                });
            }
            (
                MouseButton::Left,
                None,
                Some(CanvasShape::RootVertex | CanvasShape::Vertex | CanvasShape::Edge),
                FileExtension::Skel | FileExtension::Mesh,
            ) => {
                world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                    tab_manager.current_tab_execute_shape_action(
                        world,
                        self,
                        ShapeAction::SelectShape(self.hovered_entity),
                    );
                });
            }
            (MouseButton::Left, None, Some(CanvasShape::Face), FileExtension::Mesh) => {
                world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                    tab_manager.current_tab_execute_shape_action(
                        world,
                        self,
                        ShapeAction::SelectShape(self.hovered_entity),
                    );
                });
            }
            (MouseButton::Right, _, _, FileExtension::Skel | FileExtension::Mesh) => {
                // deselect vertex
                world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                    tab_manager.current_tab_execute_shape_action(
                        world,
                        self,
                        ShapeAction::SelectShape(None),
                    );
                });
            }
            _ => {}
        }
    }

    pub(crate) fn handle_mouse_click_anim_framing(
        &mut self,
        world: &mut World,
        click_type: MouseButton,
        mouse_position: &Vec2,
    ) {
        if click_type != MouseButton::Left {
            return;
        }

        // check if mouse position is outside of canvas
        if !world
            .get_resource::<Canvas>()
            .unwrap()
            .is_position_inside(*mouse_position)
        {
            return;
        }

        let animation_manager = world.get_resource::<AnimationManager>().unwrap();

        let current_frame_index = animation_manager.current_frame_index();
        let frame_index_hover = animation_manager.frame_index_hover();

        if frame_index_hover.is_some() {
            let frame_index_hover = frame_index_hover.unwrap();

            let double_clicked = frame_index_hover == self.last_frame_index_hover
                && self.last_left_click_instant.elapsed().as_millis() < 500;
            self.last_left_click_instant = Instant::now();
            self.last_frame_index_hover = frame_index_hover;

            if current_frame_index != frame_index_hover {
                world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                    let current_file_entity = *tab_manager.current_tab_entity().unwrap();
                    tab_manager.current_tab_execute_anim_action(
                        world,
                        self,
                        AnimAction::SelectFrame(
                            current_file_entity,
                            frame_index_hover,
                            current_frame_index,
                        ),
                    );
                });
            }

            if double_clicked {
                let mut system_state: SystemState<(ResMut<Canvas>, ResMut<AnimationManager>)> =
                    SystemState::new(world);
                let (mut canvas, mut animation_manager) = system_state.get_mut(world);
                animation_manager.set_posing(&mut canvas);
            }
        }
    }

    pub(crate) fn handle_mouse_click_anim_posing(
        &mut self,
        world: &mut World,
        click_type: MouseButton,
        mouse_position: &Vec2,
    ) {
        // check if mouse position is outside of canvas
        if !world
            .get_resource::<Canvas>()
            .unwrap()
            .is_position_inside(*mouse_position)
        {
            return;
        }

        let selected_shape = self.selected_shape.map(|(_, shape)| shape);
        let hovered_shape = self.hovered_entity.map(|(_, shape)| shape);

        // click_type, selected_shape, hovered_shape, current_file_type
        match (click_type, selected_shape, hovered_shape) {
            (MouseButton::Left, Some(_), Some(shape)) => {
                match shape {
                    CanvasShape::Vertex | CanvasShape::Edge => {
                        // select hovered entity
                        world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                            tab_manager.current_tab_execute_anim_action(
                                world,
                                self,
                                AnimAction::SelectShape(self.hovered_entity),
                            );
                        });
                        return;
                    }
                    _ => {
                        // deselect vertex
                        world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                            tab_manager.current_tab_execute_anim_action(
                                world,
                                self,
                                AnimAction::SelectShape(None),
                            );
                        });
                        return;
                    }
                }
            }
            (MouseButton::Left, Some(CanvasShape::Vertex), None) => {
                // deselect vertex
                world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                    tab_manager.current_tab_execute_anim_action(
                        world,
                        self,
                        AnimAction::SelectShape(None),
                    );
                });
                return;
            }
            (MouseButton::Left, None, Some(CanvasShape::Vertex | CanvasShape::Edge)) => {
                world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                    tab_manager.current_tab_execute_anim_action(
                        world,
                        self,
                        AnimAction::SelectShape(self.hovered_entity),
                    );
                });
            }
            (MouseButton::Right, _, _) => {
                // deselect vertex
                world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                    tab_manager.current_tab_execute_anim_action(
                        world,
                        self,
                        AnimAction::SelectShape(None),
                    );
                });
            }
            _ => {}
        }
    }

    pub(crate) fn handle_mouse_drag_skelmesh(
        &mut self,
        world: &mut World,
        current_file_type: FileExtension,
        click_type: MouseButton,
        mouse_position: Vec2,
        delta: Vec2,
    ) {
        if !world.get_resource::<Canvas>().unwrap().has_focus() {
            return;
        }

        let shape_is_selected = self.selected_shape.is_some();
        let shape_can_drag = shape_is_selected
            && match self.selected_shape.unwrap().1 {
                CanvasShape::Vertex => true,
                CanvasShape::Edge => current_file_type != FileExtension::Mesh,
                _ => false,
            };

        if shape_is_selected && shape_can_drag {
            match click_type {
                MouseButton::Left => {
                    match self.selected_shape.unwrap() {
                        (vertex_2d_entity, CanvasShape::Vertex) => {
                            // move vertex
                            let Some(vertex_3d_entity) = world.get_resource::<VertexManager>().unwrap().vertex_entity_2d_to_3d(&vertex_2d_entity) else {
                                warn!(
                                    "Selected vertex entity: {:?} has no 3d counterpart",
                                    vertex_2d_entity
                                );
                                return;
                            };

                            let mut system_state: SystemState<(
                                Commands,
                                Client,
                                Res<CameraManager>,
                                ResMut<VertexManager>,
                                ResMut<Canvas>,
                                Query<(&Camera, &Projection)>,
                                Query<&Transform>,
                                Query<&mut Vertex3d>,
                            )> = SystemState::new(world);
                            let (
                                mut commands,
                                client,
                                camera_manager,
                                mut vertex_manager,
                                mut canvas,
                                camera_q,
                                transform_q,
                                mut vertex_3d_q,
                            ) = system_state.get_mut(world);

                            // check status
                            let auth_status = commands
                                .entity(vertex_3d_entity)
                                .authority(&client)
                                .unwrap();
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

                            vertex_manager.update_last_vertex_dragged(
                                vertex_2d_entity,
                                vertex_3d.as_vec3(),
                                new_3d_position,
                            );

                            vertex_3d.set_vec3(&new_3d_position);

                            // redraw
                            canvas.queue_resync_shapes();
                        }
                        (edge_2d_entity, CanvasShape::Edge) => {
                            let edge_3d_entity = world
                                .get_resource::<EdgeManager>()
                                .unwrap()
                                .edge_entity_2d_to_3d(&edge_2d_entity)
                                .unwrap();

                            let mut system_state: SystemState<(
                                Commands,
                                Client,
                                ResMut<EdgeManager>,
                                ResMut<Canvas>,
                                Query<&Transform>,
                                Query<&mut EdgeAngle>,
                            )> = SystemState::new(world);
                            let (
                                mut commands,
                                client,
                                mut edge_manager,
                                mut canvas,
                                transform_q,
                                mut edge_angle_q,
                            ) = system_state.get_mut(world);

                            // rotate edge angle
                            let auth_status =
                                commands.entity(edge_3d_entity).authority(&client).unwrap();
                            if !(auth_status.is_requested() || auth_status.is_granted()) {
                                // only continue to mutate if requested or granted authority over edge
                                info!("No authority over edge, skipping..");
                                return;
                            }

                            let edge_2d_transform = transform_q.get(edge_2d_entity).unwrap();
                            let start_pos = edge_2d_transform.translation.truncate();
                            let end_pos = get_2d_line_transform_endpoint(&edge_2d_transform);
                            let base_angle = angle_between(&start_pos, &end_pos);

                            let edge_angle_entity =
                                edge_manager.edge_get_base_circle_entity(&edge_3d_entity);
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

                            edge_manager.update_last_edge_dragged(
                                edge_2d_entity,
                                edge_angle.get_radians(),
                                new_angle,
                            );

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
            Self::handle_drag_empty_space(world, click_type, delta);
        }
    }

    pub(crate) fn handle_mouse_drag_anim_framing(
        &mut self,
        world: &mut World,
        click_type: MouseButton,
        _mouse_position: Vec2,
        delta: Vec2,
    ) {
        if !world.get_resource::<Canvas>().unwrap().has_focus() {
            return;
        }

        if click_type != MouseButton::Left {
            return;
        }

        world
            .get_resource_mut::<AnimationManager>()
            .unwrap()
            .handle_mouse_drag_anim_framing(delta.y);
    }

    pub(crate) fn handle_mouse_drag_anim_posing(
        &mut self,
        world: &mut World,
        click_type: MouseButton,
        mouse_position: Vec2,
        delta: Vec2,
    ) {
        if !world.get_resource::<Canvas>().unwrap().has_focus() {
            return;
        }

        let current_file_entity = *world
            .get_resource::<TabManager>()
            .unwrap()
            .current_tab_entity()
            .unwrap();

        let shape_is_selected = self.selected_shape.is_some();
        let shape_can_drag = shape_is_selected
            && match self.selected_shape.unwrap().1 {
                CanvasShape::RootVertex | CanvasShape::Vertex | CanvasShape::Edge => true,
                _ => false,
            };

        if shape_is_selected && shape_can_drag {
            match click_type {
                MouseButton::Left => {
                    match self.selected_shape.unwrap() {
                        (vertex_2d_entity, CanvasShape::Vertex) => {
                            // move vertex
                            let Some(vertex_3d_entity) = world.get_resource::<VertexManager>().unwrap().vertex_entity_2d_to_3d(&vertex_2d_entity) else {
                                warn!(
                                    "Selected vertex entity: {:?} has no 3d counterpart",
                                    vertex_2d_entity
                                );
                                return;
                            };

                            world.resource_scope(
                                |world, mut animation_manager: Mut<AnimationManager>| {
                                    animation_manager.drag_vertex(
                                        world,
                                        &current_file_entity,
                                        vertex_3d_entity,
                                        vertex_2d_entity,
                                        mouse_position,
                                    );
                                },
                            );
                        }
                        (edge_2d_entity, CanvasShape::Edge) => {
                            let edge_3d_entity = world
                                .get_resource::<EdgeManager>()
                                .unwrap()
                                .edge_entity_2d_to_3d(&edge_2d_entity)
                                .unwrap();

                            world.resource_scope(
                                |world, mut animation_manager: Mut<AnimationManager>| {
                                    animation_manager.drag_edge(
                                        world,
                                        &current_file_entity,
                                        edge_3d_entity,
                                        edge_2d_entity,
                                        mouse_position,
                                    );
                                },
                            );
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
            Self::handle_drag_empty_space(world, click_type, delta);
        }
    }

    fn handle_drag_empty_space(world: &mut World, click_type: MouseButton, delta: Vec2) {
        let mut system_state: SystemState<(ResMut<TabManager>, ResMut<CameraManager>)> =
            SystemState::new(world);
        let (mut tab_manager, mut camera_manager) = system_state.get_mut(world);

        let camera_state = &mut tab_manager.current_tab_state_mut().unwrap().camera_state;
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

    fn handle_mouse_scroll_wheel(world: &mut World, scroll_y: f32) {
        let mut system_state: SystemState<(ResMut<CameraManager>, ResMut<TabManager>)> =
            SystemState::new(world);
        let (mut camera_manager, mut tab_manager) = system_state.get_mut(world);

        camera_manager.camera_zoom(
            tab_manager.current_tab_camera_state_mut().unwrap(),
            scroll_y,
        );
    }

    fn handle_mouse_scroll_anim_framing(world: &mut World, scroll_y: f32) {
        let mut animation_manager = world.get_resource_mut::<AnimationManager>().unwrap();
        animation_manager.framing_handle_mouse_wheel(scroll_y);
    }

    pub(crate) fn sync_hover_shape_scale(
        &mut self,
        transform_q: &mut Query<(&mut Transform, Option<&LocalShape>)>,
        camera_3d_scale: f32,
    ) {
        let Some((hover_entity, shape)) = self.hovered_entity else {
            return;
        };
        if self.hovered_entity == self.selected_shape {
            return;
        }

        let hover_vertex_2d_scale = Vertex2d::HOVER_RADIUS * camera_3d_scale;
        let hover_edge_2d_scale = Edge2dLocal::HOVER_THICKNESS * camera_3d_scale;
        let hover_face_2d_scale = FaceIcon2d::HOVER_SIZE * camera_3d_scale;

        match shape {
            CanvasShape::RootVertex | CanvasShape::Vertex => {
                let (mut hover_vert_transform, _) = transform_q.get_mut(hover_entity).unwrap();
                hover_vert_transform.scale.x = hover_vertex_2d_scale;
                hover_vert_transform.scale.y = hover_vertex_2d_scale;
            }
            CanvasShape::Edge => {
                let (mut hover_edge_transform, _) = transform_q.get_mut(hover_entity).unwrap();
                hover_edge_transform.scale.y = hover_edge_2d_scale;
            }
            CanvasShape::Face => {
                let (mut hover_face_transform, _) = transform_q.get_mut(hover_entity).unwrap();
                hover_face_transform.scale.x = hover_face_2d_scale;
                hover_face_transform.scale.y = hover_face_2d_scale;
            }
        }
    }
}
