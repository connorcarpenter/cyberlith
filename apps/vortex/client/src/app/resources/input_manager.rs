use bevy_ecs::system::Resource;

use input::{InputAction, Key, MouseButton};
use math::Vec2;

use crate::app::resources::{camera_manager::CameraAngle, key_action_map::KeyActionMap};

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
    pub fn input_to_app_actions(&mut self, input_actions: Vec<InputAction>) -> Vec<AppInputAction> {
        let mut output = Vec::new();

        for action in input_actions {
            match action {
                InputAction::MiddleMouseScroll(scroll_amount) => {
                    output.push(AppInputAction::MiddleMouseScroll(scroll_amount))
                }
                InputAction::MouseMoved => output.push(AppInputAction::MouseMoved),
                InputAction::MouseDragged(click_type, mouse_position, delta) => output.push(
                    AppInputAction::MouseDragged(click_type, mouse_position, delta),
                ),
                InputAction::MouseClick(click_type, mouse_position) => {
                    output.push(AppInputAction::MouseClick(click_type, mouse_position))
                }
                InputAction::MouseRelease(click_type) => {
                    output.push(AppInputAction::MouseRelease(click_type))
                }
                InputAction::KeyPress(key) => {
                    if let Some(action) = self.key_action_map.key_to_action(key) {
                        output.push(action);
                    }
                }
                _ => {}
            }
        }

        output
    }
}
