use bevy_ecs::{
    entity::Entity,
    query::With,
    system::{ResMut, Commands, Query, Res, SystemState},
    world::{Mut, World},
};
use bevy_log::{info, warn};

use naia_bevy_client::{Client, CommandsExt};

use input::{InputAction, Key, MouseButton};
use math::Vec2;
use render_api::{components::Transform, shapes::{distance_to_2d_line, get_2d_line_transform_endpoint}};

use vortex_proto::components::IconVertex;

use crate::app::{
    components::{OwnedByFileLocal, IconVertexActionData, Edge2dLocal, FaceIcon2d, IconEdgeLocal, IconLocalFace, Vertex2d},
    resources::{
        action::icon::IconAction, canvas::Canvas,
        input::InputManager, shape_data::CanvasShape,
        tab_manager::TabManager, icon_manager::IconManager,
    },
};

pub struct IconInputManager;

impl IconInputManager {
    pub fn update_input(
        world: &mut World,
        input_manager: &mut InputManager,
        input_actions: Vec<InputAction>,
    ) {
        for action in input_actions {
            match action {
                InputAction::MouseClick(click_type, mouse_position) => {
                    Self::handle_mouse_click(world, input_manager, &mouse_position, click_type)
                }
                InputAction::MouseDragged(click_type, mouse_position, delta) => {
                    Self::handle_mouse_drag(world, input_manager, mouse_position, delta, click_type)
                }
                InputAction::MiddleMouseScroll(scroll_y) => {
                    InputManager::handle_mouse_scroll_wheel(world, scroll_y)
                }
                InputAction::MouseMoved => {
                    input_manager.queue_resync_hover_ui();
                    input_manager.queue_resync_selection_ui();
                }
                InputAction::MouseRelease(MouseButton::Left) => {
                    world.resource_scope(|world, mut icon_manager: Mut<IconManager>| {
                        icon_manager.reset_last_dragged_vertex(world, input_manager)
                    });
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
                    | Key::PageDown => InputManager::handle_keypress_camera_controls(world, key),
                    Key::Delete => Self::handle_delete_key_press(input_manager, world),
                    Key::Insert => Self::handle_insert_key_press(world, input_manager),
                    _ => {}
                },
                _ => {}
            }
        }
    }

    pub(crate) fn handle_insert_key_press(world: &mut World, input_manager: &mut InputManager) {
        if input_manager.selected_shape.is_some() {
            return;
        }
        world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
            tab_manager.current_tab_execute_icon_action(
                world,
                input_manager,
                IconAction::CreateVertex(
                    IconVertexActionData::new(Vec::new(), Vec::new()),
                    Vec2::ZERO,
                    None,
                ),
            );
        })
    }

    pub(crate) fn handle_delete_key_press(input_manager: &mut InputManager, world: &mut World) {
        match input_manager.selected_shape {
            Some((vertex_entity, CanvasShape::Vertex)) => {
                input_manager.handle_delete_vertex_action(world, &vertex_entity)
            }
            Some((edge_entity, CanvasShape::Edge)) => {
                let mut system_state: SystemState<(Commands, Client)> =
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
                    commands
                        .entity(edge_entity)
                        .request_authority(&mut client);
                }

                world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                    tab_manager.current_tab_execute_icon_action(
                        world,
                        input_manager,
                        IconAction::DeleteEdge(edge_entity, None),
                    );
                });

                input_manager.selected_shape = None;
            }
            Some((local_face_entity, CanvasShape::Face)) => {
                let mut system_state: SystemState<(Commands, Client, Res<IconManager>)> =
                    SystemState::new(world);
                let (mut commands, mut client, icon_manager) = system_state.get_mut(world);

                let net_face_entity = icon_manager.face_entity_local_to_net(&local_face_entity).unwrap();

                // check whether we can delete edge
                let auth_status = commands.entity(net_face_entity).authority(&client).unwrap();
                if !auth_status.is_granted() && !auth_status.is_available() {
                    // do nothing, face is not available
                    // TODO: queue for deletion? check before this?
                    warn!("Face `{:?}` is not available for deletion!", net_face_entity);
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
                        input_manager,
                        IconAction::DeleteFace(local_face_entity),
                    );
                });

                input_manager.selected_shape = None;
            }
            _ => {}
        }
    }

    pub(crate) fn handle_mouse_click(
        world: &mut World,
        input_manager: &mut InputManager,
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

        let selected_shape = input_manager.selected_shape.map(|(_, shape)| shape);
        let hovered_shape = input_manager.hovered_entity.map(|(_, shape)| shape);

        // click_type, selected_shape, hovered_shape
        match (click_type, selected_shape, hovered_shape) {
            (MouseButton::Left, Some(CanvasShape::Vertex), Some(CanvasShape::Vertex)) => {
                Self::link_vertices(world, input_manager);
            }
            (MouseButton::Left, Some(CanvasShape::Vertex), None) => {
                // create new vertex
                let (vertex_entity, _) = input_manager.selected_shape.unwrap();
                let vertex_type_data =
                    IconVertexActionData::new(vec![(vertex_entity, None)], Vec::new());
                Self::handle_create_new_vertex(
                    world,
                    input_manager,
                    &mouse_position,
                    vertex_type_data,
                );
            }
            (MouseButton::Left, _, _) => {
                // select hovered shape (or None if there is no hovered shape)
                world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                    tab_manager.current_tab_execute_icon_action(
                        world,
                        input_manager,
                        IconAction::SelectShape(input_manager.hovered_entity),
                    );
                });
            }
            (MouseButton::Right, Some(_), _) => {
                // deselect shape
                world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                    tab_manager.current_tab_execute_icon_action(
                        world,
                        input_manager,
                        IconAction::SelectShape(None),
                    );
                });
            }
            _ => {}
        }
    }

    fn handle_create_new_vertex(
        world: &mut World,
        input_manager: &mut InputManager,
        mouse_position: &Vec2,
        vertex_type_data: IconVertexActionData,
    ) {
        // spawn new vertex
        world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
            tab_manager.current_tab_execute_icon_action(
                world,
                input_manager,
                IconAction::CreateVertex(vertex_type_data, *mouse_position, None),
            );
        });
    }

    fn link_vertices(world: &mut World, input_manager: &mut InputManager) {
        let mut system_state: SystemState<Res<IconManager>> =
            SystemState::new(world);
        let icon_manager = system_state.get_mut(world);

        // link vertices together
        let (vertex_entity_a, _) = input_manager.selected_shape.unwrap();
        let (vertex_entity_b, _) = input_manager.hovered_entity.unwrap();
        if vertex_entity_a == vertex_entity_b {
            return;
        }

        // check if edge already exists
        if icon_manager
            .edge_entity_from_vertices(vertex_entity_a, vertex_entity_b)
            .is_some()
        {
            // select vertex
            world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                tab_manager.current_tab_execute_icon_action(
                    world,
                    input_manager,
                    IconAction::SelectShape(Some((vertex_entity_b, CanvasShape::Vertex))),
                );
            });
        } else {
            // create edge
            world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                tab_manager.current_tab_execute_icon_action(
                    world,
                    input_manager,
                    IconAction::CreateEdge(
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

    pub(crate) fn handle_mouse_drag(
        world: &mut World,
        input_manager: &mut InputManager,
        mouse_position: Vec2,
        delta: Vec2,
        click_type: MouseButton,
    ) {
        if !world.get_resource::<TabManager>().unwrap().has_focus() {
            return;
        }

        match (click_type, input_manager.selected_shape) {
            (MouseButton::Left, Some((vertex_entity, CanvasShape::Vertex))) => {
                Self::handle_vertex_drag(world, &vertex_entity, &mouse_position)
            }
            (_, _) => InputManager::handle_drag_empty_space(world, click_type, delta),
        }
    }

    pub(crate) fn handle_vertex_drag(
        world: &mut World,
        vertex_entity: &Entity,
        mouse_position: &Vec2,
    ) {
        // move vertex

        let mut system_state: SystemState<(
            Commands,
            Client,
            ResMut<IconManager>,
            ResMut<Canvas>,
            Query<&mut IconVertex>,
        )> = SystemState::new(world);
        let (
            mut commands,
            client,
            mut icon_manager,
            mut canvas,
            mut vertex_q,
        ) = system_state.get_mut(world);

        // check status
        let auth_status = commands
            .entity(*vertex_entity)
            .authority(&client)
            .unwrap();
        if !(auth_status.is_requested() || auth_status.is_granted()) {
            // only continue to mutate if requested or granted authority over vertex
            info!("No authority over vertex, skipping..");
            return;
        }

        // set networked 3d vertex position
        let mut vertex = vertex_q.get_mut(*vertex_entity).unwrap();

        icon_manager.update_last_vertex_dragged(
            *vertex_entity,
            vertex.as_vec2(),
            *mouse_position,
        );

        vertex.set_vec2(mouse_position);

        // redraw
        canvas.queue_resync_shapes();
    }

    pub(crate) fn sync_mouse_hover_ui(
        world: &mut World,
        current_file_entity: &Entity,
        mouse_position: &Vec2,
    ) -> Option<(Entity, CanvasShape)> {
        let mut system_state: SystemState<(
            Query<&Transform>,
            Query<(Entity, &OwnedByFileLocal), With<IconVertex>>,
            Query<(Entity, &OwnedByFileLocal), With<IconEdgeLocal>>,
            Query<(Entity, &OwnedByFileLocal), With<IconLocalFace>>,
        )> = SystemState::new(world);
        let (
            transform_q,
            vertex_q,
            edge_q,
            face_q
        ) = system_state.get_mut(world);

        let mut least_distance = f32::MAX;
        let mut least_entity = None;
        let mut is_hovering = false;

        Self::handle_vertex_hover(
            &transform_q,
            &vertex_q,
            current_file_entity,
            mouse_position,
            &mut least_distance,
            &mut least_entity,
            &mut is_hovering,
        );

        Self::handle_edge_hover(
            &transform_q,
            &edge_q,
            current_file_entity,
            mouse_position,
            &mut least_distance,
            &mut least_entity,
            &mut is_hovering,
        );

        Self::handle_face_hover(
            &transform_q,
            &face_q,
            current_file_entity,
            mouse_position,
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
        transform_q: &Query<&Transform>,
        vertex_q: &Query<(Entity, &OwnedByFileLocal), With<IconVertex>>,
        current_file_entity: &Entity,
        mouse_position: &Vec2,
        least_distance: &mut f32,
        least_entity: &mut Option<(Entity, CanvasShape)>,
        is_hovering: &mut bool,
    ) {
        // check for vertices
        for (vertex_entity, owned_by_file) in vertex_q.iter() {
            if owned_by_file.file_entity != *current_file_entity {
                continue;
            }
            Self::hover_check_vertex(
                transform_q,
                mouse_position,
                least_distance,
                least_entity,
                &vertex_entity,
            );
        }

        *is_hovering = *least_distance <= Vertex2d::DETECT_RADIUS;
    }

    fn hover_check_vertex(
        transform_q: &Query<&Transform>,
        mouse_position: &Vec2,
        least_distance: &mut f32,
        least_entity: &mut Option<(Entity, CanvasShape)>,
        vertex_entity: &Entity,
    ) {
        let Ok(vertex_transform) = transform_q.get(*vertex_entity) else {
            return;
        };
        let vertex_position = vertex_transform.translation.truncate();
        let distance = vertex_position.distance(*mouse_position);
        if distance < *least_distance {
            *least_distance = distance;
            *least_entity = Some((*vertex_entity, CanvasShape::Vertex));
        }
    }

    fn handle_edge_hover(
        transform_q: &Query<&Transform>,
        edge_q: &Query<(Entity, &OwnedByFileLocal), With<IconEdgeLocal>>,
        current_file_entity: &Entity,
        mouse_position: &Vec2,
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

                Self::hover_check_edge(
                    transform_q,
                    mouse_position,
                    least_distance,
                    least_entity,
                    &edge_entity,
                );
            }

            *is_hovering = *least_distance <= Edge2dLocal::DETECT_THICKNESS;
        }
    }

    fn hover_check_edge(
        transform_q: &Query<&Transform>,
        mouse_position: &Vec2,
        least_distance: &mut f32,
        least_entity: &mut Option<(Entity, CanvasShape)>,
        edge_entity: &Entity,
    ) {

        let edge_transform = transform_q.get(*edge_entity).unwrap();
        let edge_start = edge_transform.translation.truncate();
        let edge_end = get_2d_line_transform_endpoint(&edge_transform);

        let distance = distance_to_2d_line(*mouse_position, edge_start, edge_end);
        if distance < *least_distance {
            *least_distance = distance;
            *least_entity = Some((*edge_entity, CanvasShape::Edge));
        }
    }

    fn handle_face_hover(
        transform_q: &Query<&Transform>,
        face_q: &Query<(Entity, &OwnedByFileLocal), With<IconLocalFace>>,
        current_file_entity: &Entity,
        mouse_position: &Vec2,
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

                let face_transform = transform_q.get(face_entity).unwrap();
                let face_position = face_transform.translation.truncate();
                let distance = face_position.distance(*mouse_position);
                if distance < *least_distance {
                    *least_distance = distance;

                    *least_entity = Some((face_entity, CanvasShape::Face));
                }
            }

            *is_hovering = *least_distance <= FaceIcon2d::DETECT_RADIUS;
        }
    }
}