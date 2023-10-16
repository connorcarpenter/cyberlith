use bevy_ecs::world::{Mut, World};

use input::{InputAction, Key, MouseButton};

use vortex_proto::components::FileExtension;

use crate::app::{
    resources::{
        action::shape::ShapeAction, edge_manager::EdgeManager, input::InputManager,
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
                InputAction::MouseClick(click_type, mouse_position) => input_manager
                    .handle_mouse_click_skelmesh(
                        world,
                        FileExtension::Skel,
                        click_type,
                        &mouse_position,
                    ),
                InputAction::MouseDragged(click_type, mouse_position, delta) => input_manager
                    .handle_mouse_drag_skelmesh(
                        world,
                        FileExtension::Skel,
                        click_type,
                        mouse_position,
                        delta,
                    ),
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
}
