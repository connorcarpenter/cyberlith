use bevy_ecs::{
    entity::Entity,
    query::{With, Without},
    system::{Query, SystemState},
    world::{Mut, World},
};

use input::{InputAction, Key, MouseButton};
use math::Vec2;
use render_api::components::{Transform, Visibility};

use vortex_proto::components::VertexRoot;

use crate::app::{
    components::{Edge2dLocal, LocalShape, Vertex2d},
    resources::{
        canvas::Canvas, input::InputManager, shape_data::CanvasShape, tab_manager::TabManager,
    },
};

pub struct ModelInputManager;

impl ModelInputManager {
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
                    // input_manager.reset_last_dragged_vertex(world);
                    // Self::reset_last_dragged_edge(world, input_manager);
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
                    _ => {}
                },
                _ => {}
            }
        }
    }

    pub(crate) fn handle_delete_key_press(world: &mut World, input_manager: &mut InputManager) {
        // todo
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
            (
                MouseButton::Left,
                _,
                Some(CanvasShape::Vertex | CanvasShape::RootVertex | CanvasShape::Face),
            ) => {
                // // unselect shape
                // world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                //     tab_manager.current_tab_execute_shape_action(
                //         world,
                //         input_manager,
                //         ShapeAction::SelectShape(None),
                //     );
                // });
            }
            (MouseButton::Left, _, _) => {
                // // select hovered shape (or None if there is no hovered shape)
                // world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                //     tab_manager.current_tab_execute_shape_action(
                //         world,
                //         input_manager,
                //         ShapeAction::SelectShape(input_manager.hovered_entity),
                //     );
                // });
            }
            (MouseButton::Right, _, _) => {
                // // deselect shape
                // world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                //     tab_manager.current_tab_execute_shape_action(
                //         world,
                //         input_manager,
                //         ShapeAction::SelectShape(None),
                //     );
                // });
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
            (_, _) => InputManager::handle_drag_empty_space(world, click_type, delta),
        }
    }

    pub(crate) fn sync_mouse_hover_ui(
        world: &mut World,
        camera_3d_scale: f32,
        mouse_position: &Vec2,
    ) -> Option<(Entity, CanvasShape)> {
        let mut system_state: SystemState<(
            Query<(&Transform, Option<&LocalShape>)>,
            Query<&Visibility>,
            Query<(Entity, Option<&VertexRoot>), (With<Vertex2d>, Without<LocalShape>)>,
            Query<(Entity, &Edge2dLocal), Without<LocalShape>>,
        )> = SystemState::new(world);
        let (transform_q, visibility_q, vertex_2d_q, edge_2d_q) = system_state.get_mut(world);

        let mut least_distance = f32::MAX;
        let mut least_entity = None;
        let mut is_hovering = false;

        InputManager::handle_vertex_and_edge_hover(
            &transform_q,
            &visibility_q,
            &vertex_2d_q,
            &edge_2d_q,
            None,
            camera_3d_scale,
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
}
