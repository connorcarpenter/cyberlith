use std::collections::HashMap;

use bevy_ecs::{
    entity::Entity,
    query::{With, Without},
    system::{Commands, Query, Res, ResMut, SystemState},
    world::{Mut, World},
};
use bevy_log::{info, warn};

use naia_bevy_client::{Client, CommandsExt, Instant};

use input::{InputEvent, Key, MouseButton};
use math::Vec2;
use render_api::{
    components::Transform,
    shapes::{distance_to_2d_line, get_2d_line_transform_endpoint},
};

use editor_proto::components::{IconEdge, IconFace, IconVertex};

use crate::app::{
    components::{
        Edge2dLocal, FaceIcon2d, IconEdgeLocal, IconLocalFace, IconVertexActionData, LocalShape,
        OwnedByFileLocal, Vertex2d,
    },
    plugin::Main,
    resources::{
        action::icon::IconAction,
        camera_manager::CameraManager,
        canvas::Canvas,
        icon_manager::{IconManager, IconShapeData},
        input::{CardinalDirection, InputManager},
        shape_data::CanvasShape,
        tab_manager::TabManager,
    },
};

pub struct IconInputManager;

impl IconInputManager {
    pub fn update_input(
        world: &mut World,
        current_file_entity: &Entity,
        icon_manager: &mut IconManager,
        input_actions: Vec<InputEvent>,
    ) {
        if icon_manager.is_meshing() {
            Self::update_input_meshing(world, current_file_entity, icon_manager, input_actions);
        } else {
            Self::update_input_framing(world, icon_manager, input_actions);
        }
    }

    // Meshing

    fn update_input_meshing(
        world: &mut World,
        current_file_entity: &Entity,
        icon_manager: &mut IconManager,
        input_actions: Vec<InputEvent>,
    ) {
        let wired = icon_manager.is_wired();
        for action in input_actions {
            match action {
                InputEvent::MouseClicked(click_type, mouse_position, _) => {
                    if wired {
                        Self::handle_mouse_click_meshing(
                            world,
                            current_file_entity,
                            icon_manager,
                            &mouse_position,
                            click_type,
                        )
                    }
                }
                InputEvent::MouseDragged(click_type, mouse_position, delta, _) => {
                    Self::handle_mouse_drag_meshing(
                        world,
                        icon_manager,
                        mouse_position,
                        delta,
                        click_type,
                    )
                }
                InputEvent::MouseMiddleScrolled(scroll_y) => {
                    InputManager::handle_mouse_scroll_wheel(world, scroll_y)
                }
                InputEvent::MouseMoved(_mouse_position) => {
                    if wired {
                        icon_manager.queue_resync_hover_ui();
                    }
                }
                InputEvent::MouseReleased(MouseButton::Left) => {
                    if wired {
                        icon_manager.reset_last_dragged_vertex(world)
                    }
                }
                InputEvent::KeyPressed(key, _) => match key {
                    Key::S | Key::W => {
                        icon_manager.handle_keypress_camera_controls(key);
                    }
                    Key::Delete => {
                        if wired {
                            Self::handle_delete_key_press_meshing(world, icon_manager);
                        }
                    }
                    Key::Insert => {
                        if wired {
                            Self::handle_insert_key_press_meshing(
                                world,
                                current_file_entity,
                                icon_manager,
                            );
                        }
                    }
                    Key::Escape => {
                        icon_manager.set_framing();
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    pub(crate) fn handle_insert_key_press_meshing(
        world: &mut World,
        current_file_entity: &Entity,
        icon_manager: &mut IconManager,
    ) {
        if icon_manager.selected_shape.is_some() {
            return;
        }
        let current_frame_entity = icon_manager
            .current_frame_entity(current_file_entity)
            .unwrap();
        world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
            tab_manager.current_tab_execute_icon_action(
                world,
                icon_manager,
                IconAction::CreateVertex(
                    IconVertexActionData::new(current_frame_entity, Vec::new(), Vec::new()),
                    Vec2::ZERO,
                    None,
                ),
            );
        })
    }

    pub(crate) fn handle_delete_key_press_meshing(
        world: &mut World,
        icon_manager: &mut IconManager,
    ) {
        match icon_manager.selected_shape {
            Some((vertex_entity, CanvasShape::Vertex)) => {
                icon_manager.handle_delete_vertex_action(world, &vertex_entity)
            }
            Some((edge_entity, CanvasShape::Edge)) => {
                let mut system_state: SystemState<(Commands, Client<Main>)> =
                    SystemState::new(world);
                let (mut commands, mut client) = system_state.get_mut(world);

                // check whether we can delete edge
                let auth_status = commands.entity(edge_entity).authority(&client).unwrap();
                if !auth_status.is_granted() && !auth_status.is_available() {
                    // do nothing, edge is not available
                    // TODO: queue for deletion? check before this?
                    warn!("Edge {:?} is not available for deletion!", edge_entity);
                    return;
                }

                let auth_status = commands.entity(edge_entity).authority(&client).unwrap();
                if !auth_status.is_granted() {
                    // request authority if needed
                    commands.entity(edge_entity).request_authority(&mut client);
                }

                world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                    tab_manager.current_tab_execute_icon_action(
                        world,
                        icon_manager,
                        IconAction::DeleteEdge(edge_entity, None),
                    );
                });

                icon_manager.selected_shape = None;
            }
            Some((local_face_entity, CanvasShape::Face)) => {
                let mut system_state: SystemState<(Commands, Client<Main>)> =
                    SystemState::new(world);
                let (mut commands, mut client) = system_state.get_mut(world);

                let net_face_entity = icon_manager
                    .face_entity_local_to_net(&local_face_entity)
                    .unwrap();

                // check whether we can delete edge
                let auth_status = commands.entity(net_face_entity).authority(&client).unwrap();
                if !auth_status.is_granted() && !auth_status.is_available() {
                    // do nothing, face is not available
                    // TODO: queue for deletion? check before this?
                    warn!(
                        "Face `{:?}` is not available for deletion!",
                        net_face_entity
                    );
                    return;
                }

                let auth_status = commands.entity(net_face_entity).authority(&client).unwrap();
                if !auth_status.is_granted() {
                    // request authority if needed
                    commands
                        .entity(net_face_entity)
                        .request_authority(&mut client);
                }

                world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                    tab_manager.current_tab_execute_icon_action(
                        world,
                        icon_manager,
                        IconAction::DeleteFace(local_face_entity),
                    );
                });

                icon_manager.selected_shape = None;
            }
            _ => {}
        }
    }

    pub(crate) fn handle_mouse_click_meshing(
        world: &mut World,
        current_file_entity: &Entity,
        icon_manager: &mut IconManager,
        mouse_position: &Vec2,
        click_type: MouseButton,
    ) {
        // check if mouse position is outside of canvas
        if !world
            .get_resource::<Canvas>()
            .unwrap()
            .is_position_inside(*mouse_position)
        {
            return;
        }

        let selected_shape = icon_manager.selected_shape.map(|(_, shape)| shape);
        let hovered_shape = icon_manager.hovered_entity.map(|(_, shape)| shape);

        // click_type, selected_shape, hovered_shape
        match (click_type, selected_shape, hovered_shape) {
            (MouseButton::Left, Some(CanvasShape::Vertex), Some(CanvasShape::Vertex)) => {
                Self::link_vertices(world, current_file_entity, icon_manager);
            }
            (MouseButton::Left, Some(CanvasShape::Vertex), None) => {
                // create new vertex
                let frame_entity = icon_manager
                    .current_frame_entity(current_file_entity)
                    .unwrap();
                let (vertex_entity, _) = icon_manager.selected_shape.unwrap();
                let vertex_type_data = IconVertexActionData::new(
                    frame_entity,
                    vec![(vertex_entity, None)],
                    Vec::new(),
                );

                // convert screen mouse to view mouse
                let mut system_state: SystemState<(Res<Canvas>, Query<&Transform>)> =
                    SystemState::new(world);
                let (canvas, transform_q) = system_state.get_mut(world);

                let Ok(camera_transform) = transform_q.get(icon_manager.camera_entity) else {
                    return;
                };
                let view_mouse_position =
                    IconManager::screen_to_view(&canvas, camera_transform, mouse_position);

                Self::handle_create_new_vertex(
                    world,
                    icon_manager,
                    &view_mouse_position,
                    vertex_type_data,
                );
            }
            (MouseButton::Left, _, _) => {
                if icon_manager.selected_shape != icon_manager.hovered_entity {
                    // select hovered shape (or None if there is no hovered shape)
                    world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                        tab_manager.current_tab_execute_icon_action(
                            world,
                            icon_manager,
                            IconAction::SelectShape(icon_manager.hovered_entity),
                        );
                    });
                }
            }
            (MouseButton::Right, Some(_), _) => {
                // deselect shape
                world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                    tab_manager.current_tab_execute_icon_action(
                        world,
                        icon_manager,
                        IconAction::SelectShape(None),
                    );
                });
            }
            _ => {}
        }
    }

    fn handle_create_new_vertex(
        world: &mut World,
        icon_manager: &mut IconManager,
        mouse_position: &Vec2,
        vertex_type_data: IconVertexActionData,
    ) {
        // spawn new vertex
        world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
            tab_manager.current_tab_execute_icon_action(
                world,
                icon_manager,
                IconAction::CreateVertex(vertex_type_data, *mouse_position, None),
            );
        });
    }

    fn link_vertices(
        world: &mut World,
        current_file_entity: &Entity,
        icon_manager: &mut IconManager,
    ) {
        // link vertices together
        let (vertex_entity_a, _) = icon_manager.selected_shape.unwrap();
        let (vertex_entity_b, _) = icon_manager.hovered_entity.unwrap();
        if vertex_entity_a == vertex_entity_b {
            return;
        }

        let frame_entity = icon_manager
            .current_frame_entity(current_file_entity)
            .unwrap();

        // check if edge already exists
        if icon_manager
            .edge_entity_from_vertices(vertex_entity_a, vertex_entity_b)
            .is_some()
        {
            // select vertex
            world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                tab_manager.current_tab_execute_icon_action(
                    world,
                    icon_manager,
                    IconAction::SelectShape(Some((vertex_entity_b, CanvasShape::Vertex))),
                );
            });
        } else {
            // create edge
            world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                tab_manager.current_tab_execute_icon_action(
                    world,
                    icon_manager,
                    IconAction::CreateEdge(
                        frame_entity,
                        vertex_entity_a,
                        vertex_entity_b,
                        (vertex_entity_b, CanvasShape::Vertex),
                        None,
                        None,
                    ),
                );
            });
        }
    }

    fn handle_mouse_drag_meshing(
        world: &mut World,
        icon_manager: &mut IconManager,
        mouse_position: Vec2,
        delta: Vec2,
        click_type: MouseButton,
    ) {
        if !world.get_resource::<TabManager>().unwrap().has_focus() {
            return;
        }

        match (click_type, icon_manager.selected_shape) {
            (MouseButton::Left, Some((vertex_entity, CanvasShape::Vertex))) => {
                Self::handle_vertex_drag(world, icon_manager, &vertex_entity, &mouse_position)
            }
            (_, _) => Self::handle_drag_empty_space(world, click_type, delta),
        }
    }

    pub(crate) fn handle_vertex_drag(
        world: &mut World,
        icon_manager: &mut IconManager,
        vertex_entity: &Entity,
        screen_mouse_position: &Vec2,
    ) {
        // move vertex

        let mut system_state: SystemState<(
            Commands,
            Client<Main>,
            ResMut<Canvas>,
            Query<&mut IconVertex>,
            Query<&Transform>,
        )> = SystemState::new(world);
        let (mut commands, client, mut canvas, mut vertex_q, transform_q) =
            system_state.get_mut(world);

        // check status
        let auth_status = commands.entity(*vertex_entity).authority(&client).unwrap();
        if !(auth_status.is_requested() || auth_status.is_granted()) {
            // only continue to mutate if requested or granted authority over vertex
            info!("No authority over vertex, skipping..");
            return;
        }

        let Ok(camera_transform) = transform_q.get(icon_manager.camera_entity) else {
            return;
        };
        let view_mouse_position =
            IconManager::screen_to_view(&canvas, camera_transform, screen_mouse_position);

        // set networked 3d vertex position
        let mut vertex = vertex_q.get_mut(*vertex_entity).unwrap();

        icon_manager.update_last_vertex_dragged(
            *vertex_entity,
            vertex.as_vec2(),
            view_mouse_position,
        );

        vertex.set_vec2(&view_mouse_position);

        // redraw
        canvas.queue_resync_shapes();
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
            _ => {}
        }
    }

    pub(crate) fn sync_mouse_hover_ui(
        icon_manager: &IconManager,
        world: &mut World,
        current_file_entity: &Entity,
        frame_entity: &Entity,
        mouse_position: &Vec2,
        camera_scale: f32,
    ) -> Option<(Entity, CanvasShape)> {
        let mut system_state: SystemState<(
            Query<&Transform>,
            Query<(Entity, &OwnedByFileLocal), With<IconVertex>>,
            Query<(Entity, &OwnedByFileLocal), With<IconEdgeLocal>>,
            Query<(Entity, &OwnedByFileLocal), With<IconLocalFace>>,
        )> = SystemState::new(world);
        let (transform_q, vertex_q, edge_q, face_q) = system_state.get_mut(world);

        let mut least_distance = f32::MAX;
        let mut least_entity = None;
        let mut is_hovering = false;

        Self::handle_vertex_hover(
            icon_manager,
            &transform_q,
            &vertex_q,
            current_file_entity,
            frame_entity,
            mouse_position,
            camera_scale,
            &mut least_distance,
            &mut least_entity,
            &mut is_hovering,
        );

        Self::handle_edge_hover(
            icon_manager,
            &transform_q,
            &edge_q,
            current_file_entity,
            frame_entity,
            mouse_position,
            camera_scale,
            &mut least_distance,
            &mut least_entity,
            &mut is_hovering,
        );

        Self::handle_face_hover(
            icon_manager,
            &transform_q,
            &face_q,
            current_file_entity,
            frame_entity,
            mouse_position,
            camera_scale,
            &mut least_distance,
            &mut least_entity,
            &mut is_hovering,
        );

        if is_hovering {
            least_entity
        } else {
            None
        }
    }

    fn handle_vertex_hover(
        icon_manager: &IconManager,
        transform_q: &Query<&Transform>,
        vertex_q: &Query<(Entity, &OwnedByFileLocal), With<IconVertex>>,
        current_file_entity: &Entity,
        frame_entity: &Entity,
        mouse_position: &Vec2,
        camera_scale: f32,
        least_distance: &mut f32,
        least_entity: &mut Option<(Entity, CanvasShape)>,
        is_hovering: &mut bool,
    ) {
        // check for vertices
        for (vertex_entity, owned_by_file) in vertex_q.iter() {
            if owned_by_file.file_entity != *current_file_entity {
                continue;
            }
            let Ok(vertex_transform) = transform_q.get(vertex_entity) else {
                continue;
            };
            let Some(vertex_frame_entity) = icon_manager.vertex_get_frame_entity(&vertex_entity)
            else {
                continue;
            };
            if vertex_frame_entity != *frame_entity {
                continue;
            }
            let vertex_position = vertex_transform.translation.truncate();
            let distance = vertex_position.distance(*mouse_position);
            if distance < *least_distance {
                *least_distance = distance;
                *least_entity = Some((vertex_entity, CanvasShape::Vertex));
            }
        }

        *is_hovering = *least_distance <= Vertex2d::DETECT_RADIUS * camera_scale;
    }

    fn handle_edge_hover(
        icon_manager: &IconManager,
        transform_q: &Query<&Transform>,
        edge_q: &Query<(Entity, &OwnedByFileLocal), With<IconEdgeLocal>>,
        current_file_entity: &Entity,
        frame_entity: &Entity,
        mouse_position: &Vec2,
        camera_scale: f32,
        least_distance: &mut f32,
        least_entity: &mut Option<(Entity, CanvasShape)>,
        is_hovering: &mut bool,
    ) {
        // check for edges
        if !*is_hovering {
            for (edge_entity, owned_by_file) in edge_q.iter() {
                if owned_by_file.file_entity != *current_file_entity {
                    continue;
                }
                let Some(edge_frame_entity) = icon_manager.edge_get_frame_entity(&edge_entity)
                else {
                    continue;
                };
                if edge_frame_entity != *frame_entity {
                    continue;
                }

                let edge_transform = transform_q.get(edge_entity).unwrap();
                let edge_start = edge_transform.translation.truncate();
                let edge_end = get_2d_line_transform_endpoint(&edge_transform);

                let distance = distance_to_2d_line(*mouse_position, edge_start, edge_end);
                if distance < *least_distance {
                    *least_distance = distance;
                    *least_entity = Some((edge_entity, CanvasShape::Edge));
                }
            }

            *is_hovering = *least_distance <= Edge2dLocal::DETECT_THICKNESS * camera_scale;
        }
    }

    fn handle_face_hover(
        icon_manager: &IconManager,
        transform_q: &Query<&Transform>,
        face_q: &Query<(Entity, &OwnedByFileLocal), With<IconLocalFace>>,
        current_file_entity: &Entity,
        frame_entity: &Entity,
        mouse_position: &Vec2,
        camera_scale: f32,
        least_distance: &mut f32,
        least_entity: &mut Option<(Entity, CanvasShape)>,
        is_hovering: &mut bool,
    ) {
        // check for faces
        if !*is_hovering {
            for (face_entity, owned_by_file) in face_q.iter() {
                if owned_by_file.file_entity != *current_file_entity {
                    continue;
                }
                let Some(face_frame_entity) = icon_manager.face_get_frame_entity(&face_entity)
                else {
                    continue;
                };
                if face_frame_entity != *frame_entity {
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

            *is_hovering = *least_distance <= FaceIcon2d::DETECT_RADIUS * camera_scale;
        }
    }

    // Framing

    fn update_input_framing(
        world: &mut World,
        icon_manager: &mut IconManager,
        input_actions: Vec<InputEvent>,
    ) {
        for action in input_actions {
            match action {
                InputEvent::MouseClicked(click_type, mouse_position, _) => {
                    Self::handle_mouse_click_framing(
                        world,
                        icon_manager,
                        click_type,
                        &mouse_position,
                    )
                }
                InputEvent::MouseDragged(click_type, _mouse_position, delta, _) => {
                    Self::handle_mouse_drag_framing(world, icon_manager, click_type, delta)
                }
                InputEvent::MouseMiddleScrolled(scroll_y) => {
                    Self::handle_mouse_scroll_framing(icon_manager, scroll_y)
                }
                InputEvent::MouseMoved(_mouse_position) => {
                    icon_manager.queue_resync_hover_ui();
                }
                InputEvent::KeyPressed(key, _) => match key {
                    Key::Delete => Self::handle_delete_frame(world, icon_manager),
                    Key::Insert => Self::handle_insert_frame(world, icon_manager),
                    Key::Space => Self::handle_play_pause(icon_manager),
                    Key::Enter => {
                        icon_manager.set_meshing();
                    }
                    Key::S | Key::W => {
                        icon_manager.handle_keypress_camera_controls(key);
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
                        if let Some((prev_index, next_index)) =
                            icon_manager.framing_navigate(&current_file_entity, dir)
                        {
                            world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                                tab_manager.current_tab_execute_icon_action(
                                    world,
                                    icon_manager,
                                    IconAction::SelectFrame(
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

    fn handle_mouse_click_framing(
        world: &mut World,
        icon_manager: &mut IconManager,
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

        let current_frame_index = icon_manager.current_frame_index();
        let frame_index_hover = icon_manager.frame_index_hover();

        if frame_index_hover.is_some() {
            let frame_index_hover = frame_index_hover.unwrap();

            let double_clicked = frame_index_hover == icon_manager.last_frame_index_hover
                && icon_manager.last_left_click_instant.elapsed().as_millis() < 500;
            icon_manager.last_left_click_instant = Instant::now();
            icon_manager.last_frame_index_hover = frame_index_hover;

            if frame_index_hover == 0 {
                // clicked preview frame .. do nothing!
            } else {
                if current_frame_index != frame_index_hover - 1 {
                    world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                        let current_file_entity = *tab_manager.current_tab_entity().unwrap();
                        tab_manager.current_tab_execute_icon_action(
                            world,
                            icon_manager,
                            IconAction::SelectFrame(
                                current_file_entity,
                                frame_index_hover - 1,
                                current_frame_index,
                            ),
                        );
                    });
                }

                if double_clicked {
                    icon_manager.set_meshing();
                }
            }
        }
    }

    fn handle_mouse_drag_framing(
        world: &mut World,
        icon_manager: &mut IconManager,
        click_type: MouseButton,
        delta: Vec2,
    ) {
        if !world.get_resource::<TabManager>().unwrap().has_focus() {
            return;
        }

        if click_type != MouseButton::Left {
            return;
        }

        icon_manager.handle_mouse_drag_framing(delta.y);
    }

    fn handle_mouse_scroll_framing(icon_manager: &mut IconManager, scroll_y: f32) {
        icon_manager.framing_handle_mouse_wheel(scroll_y);
    }

    pub(crate) fn handle_insert_frame(world: &mut World, icon_manager: &mut IconManager) {
        world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
            let current_file_entity = *tab_manager.current_tab_entity().unwrap();
            let current_frame_index = icon_manager.current_frame_index();

            // copy all shapes from current frame
            let copied_shapes = Self::pack_shape_data(world, icon_manager, &current_file_entity);

            // execute insertion
            tab_manager.current_tab_execute_icon_action(
                world,
                icon_manager,
                IconAction::InsertFrame(
                    current_file_entity,
                    current_frame_index + 1,
                    Some(copied_shapes),
                ),
            );
        });
    }

    pub fn pack_shape_data(
        world: &mut World,
        icon_manager: &mut IconManager,
        current_file_entity: &Entity,
    ) -> Vec<IconShapeData> {
        let mut copied_shapes = Vec::new();
        let current_frame_entity = icon_manager
            .current_frame_entity(&current_file_entity)
            .unwrap();

        let mut system_state: SystemState<(
            Client<Main>,
            Query<(Entity, &IconVertex), Without<LocalShape>>,
            Query<(Entity, &IconEdge), Without<LocalShape>>,
            Query<&IconFace>,
        )> = SystemState::new(world);
        let (client, vertex_q, edge_q, face_q) = system_state.get_mut(world);

        // vertices
        let mut vertex_map = HashMap::new();
        let mut vertex_index: usize = 0;

        for (vertex_entity, vertex) in vertex_q.iter() {
            if vertex.frame_entity.get(&client).unwrap() == current_frame_entity {
                copied_shapes.push(IconShapeData::Vertex(vertex.x(), vertex.y()));
                vertex_map.insert(vertex_entity, vertex_index);
                vertex_index += 1;
            }
        }

        // edges
        let mut edge_map = HashMap::new();
        let mut edge_index: usize = 0;

        for (edge_entity, edge) in edge_q.iter() {
            if edge.frame_entity.get(&client).unwrap() != current_frame_entity {
                continue;
            }

            let vertex_a_entity = edge.start.get(&client).unwrap();
            let vertex_b_entity = edge.end.get(&client).unwrap();
            let vertex_a_index = *vertex_map.get(&vertex_a_entity).unwrap();
            let vertex_b_index = *vertex_map.get(&vertex_b_entity).unwrap();

            copied_shapes.push(IconShapeData::Edge(vertex_a_index, vertex_b_index));
            edge_map.insert(edge_entity, edge_index);
            edge_index += 1;
        }

        // faces
        for face in face_q.iter() {
            if face.frame_entity.get(&client).unwrap() != current_frame_entity {
                continue;
            }

            let palette_color_entity = face.palette_color_entity.get(&client).unwrap();

            let vertex_a_entity = face.vertex_a.get(&client).unwrap();
            let vertex_b_entity = face.vertex_b.get(&client).unwrap();
            let vertex_c_entity = face.vertex_c.get(&client).unwrap();
            let vertex_a_index = *vertex_map.get(&vertex_a_entity).unwrap();
            let vertex_b_index = *vertex_map.get(&vertex_b_entity).unwrap();
            let vertex_c_index = *vertex_map.get(&vertex_c_entity).unwrap();

            copied_shapes.push(IconShapeData::Face(
                palette_color_entity,
                vertex_a_index,
                vertex_b_index,
                vertex_c_index,
            ));
        }
        copied_shapes
    }

    pub(crate) fn handle_delete_frame(world: &mut World, icon_manager: &mut IconManager) {
        let Some(current_file_entity) = world
            .get_resource::<TabManager>()
            .unwrap()
            .current_tab_entity()
        else {
            return;
        };
        let current_file_entity = *current_file_entity;

        let mut system_state: SystemState<(Commands, Client<Main>)> = SystemState::new(world);
        let (mut commands, client) = system_state.get_mut(world);

        // delete vertex
        let Some(current_frame_entity) = icon_manager.current_frame_entity(&current_file_entity)
        else {
            return;
        };

        // check whether we can delete vertex
        let auth_status = commands
            .entity(current_frame_entity)
            .authority(&client)
            .unwrap();
        if !auth_status.is_granted() && !auth_status.is_available() {
            // do nothing, file is not available
            // TODO: queue for deletion? check before this?
            warn!(
                "Frame `{:?}` is not available for deletion!",
                current_frame_entity
            );
            return;
        }

        let current_frame_index = icon_manager.current_frame_index();

        world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
            tab_manager.current_tab_execute_icon_action(
                world,
                icon_manager,
                IconAction::DeleteFrame(current_file_entity, current_frame_index),
            );
        });
    }

    fn handle_play_pause(icon_manager: &mut IconManager) {
        if icon_manager.preview_is_playing() {
            icon_manager.preview_pause();
        } else {
            icon_manager.preview_play();
        }
    }
}
