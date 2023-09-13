use bevy_ecs::system::{Commands, Query, Resource};

use naia_bevy_client::Client;

use input::{InputAction, Key, MouseButton};
use math::Vec2;
use render_api::components::{Camera, Projection, Transform};

use vortex_proto::components::{EdgeAngle, Vertex3d};

use crate::app::resources::{
    camera_manager::CameraAngle, canvas::Canvas, key_action_map::KeyActionMap,
    action::ShapeAction,
    animation_manager::AnimationManager,
    camera_manager::CameraManager,
    edge_manager::EdgeManager,
    face_manager::FaceManager,
    shape_manager::ShapeManager,
    tab_manager::TabState,
    vertex_manager::VertexManager,
};

#[derive(Clone, Copy)]
pub enum AppInputAction {
    MiddleMouseScroll(f32),
    MouseMoved,
    MouseDragged(MouseButton, Vec2, Vec2),
    MouseClick(MouseButton, Vec2),
    MouseRelease(MouseButton),

    SwitchTo3dMode,
    SwitchTo2dMode,
    SetCameraAngleFixed(CameraAngle),
    CameraAngleYawRotate(bool),
    DeleteKeyPress,
    InsertKeyPress,
}

#[derive(Resource)]
pub struct InputManager {
    key_action_map: KeyActionMap<AppInputAction>,
}

impl Default for InputManager {
    fn default() -> Self {
        let key_state = KeyActionMap::init(vec![
            (Key::S, AppInputAction::SwitchTo3dMode),
            (Key::W, AppInputAction::SwitchTo2dMode),
            (
                Key::Num1,
                AppInputAction::SetCameraAngleFixed(CameraAngle::Ingame(1)),
            ),
            (
                Key::Num2,
                AppInputAction::SetCameraAngleFixed(CameraAngle::Ingame(2)),
            ),
            (
                Key::Num3,
                AppInputAction::SetCameraAngleFixed(CameraAngle::Ingame(3)),
            ),
            (
                Key::Num4,
                AppInputAction::SetCameraAngleFixed(CameraAngle::Ingame(4)),
            ),
            (
                Key::Num5,
                AppInputAction::SetCameraAngleFixed(CameraAngle::Ingame(5)),
            ),
            (
                Key::D,
                AppInputAction::SetCameraAngleFixed(CameraAngle::Side),
            ),
            (
                Key::T,
                AppInputAction::SetCameraAngleFixed(CameraAngle::Top),
            ),
            (
                Key::F,
                AppInputAction::SetCameraAngleFixed(CameraAngle::Front),
            ),
            (Key::PageUp, AppInputAction::CameraAngleYawRotate(true)),
            (Key::PageDown, AppInputAction::CameraAngleYawRotate(false)),
            (Key::Insert, AppInputAction::InsertKeyPress),
            (Key::Delete, AppInputAction::DeleteKeyPress),
        ]);

        Self {
            key_action_map: key_state,
        }
    }
}

impl InputManager {

    pub fn update_input(
        &self,

        // input
        input_actions: Vec<InputAction>,

        // resources
        commands: &mut Commands,
        client: &mut Client,
        canvas: &Canvas,
        camera_manager: &mut CameraManager,
        shape_manager: &mut ShapeManager,
        animation_manager: &mut AnimationManager,
        tab_state: &mut TabState,
        vertex_manager: &mut VertexManager,
        edge_manager: &mut EdgeManager,
        face_manager: &FaceManager,

        // queries
        transform_q: &mut Query<&mut Transform>,
        camera_q: &mut Query<(&mut Camera, &mut Projection)>,
        vertex_3d_q: &mut Query<&mut Vertex3d>,
        edge_angle_q: &mut Query<&mut EdgeAngle>,
    ) {
        let camera_state = &mut tab_state.camera_state;

        let mut app_actions = Vec::new();

        for action in input_actions {
            match action {
                InputAction::MiddleMouseScroll(scroll_amount) => {
                    app_actions.push(AppInputAction::MiddleMouseScroll(scroll_amount))
                }
                InputAction::MouseMoved => app_actions.push(AppInputAction::MouseMoved),
                InputAction::MouseDragged(click_type, mouse_position, delta) => {
                    if canvas.has_focus() {
                        app_actions.push(AppInputAction::MouseDragged(
                            click_type,
                            mouse_position,
                            delta,
                        ));
                    }
                }
                InputAction::MouseClick(click_type, mouse_position) => {
                    // check if mouse position is outside of canvas
                    if !canvas.is_position_inside(mouse_position) {
                        continue;
                    }

                    app_actions.push(AppInputAction::MouseClick(click_type, mouse_position))
                }
                InputAction::MouseRelease(click_type) => {
                    app_actions.push(AppInputAction::MouseRelease(click_type))
                }
                InputAction::KeyPress(key) => {
                    if let Some(action) = self.key_action_map.key_to_action(key) {
                        app_actions.push(action);
                    }
                }
                _ => {}
            }
        }

        // TODO: unify input_actions and app_actions!

        for input_action in app_actions {
            match input_action {
                AppInputAction::MiddleMouseScroll(scroll_y) => {
                    camera_manager.camera_zoom(camera_state, scroll_y);
                }
                AppInputAction::MouseMoved => {
                    shape_manager.queue_resync_hover_ui();
                    shape_manager.queue_resync_selection_ui();
                }
                AppInputAction::SwitchTo3dMode => {
                    // disable 2d camera, enable 3d camera
                    camera_state.set_3d_mode();
                    camera_manager.recalculate_3d_view();
                    shape_manager.queue_resync_shapes();
                }
                AppInputAction::SwitchTo2dMode => {
                    // disable 3d camera, enable 2d camera
                    camera_state.set_2d_mode();
                    camera_manager.recalculate_3d_view();
                    shape_manager.queue_resync_shapes();
                }
                AppInputAction::SetCameraAngleFixed(camera_angle) => match camera_angle {
                    CameraAngle::Side => {
                        camera_manager.set_camera_angle_side(camera_state);
                    }
                    CameraAngle::Front => {
                        camera_manager.set_camera_angle_front(camera_state);
                    }
                    CameraAngle::Top => {
                        camera_manager.set_camera_angle_top(camera_state);
                    }
                    CameraAngle::Ingame(angle_index) => {
                        camera_manager.set_camera_angle_ingame(camera_state, angle_index);
                    }
                },
                AppInputAction::InsertKeyPress => {
                    shape_manager.handle_insert_key_press(&mut tab_state.action_stack);
                }
                AppInputAction::DeleteKeyPress => {
                    shape_manager.handle_delete_key_press(
                        commands,
                        client,
                        &mut tab_state.action_stack,
                        &vertex_manager,
                        &edge_manager,
                        &face_manager,
                    );
                }
                AppInputAction::CameraAngleYawRotate(clockwise) => {
                    camera_manager.set_camera_angle_yaw_rotate(camera_state, clockwise);
                }
                AppInputAction::MouseDragged(click_type, mouse_position, delta) => {
                    shape_manager.handle_mouse_drag(
                        commands,
                        client,
                        camera_manager,
                        camera_state,
                        vertex_manager,
                        edge_manager,
                        animation_manager,
                        click_type,
                        mouse_position,
                        delta,
                        camera_q,
                        transform_q,
                        vertex_3d_q,
                        edge_angle_q,
                    );
                }
                AppInputAction::MouseClick(click_type, mouse_position) => {
                    shape_manager.handle_mouse_click(
                        camera_manager,
                        vertex_manager,
                        edge_manager,
                        &mut tab_state.action_stack,
                        click_type,
                        &mouse_position,
                        camera_q,
                        transform_q,
                    );
                }
                AppInputAction::MouseRelease(MouseButton::Left) => {
                    if let Some((vertex_2d_entity, old_pos, new_pos)) =
                        vertex_manager.last_vertex_dragged.take()
                    {
                        tab_state
                            .action_stack
                            .buffer_action(ShapeAction::MoveVertex(
                                vertex_2d_entity,
                                old_pos,
                                new_pos,
                            ));
                    }
                    if let Some((edge_2d_entity, old_angle, new_angle)) =
                        edge_manager.last_edge_dragged.take()
                    {
                        tab_state
                            .action_stack
                            .buffer_action(ShapeAction::RotateEdge(
                                edge_2d_entity,
                                old_angle,
                                new_angle,
                            ));
                    }
                }
                _ => {}
            }
        }
    }
}
