use bevy_ecs::{
    entity::Entity,
    query::Without,
    system::{Query, Res, SystemState},
    world::{World, Mut},
};

use input::{InputAction, Key, MouseButton};
use math::Vec2;
use render_api::components::{Transform, Visibility};

use vortex_proto::components::ShapeName;

use crate::app::{
    components::{Edge2dLocal, LocalShape},
    resources::{
        canvas::Canvas, edge_manager::EdgeManager, input::InputManager, shape_data::CanvasShape,
        tab_manager::TabManager, action::model::ModelAction
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

    pub(crate) fn handle_delete_key_press(_world: &mut World, _input_manager: &mut InputManager) {
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
                Some(_),
                Some(CanvasShape::Vertex | CanvasShape::RootVertex | CanvasShape::Face) | None,
            ) | (MouseButton::Right, _, _) => {
                // deselect vertex
                world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                    tab_manager.current_tab_execute_model_action(
                        world,
                        input_manager,
                        ModelAction::SelectShape(None),
                    );
                });
            }
            (MouseButton::Left, _, Some(CanvasShape::Edge)) => {
                // select hovered shape (or None if there is no hovered shape)
                world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                    tab_manager.current_tab_execute_model_action(
                        world,
                        input_manager,
                        ModelAction::SelectShape(input_manager.hovered_entity),
                    );
                });
            }
            _ => {}
        }
    }

    pub(crate) fn handle_mouse_drag(
        world: &mut World,
        input_manager: &mut InputManager,
        _mouse_position: Vec2,
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
            Res<EdgeManager>,
            Query<(&Transform, Option<&LocalShape>)>,
            Query<&Visibility>,
            Query<&ShapeName>,
            Query<(Entity, &Edge2dLocal), Without<LocalShape>>,
        )> = SystemState::new(world);
        let (edge_manager, transform_q, visibility_q, shape_name_q, edge_2d_q) =
            system_state.get_mut(world);

        let mut least_distance = f32::MAX;
        let mut least_entity = None;
        let mut is_hovering = false;

        InputManager::handle_edge_hover(
            &transform_q,
            &visibility_q,
            &edge_2d_q,
            Some((&edge_manager, &shape_name_q)),
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
