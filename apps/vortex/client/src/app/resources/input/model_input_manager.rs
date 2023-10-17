use bevy_ecs::{
    entity::Entity,
    query::{With, Without},
    system::{Query, SystemState},
    world::World,
};

use input::{InputAction, Key, MouseButton};
use math::Vec2;
use render_api::components::{Transform, Visibility};

use crate::app::{
    components::{Edge2dLocal, LocalShape},
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
                _,
                Some(CanvasShape::Vertex | CanvasShape::RootVertex | CanvasShape::Face),
            ) => {
                // deselect shape
                let mut canvas = world.get_resource_mut::<Canvas>().unwrap();
                input_manager.deselect_shape(&mut canvas);
            }
            (MouseButton::Left, _, shape) => {
                // select hovered shape (or None if there is no hovered shape)
                let mut canvas = world.get_resource_mut::<Canvas>().unwrap();
                match shape {
                    Some(CanvasShape::Edge) => {

                        if input_manager.selected_shape_2d().is_some() {
                            input_manager.deselect_shape(&mut canvas);
                        }

                        let entity = input_manager.hovered_entity.unwrap().0;
                        input_manager.select_shape(&mut canvas, &entity, CanvasShape::Edge);
                    }
                    None => {
                        input_manager.deselect_shape(&mut canvas);
                    }
                    _ => panic!(""),
                }
            }
            (MouseButton::Right, _, _) => {
                // deselect shape
                let mut canvas = world.get_resource_mut::<Canvas>().unwrap();
                input_manager.deselect_shape(&mut canvas);
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
            Query<(&Transform, Option<&LocalShape>)>,
            Query<&Visibility>,
            Query<(Entity, &Edge2dLocal), Without<LocalShape>>,
        )> = SystemState::new(world);
        let (transform_q, visibility_q, edge_2d_q) = system_state.get_mut(world);

        let mut least_distance = f32::MAX;
        let mut least_entity = None;
        let mut is_hovering = false;

        InputManager::handle_edge_hover(
            &transform_q,
            &visibility_q,
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
