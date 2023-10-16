use bevy_ecs::{
    system::{Commands, Res, SystemState},
    world::{Mut, World},
};
use bevy_log::warn;

use naia_bevy_client::{Client, CommandsExt};

use input::{InputAction, Key, MouseButton};
use math::{Vec2, Vec3};

use crate::app::{
    components::VertexTypeData,
    resources::{
        action::shape::ShapeAction, canvas::Canvas, edge_manager::EdgeManager,
        face_manager::FaceManager, input::InputManager, shape_data::CanvasShape,
        tab_manager::TabManager, vertex_manager::VertexManager,
    },
};

pub struct MeshInputManager;

impl MeshInputManager {
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
                    input_manager.reset_last_dragged_vertex(world)
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
            tab_manager.current_tab_execute_shape_action(
                world,
                input_manager,
                ShapeAction::CreateVertex(
                    VertexTypeData::Mesh(Vec::new(), Vec::new()),
                    Vec3::ZERO,
                    None,
                ),
            );
        })
    }

    pub(crate) fn handle_delete_key_press(input_manager: &mut InputManager, world: &mut World) {
        match input_manager.selected_shape {
            Some((vertex_2d_entity, CanvasShape::Vertex)) => {
                input_manager.handle_delete_vertex_action(world, &vertex_2d_entity)
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
                        input_manager,
                        ShapeAction::DeleteEdge(edge_2d_entity, None),
                    );
                });

                input_manager.selected_shape = None;
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
                        input_manager,
                        ShapeAction::DeleteFace(face_2d_entity),
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
                let (vertex_2d_entity, _) = input_manager.selected_shape.unwrap();
                let vertex_type_data =
                    VertexTypeData::Mesh(vec![(vertex_2d_entity, None)], Vec::new());
                InputManager::handle_create_new_vertex(
                    world,
                    input_manager,
                    &mouse_position,
                    vertex_2d_entity,
                    vertex_type_data,
                );
            }
            (MouseButton::Left, _, _) => {
                // select hovered shape (or None if there is no hovered shape)
                world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                    tab_manager.current_tab_execute_shape_action(
                        world,
                        input_manager,
                        ShapeAction::SelectShape(input_manager.hovered_entity),
                    );
                });
            }
            (MouseButton::Right, _, _) => {
                // deselect shape
                world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                    tab_manager.current_tab_execute_shape_action(
                        world,
                        input_manager,
                        ShapeAction::SelectShape(None),
                    );
                });
            }
            _ => {}
        }
    }

    fn link_vertices(world: &mut World, input_manager: &mut InputManager) {
        let mut system_state: SystemState<(Res<VertexManager>, Res<EdgeManager>)> =
            SystemState::new(world);
        let (vertex_manager, edge_manager) = system_state.get_mut(world);

        // link vertices together
        let (vertex_2d_entity_a, _) = input_manager.selected_shape.unwrap();
        let (vertex_2d_entity_b, _) = input_manager.hovered_entity.unwrap();
        if vertex_2d_entity_a == vertex_2d_entity_b {
            return;
        }

        // check if edge already exists
        if edge_manager
            .edge_2d_entity_from_vertices(&vertex_manager, vertex_2d_entity_a, vertex_2d_entity_b)
            .is_some()
        {
            // select vertex
            world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                tab_manager.current_tab_execute_shape_action(
                    world,
                    input_manager,
                    ShapeAction::SelectShape(Some((vertex_2d_entity_b, CanvasShape::Vertex))),
                );
            });
        } else {
            // create edge
            world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                tab_manager.current_tab_execute_shape_action(
                    world,
                    input_manager,
                    ShapeAction::CreateEdge(
                        vertex_2d_entity_a,
                        vertex_2d_entity_b,
                        (vertex_2d_entity_b, CanvasShape::Vertex),
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
            (MouseButton::Left, Some((vertex_2d_entity, CanvasShape::Vertex))) => InputManager::handle_vertex_drag(world, &vertex_2d_entity, &mouse_position),
            (_, _) => InputManager::handle_drag_empty_space(world, click_type, delta),
        }
    }
}
