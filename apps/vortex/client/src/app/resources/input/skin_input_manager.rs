use bevy_ecs::{
    entity::Entity,
    system::{Query, Res, ResMut, SystemState},
    world::{Mut, World},
};

use input::{InputAction, Key, MouseButton};
use math::Vec2;
use render_api::components::{Transform, Visibility};

use crate::app::{
    components::{FaceIcon2d, LocalShape},
    resources::{
        action::skin::SkinAction, canvas::Canvas, input::InputManager, shape_data::CanvasShape,
        tab_manager::TabManager,
    },
};

pub struct SkinInputManager;

impl SkinInputManager {
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
                InputAction::MouseDragged(click_type, _mouse_position, delta) => {
                    Self::handle_mouse_drag(world, click_type, delta)
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
                    Key::Delete => Self::handle_delete_key_press(world, input_manager),
                    _ => {}
                },
                _ => {}
            }
        }
    }

    pub(crate) fn handle_delete_key_press(world: &mut World, input_manager: &mut InputManager) {
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

    pub(crate) fn handle_mouse_click(
        world: &mut World,
        input_manager: &mut InputManager,
        mouse_position: &Vec2,
        click_type: MouseButton,
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

    pub(crate) fn handle_mouse_drag(world: &mut World, click_type: MouseButton, delta: Vec2) {
        if !world.get_resource::<TabManager>().unwrap().has_focus() {
            return;
        }

        InputManager::handle_drag_empty_space(world, click_type, delta);
    }

    pub(crate) fn sync_mouse_hover_ui(
        world: &mut World,
        input_manager: &mut InputManager,
        mouse_position: &Vec2,
    ) {
        let mut system_state: SystemState<(
            ResMut<Canvas>,
            Res<TabManager>,
            Query<(&mut Transform, Option<&LocalShape>)>,
            Query<&Visibility>,
            Query<(Entity, &FaceIcon2d)>,
        )> = SystemState::new(world);
        let (mut canvas, tab_manager, mut transform_q, visibility_q, face_2d_q) =
            system_state.get_mut(world);

        let Some(current_tab_state) = tab_manager.current_tab_state() else {
            return;
        };
        let camera_state = &current_tab_state.camera_state;

        let camera_3d_scale = camera_state.camera_3d_scale();

        let mut least_distance = f32::MAX;
        let mut least_entity = None;
        let mut is_hovering = false;

        // check for faces
        if !is_hovering {
            for (face_entity, _) in face_2d_q.iter() {
                // check tab ownership, skip faces from other tabs
                let Ok(visibility) = visibility_q.get(face_entity) else {
                    panic!("entity has no Visibility");
                };
                if !visibility.visible {
                    continue;
                }

                let (face_transform, _) = transform_q.get(face_entity).unwrap();
                let face_position = face_transform.translation.truncate();
                let distance = face_position.distance(*mouse_position);
                if distance < least_distance {
                    least_distance = distance;

                    least_entity = Some((face_entity, CanvasShape::Face));
                }
            }

            is_hovering = least_distance <= (FaceIcon2d::DETECT_RADIUS * camera_3d_scale);
        }

        // define old and new hovered states
        let old_hovered_entity = input_manager.hovered_entity;
        let next_hovered_entity = if is_hovering { least_entity } else { None };

        input_manager.sync_hover_shape_scale(&mut transform_q, camera_3d_scale);

        // hover state did not change
        if old_hovered_entity == next_hovered_entity {
            return;
        }

        // apply
        input_manager.hovered_entity = next_hovered_entity;
        canvas.queue_resync_shapes_light();
    }
}
