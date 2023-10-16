use std::f32::consts::FRAC_PI_2;

use bevy_ecs::{
    system::{Commands, Query, ResMut, SystemState},
    world::{Mut, World},
};
use bevy_log::info;

use naia_bevy_client::{Client, CommandsExt};

use input::{InputAction, Key, MouseButton};
use math::Vec2;
use render_api::{
    components::Transform,
    shapes::{angle_between, get_2d_line_transform_endpoint, normalize_angle},
};

use vortex_proto::components::EdgeAngle;

use crate::app::{
    components::VertexTypeData,
    resources::{
        action::shape::ShapeAction, canvas::Canvas, edge_manager::EdgeManager, input::InputManager,
        shape_data::CanvasShape, tab_manager::TabManager,
    },
    ui::widgets::naming_bar_visibility_toggle,
};

pub struct SkelInputManager;

impl SkelInputManager {
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
                    input_manager.reset_last_dragged_vertex(world);
                    Self::reset_last_dragged_edge(world, input_manager);
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
                    Key::Delete => Self::handle_delete_key_press(world, input_manager),
                    Key::N => naming_bar_visibility_toggle(world, input_manager),
                    Key::E => InputManager::handle_edge_angle_visibility_toggle(world),
                    _ => {}
                },
                _ => {}
            }
        }
    }

    pub(crate) fn handle_delete_key_press(world: &mut World, input_manager: &mut InputManager) {
        match input_manager.selected_shape {
            Some((vertex_2d_entity, CanvasShape::Vertex)) => {
                input_manager.handle_delete_vertex_action(world, &vertex_2d_entity)
            }
            _ => {}
        }
    }

    fn reset_last_dragged_edge(world: &mut World, input_manager: &mut InputManager) {
        // reset last dragged edge
        if let Some((edge_2d_entity, old_angle, new_angle)) = world
            .get_resource_mut::<EdgeManager>()
            .unwrap()
            .take_last_edge_dragged()
        {
            world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                tab_manager.current_tab_execute_shape_action(
                    world,
                    input_manager,
                    ShapeAction::RotateEdge(edge_2d_entity, old_angle, new_angle),
                );
            });
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

        // click_type, selected_shape, hovered_shape, current_file_type
        match (click_type, selected_shape, hovered_shape) {
            (MouseButton::Left, Some(CanvasShape::Edge), _) => {
                // should not ever be able to attach something to an edge
                // select hovered entity
                world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                    tab_manager.current_tab_execute_shape_action(
                        world,
                        input_manager,
                        ShapeAction::SelectShape(input_manager.hovered_entity),
                    );
                });
                return;
            }
            (MouseButton::Left, Some(CanvasShape::Vertex | CanvasShape::RootVertex), Some(_)) => {
                // skel file type does nothing when trying to connect vertices together
                // select hovered entity
                world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                    tab_manager.current_tab_execute_shape_action(
                        world,
                        input_manager,
                        ShapeAction::SelectShape(input_manager.hovered_entity),
                    );
                });
                return;
            }
            (MouseButton::Left, Some(CanvasShape::Vertex | CanvasShape::RootVertex), None) => {
                let (vertex_2d_entity, _) = input_manager.selected_shape.unwrap();
                let vertex_type_data = VertexTypeData::Skel(vertex_2d_entity, 0.0, None);
                InputManager::handle_create_new_vertex(
                    world,
                    input_manager,
                    &mouse_position,
                    vertex_2d_entity,
                    vertex_type_data,
                );
            }
            (
                MouseButton::Left,
                None,
                Some(CanvasShape::RootVertex | CanvasShape::Vertex | CanvasShape::Edge),
            ) => {
                world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                    tab_manager.current_tab_execute_shape_action(
                        world,
                        input_manager,
                        ShapeAction::SelectShape(input_manager.hovered_entity),
                    );
                });
            }
            (MouseButton::Right, _, _) => {
                // deselect vertex
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
            (MouseButton::Left, Some((vertex_2d_entity, CanvasShape::Vertex))) => {
                InputManager::handle_vertex_drag(world, &vertex_2d_entity, &mouse_position)
            }
            (MouseButton::Left, Some((edge_2d_entity, CanvasShape::Edge))) => {
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
                let auth_status = commands.entity(edge_3d_entity).authority(&client).unwrap();
                if !(auth_status.is_requested() || auth_status.is_granted()) {
                    // only continue to mutate if requested or granted authority over edge
                    info!("No authority over edge, skipping..");
                    return;
                }

                let edge_2d_transform = transform_q.get(edge_2d_entity).unwrap();
                let start_pos = edge_2d_transform.translation.truncate();
                let end_pos = get_2d_line_transform_endpoint(&edge_2d_transform);
                let base_angle = angle_between(&start_pos, &end_pos);

                let edge_angle_entity = edge_manager.edge_get_base_circle_entity(&edge_3d_entity);
                let edge_angle_pos = transform_q
                    .get(edge_angle_entity)
                    .unwrap()
                    .translation
                    .truncate();

                let mut edge_angle = edge_angle_q.get_mut(edge_3d_entity).unwrap();
                let new_angle = normalize_angle(
                    angle_between(&edge_angle_pos, &mouse_position) - FRAC_PI_2 - base_angle,
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
            (_, _) => InputManager::handle_drag_empty_space(world, click_type, delta),
        }
    }
}
