use bevy_ecs::world::{Mut, World};

use input::{InputAction, Key, MouseButton};
use math::Vec2;

use crate::app::resources::{
    action::skin::SkinAction, canvas::Canvas, input::InputManager, shape_data::CanvasShape,
    tab_manager::TabManager,
};

pub struct SkinInputManager;

impl SkinInputManager {
    pub fn update_input_skin(
        input_manager: &mut InputManager,
        world: &mut World,
        input_actions: Vec<InputAction>,
    ) {
        for action in input_actions {
            match action {
                InputAction::MouseClick(click_type, mouse_position) => {
                    Self::handle_mouse_click_skin(input_manager, world, click_type, &mouse_position)
                }
                InputAction::MouseDragged(click_type, _mouse_position, delta) => {
                    Self::handle_mouse_drag_skin(world, click_type, delta)
                }
                InputAction::MiddleMouseScroll(scroll_y) => {
                    InputManager::handle_mouse_scroll_wheel(world, scroll_y)
                }
                InputAction::MouseMoved => {
                    input_manager.queue_resync_hover_ui();
                    input_manager.queue_resync_selection_ui();
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
                    Key::Delete => Self::handle_delete_key_press_skin(input_manager, world),
                    _ => {}
                },
                _ => {}
            }
        }
    }

    pub(crate) fn handle_delete_key_press_skin(
        input_manager: &mut InputManager,
        world: &mut World,
    ) {
        match input_manager.selected_shape {
            Some((face_2d_entity, CanvasShape::Face)) => {
                // let mut system_state: SystemState<(Commands, Client, Res<FaceManager>, Res<SkinManager>)> =
                //     SystemState::new(world);
                // let (mut commands, mut client, face_manager, skin_manager) = system_state.get_mut(world);

                // // get face color
                // let face_3d_entity = face_manager
                //     .face_entity_2d_to_3d(&face_2d_entity)
                //     .unwrap();
                // let face_color_entity = skin_manager.face_to_color_entity(&face_3d_entity).unwrap();

                // // check whether we can delete face color
                // let auth_status = commands
                //     .entity(*face_color_entity)
                //     .authority(&client)
                //     .unwrap();
                // if !auth_status.is_granted() && !auth_status.is_available() {
                //     // do nothing, face color is not available
                //     // TODO: queue for deletion? check before this?
                //     warn!(
                //         "Face Color {:?} is not available for deletion!",
                //         face_color_entity
                //     );
                //     return;
                // }
                //
                // let auth_status = commands
                //     .entity(face_3d_entity)
                //     .authority(&client)
                //     .unwrap();
                // if !auth_status.is_granted() {
                //     // request authority if needed
                //     commands
                //         .entity(face_3d_entity)
                //         .request_authority(&mut client);
                // }

                world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                    tab_manager.current_tab_execute_skin_action(
                        world,
                        input_manager,
                        SkinAction::EditColor(face_2d_entity, None),
                    );
                });

                input_manager.selected_shape = None;
            }
            _ => {}
        }
    }

    pub(crate) fn handle_mouse_click_skin(
        input_manager: &mut InputManager,
        world: &mut World,
        click_type: MouseButton,
        mouse_position: &Vec2,
    ) {
        if input_manager.selected_shape == input_manager.hovered_entity {
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

        let selected_shape = input_manager.selected_shape.map(|(_, shape)| shape);
        let hovered_shape = input_manager.hovered_entity.map(|(_, shape)| shape);

        // click_type, selected_shape, hovered_shape
        match (click_type, selected_shape, hovered_shape) {
            (MouseButton::Left, _, Some(CanvasShape::Face)) => {
                world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                    tab_manager.current_tab_execute_skin_action(
                        world,
                        input_manager,
                        SkinAction::SelectFace(input_manager.hovered_entity),
                    );
                });
                return;
            }
            (MouseButton::Right, Some(CanvasShape::Face), _) => {
                // deselect vertex
                world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                    tab_manager.current_tab_execute_skin_action(
                        world,
                        input_manager,
                        SkinAction::SelectFace(None),
                    );
                });
            }
            _ => {}
        }
    }

    pub(crate) fn handle_mouse_drag_skin(world: &mut World, click_type: MouseButton, delta: Vec2) {
        if !world.get_resource::<TabManager>().unwrap().has_focus() {
            return;
        }

        InputManager::handle_drag_empty_space(world, click_type, delta);
    }
}
