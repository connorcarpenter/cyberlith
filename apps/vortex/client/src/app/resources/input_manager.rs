
use bevy_ecs::prelude::Resource;

use input::{Input, Key, MouseButton};
use math::Vec2;

use crate::app::resources::camera_manager::CameraAngle;

#[derive(Clone, Copy)]
pub enum ClickType {
    Left,
    Right,
}

#[derive(Resource)]
pub struct InputManager {
    rotate_key_down: bool,
    click_type: ClickType,
    click_start: Vec2,
    click_down: bool,
    last_mouse_position: Vec2,
}

impl Default for InputManager {
    fn default() -> Self {
        Self {
            click_type: ClickType::Left,
            click_start: Vec2::ZERO,
            click_down: false,
            rotate_key_down: false,
            last_mouse_position: Vec2::ZERO,
        }
    }
}

pub enum InputAction {
    MiddleMouseScroll(f32),
    MouseMoved,
    SwitchTo3dMode,
    SwitchTo2dMode,
    SetCameraAngleFixed(CameraAngle),
    DeleteKeyPress,
    CameraAngleYawRotate(bool),
    MouseDragged(ClickType, Vec2, Vec2),
    MouseClick(ClickType, Vec2),
    MouseRelease,
}

impl InputManager {
    pub fn update_input(
        &mut self,
        input: &mut Input,
    ) -> Vec<InputAction> {

        let mut output = Vec::new();

        let mouse_position = *input.mouse_position();

        // Mouse wheel zoom..
        let scroll_y = input.consume_mouse_scroll();
        if scroll_y > 0.1 || scroll_y < -0.1 {
            output.push(InputAction::MiddleMouseScroll(scroll_y));
        }

        // Mouse over
        if !self.click_down {
            if mouse_position.x as i16 != self.last_mouse_position.x as i16
                || mouse_position.y as i16 != self.last_mouse_position.y as i16
            {
                // mouse moved!
                self.last_mouse_position = mouse_position;
                output.push(InputAction::MouseMoved);
            }
        }

        // check keyboard input

        // (S)olid 3D View
        if input.is_pressed(Key::S) {
            output.push(InputAction::SwitchTo3dMode);
        }
        // (W)ireframe 2D View
        else if input.is_pressed(Key::W) {
            output.push(InputAction::SwitchTo2dMode);
        }
        // 1 Game Camera View
        else if input.is_pressed(Key::Num1) {
            output.push(InputAction::SetCameraAngleFixed(CameraAngle::Ingame(1)));
        }
        // 2 Game Camera View
        else if input.is_pressed(Key::Num2) {
            output.push(InputAction::SetCameraAngleFixed(CameraAngle::Ingame(2)));
        }
        // 3 Game Camera View
        else if input.is_pressed(Key::Num3) {
            output.push(InputAction::SetCameraAngleFixed(CameraAngle::Ingame(3)));
        }
        // 4 Game Camera View
        else if input.is_pressed(Key::Num4) {
            output.push(InputAction::SetCameraAngleFixed(CameraAngle::Ingame(4)));
        }
        // 5 Game Camera View
        else if input.is_pressed(Key::Num5) {
            output.push(InputAction::SetCameraAngleFixed(CameraAngle::Ingame(5)));
        }
        // Si(d)e Camera View
        else if input.is_pressed(Key::D) {
            output.push(InputAction::SetCameraAngleFixed(CameraAngle::Side));
        }
        // (F)ront Camera View
        else if input.is_pressed(Key::F) {
            output.push(InputAction::SetCameraAngleFixed(CameraAngle::Front));
        }
        // (T)op Camera View
        else if input.is_pressed(Key::T) {
            output.push(InputAction::SetCameraAngleFixed(CameraAngle::Top));
        }
        // Delete
        else if input.is_pressed(Key::Delete) {
            output.push(InputAction::DeleteKeyPress);
        }

        if !self.rotate_key_down {
            // Rotate Yaw 45 degrees
            if input.is_pressed(Key::PageUp) {
                output.push(InputAction::CameraAngleYawRotate(true));
                self.rotate_key_down = true;
            }
            // Rotate Yaw 45 degrees
            else if input.is_pressed(Key::PageDown) {
                output.push(InputAction::CameraAngleYawRotate(false));
                self.rotate_key_down = true;
            }
        } else {
            if !input.is_pressed(Key::PageUp) && !input.is_pressed(Key::PageDown) {
                self.rotate_key_down = false;
            }
        }

        // mouse clicks

        let left_button_pressed = input.is_pressed(MouseButton::Left);
        let right_button_pressed = input.is_pressed(MouseButton::Right);
        let mouse_button_pressed = left_button_pressed || right_button_pressed;

        if mouse_button_pressed {
            if left_button_pressed {
                self.click_type = ClickType::Left;
            }
            if right_button_pressed {
                self.click_type = ClickType::Right;
            }

            if self.click_down {
                // already clicking
                let delta = mouse_position - self.click_start;
                self.click_start = mouse_position;

                if delta.length() > 0.0 {
                    output.push(InputAction::MouseDragged(self.click_type, mouse_position, delta));
                }
            } else {
                // haven't clicked yet
                self.click_down = true;
                self.click_start = mouse_position;
                output.push(InputAction::MouseClick(self.click_type, mouse_position));
            }
        } else {
            if self.click_down {
                // release click
                self.click_down = false;
                output.push(InputAction::MouseRelease);
            }
        }

        output
    }
}