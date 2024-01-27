use bevy_ecs::{
    entity::Entity,
    query::{With, Without},
    system::{Commands, Query, Res, ResMut, Resource, SystemState},
    world::{Mut, World},
};
use bevy_log::{info, warn};

use naia_bevy_client::{Client, CommandsExt, Instant};

use input::{InputAction, Key, MouseButton};
use math::{Vec2, Vec3};
use render_api::{
    components::{Camera, Projection, Transform, Visibility},
    shapes::{distance_to_2d_line, get_2d_line_transform_endpoint, set_2d_line_transform},
};

use editor_proto::components::{FileExtension, ShapeName, Vertex3d, VertexRoot};

use crate::app::{
    components::{
        Edge2dLocal, FaceIcon2d, LocalShape, SelectCircle, SelectTriangle, Vertex2d, VertexTypeData,
    },
    get_new_3d_position,
    plugin::Main,
    resources::{
        action::shape::ShapeAction,
        camera_manager::CameraAngle,
        camera_manager::CameraManager,
        canvas::Canvas,
        edge_manager::EdgeManager,
        file_manager::FileManager,
        icon_manager::IconManager,
        input::{
            icon_input_manager::IconInputManager, mesh_input_manager::MeshInputManager,
            model_input_manager::ModelInputManager, skel_input_manager::SkelInputManager,
            skin_input_manager::SkinInputManager, AnimInputManager,
        },
        shape_data::CanvasShape,
        tab_manager::TabManager,
        vertex_manager::VertexManager,
    },
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
    pub(crate) selected_shape: Option<(Entity, CanvasShape)>,
    pub select_circle_entity: Option<Entity>,
    pub select_triangle_entity: Option<Entity>,
    pub select_line_entity: Option<Entity>,

    //doubleclick
    pub(crate) last_left_click_instant: Instant,
    pub(crate) last_frame_index_hover: usize, //TODO: move this to AnimInputManager?

    vertex_dragging_enabled: bool,
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

            vertex_dragging_enabled: true,
        }
    }
}

impl InputManager {
    pub fn dragging_is_enabled(&self) -> bool {
        self.vertex_dragging_enabled
    }

    pub fn toggle_dragging_is_enabled(&mut self) {
        self.vertex_dragging_enabled = !self.vertex_dragging_enabled;
    }

    pub fn update_input(&mut self, input_actions: Vec<InputAction>, world: &mut World) {
        let Some(current_file_entity) = world
            .get_resource::<TabManager>()
            .unwrap()
            .current_tab_entity() else {
            return;
        };
        let current_file_entity = *current_file_entity;
        let current_file_type = world
            .get_resource::<FileManager>()
            .unwrap()
            .get_file_type(&current_file_entity);
        match current_file_type {
            FileExtension::Skel => SkelInputManager::update_input(world, self, input_actions),
            FileExtension::Mesh => MeshInputManager::update_input(world, self, input_actions),
            FileExtension::Anim => AnimInputManager::update_input(world, self, input_actions),
            FileExtension::Skin => SkinInputManager::update_input(world, self, input_actions),
            FileExtension::Model | FileExtension::Scene => {
                ModelInputManager::update_input(world, self, &current_file_type, input_actions)
            }
            FileExtension::Icon => {
                world.resource_scope(|world, mut icon_manager: Mut<IconManager>| {
                    IconInputManager::update_input(
                        world,
                        &current_file_entity,
                        &mut icon_manager,
                        input_actions,
                    );
                });
            }
            _ => {}
        }
    }

    pub(crate) fn sync_mouse_hover_ui(
        &mut self,
        world: &mut World,
        file_ext: FileExtension,
        current_file_entity: &Entity,
        mouse_position: &Vec2,
    ) {
        if !self.resync_hover {
            return;
        }
        self.resync_hover = false;

        let Some(current_tab_state) = world.get_resource::<TabManager>().unwrap().current_tab_state() else {
            return;
        };
        let camera_state = &current_tab_state.camera_state;
        let camera_3d_scale = camera_state.camera_3d_scale();

        let next_hovered_entity = match file_ext {
            FileExtension::Skel => {
                SkelInputManager::sync_mouse_hover_ui(world, camera_3d_scale, mouse_position)
            }
            FileExtension::Mesh => {
                MeshInputManager::sync_mouse_hover_ui(world, camera_3d_scale, mouse_position)
            }
            FileExtension::Anim => AnimInputManager::sync_mouse_hover_ui(
                world,
                current_file_entity,
                camera_3d_scale,
                mouse_position,
            ),
            FileExtension::Skin => {
                SkinInputManager::sync_mouse_hover_ui(world, camera_3d_scale, mouse_position)
            }
            FileExtension::Model | FileExtension::Scene => ModelInputManager::sync_mouse_hover_ui(
                world,
                &file_ext,
                current_file_entity,
                camera_3d_scale,
                mouse_position,
            ),
            FileExtension::Icon => {
                return;
            }
            _ => {
                return;
            }
        };

        // define old and new hovered states
        self.sync_hover_shape_scale(world, camera_3d_scale);

        // hover state did not change
        if self.hovered_entity == next_hovered_entity {
            return;
        }

        // apply
        self.hovered_entity = next_hovered_entity;
        world
            .get_resource_mut::<Canvas>()
            .unwrap()
            .queue_resync_shapes_light();
    }

    pub(crate) fn handle_vertex_hover(
        transform_q: &Query<&Transform>,
        visibility_q: &Query<&Visibility>,
        vertex_2d_q: &Query<(Entity, Option<&VertexRoot>), (With<Vertex2d>, Without<LocalShape>)>,
        anim_opt: Option<(&VertexManager, &Query<&ShapeName>)>,
        camera_3d_scale: f32,
        mouse_position: &Vec2,
        least_distance: &mut f32,
        least_entity: &mut Option<(Entity, CanvasShape)>,
        is_hovering: &mut bool,
    ) {
        // check for vertices
        for (vertex_2d_entity, root_opt) in vertex_2d_q.iter() {
            let Ok(visibility) = visibility_q.get(vertex_2d_entity) else {
                panic!("Vertex entity has no Visibility");
            };
            if !visibility.visible {
                continue;
            }

            Self::hover_check_vertex(
                transform_q,
                anim_opt,
                mouse_position,
                least_distance,
                least_entity,
                &vertex_2d_entity,
                root_opt,
            );
        }

        *is_hovering = *least_distance <= (Vertex2d::DETECT_RADIUS * camera_3d_scale);
    }

    pub fn hover_check_vertex(
        transform_q: &Query<&Transform>,
        anim_opt: Option<(&VertexManager, &Query<&ShapeName>)>,
        mouse_position: &Vec2,
        least_distance: &mut f32,
        least_entity: &mut Option<(Entity, CanvasShape)>,
        vertex_2d_entity: &Entity,
        root_opt: Option<&VertexRoot>,
    ) {
        if let Some((vertex_manager, shape_name_q)) = anim_opt {
            // don't hover over disabled vertices in Anim mode
            let vertex_3d_entity = vertex_manager
                .vertex_entity_2d_to_3d(&vertex_2d_entity)
                .unwrap();
            let Ok(shape_name) = shape_name_q.get(vertex_3d_entity) else { return; };
            let shape_name = shape_name.value.as_str();
            if shape_name.len() == 0 {
                return;
            }
        }

        let Ok(vertex_transform) = transform_q.get(*vertex_2d_entity) else {
            return;
        };
        let vertex_position = vertex_transform.translation.truncate();
        let distance = vertex_position.distance(*mouse_position);
        if distance < *least_distance {
            *least_distance = distance;

            let shape = match root_opt {
                Some(_) => CanvasShape::RootVertex,
                None => CanvasShape::Vertex,
            };

            *least_entity = Some((*vertex_2d_entity, shape));
        }
    }

    pub(crate) fn handle_edge_hover(
        transform_q: &Query<&Transform>,
        visibility_q: &Query<&Visibility>,
        edge_2d_q: &Query<(Entity, &Edge2dLocal), Without<LocalShape>>,
        anim_opt: Option<(&EdgeManager, &Query<&ShapeName>)>,
        camera_3d_scale: f32,
        mouse_position: &Vec2,
        least_distance: &mut f32,
        least_entity: &mut Option<(Entity, CanvasShape)>,
        is_hovering: &mut bool,
    ) {
        // check for edges
        if !*is_hovering {
            for (edge_2d_entity, _) in edge_2d_q.iter() {
                // check visibility
                let Ok(visibility) = visibility_q.get(edge_2d_entity) else {
                    panic!("entity has no Visibility");
                };
                if !visibility.visible {
                    continue;
                }

                Self::hover_check_edge(
                    transform_q,
                    anim_opt,
                    mouse_position,
                    least_distance,
                    least_entity,
                    &edge_2d_entity,
                );
            }

            *is_hovering = *least_distance <= (Edge2dLocal::DETECT_THICKNESS * camera_3d_scale);
        }
    }

    pub fn hover_check_edge(
        transform_q: &Query<&Transform>,
        anim_opt: Option<(&EdgeManager, &Query<&ShapeName>)>,
        mouse_position: &Vec2,
        least_distance: &mut f32,
        least_entity: &mut Option<(Entity, CanvasShape)>,
        edge_2d_entity: &Entity,
    ) {
        if let Some((edge_manager, shape_name_q)) = anim_opt {
            let edge_3d_entity = edge_manager.edge_entity_2d_to_3d(&edge_2d_entity).unwrap();
            let (_, end_vertex_3d_entity) = edge_manager.edge_get_endpoints(&edge_3d_entity);
            let Ok(shape_name) = shape_name_q.get(end_vertex_3d_entity) else { return; };
            let shape_name = shape_name.value.as_str();
            if shape_name.len() == 0 {
                return;
            }
        }

        if let Ok(edge_transform) = transform_q.get(*edge_2d_entity) {
            let edge_start = edge_transform.translation.truncate();
            let edge_end = get_2d_line_transform_endpoint(&edge_transform);

            let distance = distance_to_2d_line(*mouse_position, edge_start, edge_end);
            if distance < *least_distance {
                *least_distance = distance;
                *least_entity = Some((*edge_2d_entity, CanvasShape::Edge));
            }
        }
    }

    pub(crate) fn handle_face_hover(
        transform_q: &Query<&Transform>,
        visibility_q: &Query<&Visibility>,
        face_2d_q: &Query<Entity, With<FaceIcon2d>>,
        mouse_position: &Vec2,
        camera_3d_scale: f32,
        least_distance: &mut f32,
        least_entity: &mut Option<(Entity, CanvasShape)>,
        is_hovering: &mut bool,
    ) {
        // check for faces
        if !*is_hovering {
            for face_entity in face_2d_q.iter() {
                // check tab ownership, skip faces from other tabs
                let Ok(visibility) = visibility_q.get(face_entity) else {
                    panic!("entity has no Visibility");
                };
                if !visibility.visible {
                    continue;
                }

                let face_transform = transform_q.get(face_entity).unwrap();
                let face_position = face_transform.translation.truncate();
                let distance = face_position.distance(*mouse_position);
                if distance < *least_distance {
                    *least_distance = distance;

                    *least_entity = Some((face_entity, CanvasShape::Face));
                }
            }

            *is_hovering = *least_distance <= (FaceIcon2d::DETECT_RADIUS * camera_3d_scale);
        }
    }

    pub(crate) fn reset_last_dragged_vertex(&mut self, world: &mut World) {
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

    pub(crate) fn handle_edge_angle_visibility_toggle(world: &mut World) {
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

    pub(crate) fn handle_keypress_camera_controls(world: &mut World, key: Key) {
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

    pub(crate) fn handle_create_new_vertex(
        world: &mut World,
        input_manager: &mut InputManager,
        mouse_position: &Vec2,
        vertex_2d_entity: Entity,
        vertex_type_data: VertexTypeData,
    ) {
        // create new vertex

        let mut system_state: SystemState<(
            Res<CameraManager>,
            Query<(&Camera, &Projection)>,
            Query<&Transform>,
        )> = SystemState::new(world);
        let (camera_manager, camera_q, transform_q) = system_state.get_mut(world);

        let new_3d_position = get_new_3d_position(
            &camera_manager,
            &camera_q,
            &transform_q,
            mouse_position,
            &vertex_2d_entity,
        );

        // spawn new vertex
        world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
            tab_manager.current_tab_execute_shape_action(
                world,
                input_manager,
                ShapeAction::CreateVertex(vertex_type_data, new_3d_position, None),
            );
        });
    }

    pub(crate) fn sync_selection_ui(
        &mut self,
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
            Some((selected_vertex_entity, CanvasShape::RootVertex | CanvasShape::Vertex)) => {
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
                        1.0,
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
                    select_circle_transform.translation.z += 1.0;
                }

                let current_file_entity = tab_manager.current_tab_entity().unwrap();
                let current_file_type = file_manager.get_file_type(&current_file_entity);

                select_shape_visibilities[0].visible = true; // select circle is visible
                select_shape_visibilities[1].visible = false; // no select triangle visible
                select_shape_visibilities[2].visible = match current_file_type {
                    FileExtension::Anim => false,
                    FileExtension::Skel => tab_manager.has_focus() && self.vertex_dragging_enabled,
                    _ => tab_manager.has_focus(),
                };
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
                    select_line_transform.translation.z -= 1.0;
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

    pub(crate) fn handle_delete_vertex_action(
        &mut self,
        world: &mut World,
        vertex_2d_entity: &Entity,
    ) {
        let mut system_state: SystemState<(Commands, Client<Main>, Res<VertexManager>)> =
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

    pub(crate) fn handle_vertex_drag(
        world: &mut World,
        vertex_2d_entity: &Entity,
        mouse_position: &Vec2,
    ) {
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
            Client<Main>,
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

        let new_3d_position = get_new_3d_position(
            &camera_manager,
            &camera_q,
            &transform_q,
            &mouse_position,
            &vertex_2d_entity,
        );

        // set networked 3d vertex position
        let mut vertex_3d = vertex_3d_q.get_mut(vertex_3d_entity).unwrap();

        vertex_manager.update_last_vertex_dragged(
            *vertex_2d_entity,
            vertex_3d.as_vec3(),
            new_3d_position,
        );

        vertex_3d.set_vec3(&new_3d_position);

        // redraw
        canvas.queue_resync_shapes();
    }

    pub(crate) fn handle_drag_empty_space(world: &mut World, click_type: MouseButton, delta: Vec2) {
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

    pub(crate) fn handle_mouse_scroll_wheel(world: &mut World, scroll_y: f32) {
        let mut system_state: SystemState<(ResMut<CameraManager>, ResMut<TabManager>)> =
            SystemState::new(world);
        let (mut camera_manager, mut tab_manager) = system_state.get_mut(world);

        camera_manager.camera_zoom(
            tab_manager.current_tab_camera_state_mut().unwrap(),
            scroll_y,
        );
    }

    pub(crate) fn sync_hover_shape_scale(&mut self, world: &mut World, camera_3d_scale: f32) {
        let mut system_state: SystemState<Query<(&mut Transform, Option<&LocalShape>)>> =
            SystemState::new(world);
        let mut transform_q = system_state.get_mut(world);

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
