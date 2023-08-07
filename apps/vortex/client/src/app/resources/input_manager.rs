use std::collections::HashMap;
use bevy_ecs::prelude::Resource;

use input::{Input, Key, MouseButton};
use math::Vec2;

use crate::app::resources::camera_manager::CameraAngle;

#[derive(Clone, Copy)]
pub enum ClickType {
    Left,
    Right,
}

#[derive(Clone, Copy)]
pub enum InputAction {
    MiddleMouseScroll(f32),
    MouseMoved,
    SwitchTo3dMode,
    SwitchTo2dMode,
    SetCameraAngleFixed(CameraAngle),
    DeleteKeyPress,
    InsertKeyPress,
    CameraAngleYawRotate(bool),
    MouseDragged(ClickType, Vec2, Vec2),
    MouseClick(ClickType, Vec2),
    MouseRelease,
}

struct KeyState {
    pressed: bool,
    action: InputAction,
}

impl KeyState {
    fn new(action: InputAction) -> Self {
        Self {
            pressed: false,
            action,
        }
    }
}

struct KeyMap {
    map: HashMap<Key, KeyState>,
}

impl KeyMap {

    fn init(keys: Vec<(Key, InputAction)>) -> Self {
        let mut state = HashMap::new();

        for (key, action) in keys {
            state.insert(key, KeyState::new(action));
        }

        Self {
            map: state,
        }
    }

    fn get_actions(&mut self, input: &mut Input, output: &mut Vec<InputAction>) {
        for (key, state) in self.map.iter_mut() {
            if input.is_pressed(*key) {
                if !state.pressed {
                    output.push(state.action);
                }
                state.pressed = true;
            } else {
                state.pressed = false;
            }
        }
    }
}

#[derive(Resource)]
pub struct InputManager {
    key_map: KeyMap,
    click_type: ClickType,
    click_start: Vec2,
    click_down: bool,
    last_mouse_position: Vec2,
}

impl Default for InputManager {
    fn default() -> Self {

        let key_state = KeyMap::init(vec![
            (Key::S, InputAction::SwitchTo3dMode),
            (Key::W, InputAction::SwitchTo2dMode),
            (Key::Num1, InputAction::SetCameraAngleFixed(CameraAngle::Ingame(1))),
            (Key::Num2, InputAction::SetCameraAngleFixed(CameraAngle::Ingame(2))),
            (Key::Num3, InputAction::SetCameraAngleFixed(CameraAngle::Ingame(3))),
            (Key::Num4, InputAction::SetCameraAngleFixed(CameraAngle::Ingame(4))),
            (Key::Num5, InputAction::SetCameraAngleFixed(CameraAngle::Ingame(5))),
            (Key::D, InputAction::SetCameraAngleFixed(CameraAngle::Side)),
            (Key::T, InputAction::SetCameraAngleFixed(CameraAngle::Top)),
            (Key::F, InputAction::SetCameraAngleFixed(CameraAngle::Front)),
            (Key::PageUp, InputAction::CameraAngleYawRotate(true)),
            (Key::PageDown, InputAction::CameraAngleYawRotate(false)),
            (Key::Insert, InputAction::InsertKeyPress),
            (Key::Delete, InputAction::DeleteKeyPress),
        ]);

        Self {
            click_type: ClickType::Left,
            click_start: Vec2::ZERO,
            click_down: false,
            key_map: key_state,
            last_mouse_position: Vec2::ZERO,
        }
    }
}

impl InputManager {
    pub fn update_input(&mut self, input: &mut Input) -> Vec<InputAction> {
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
        self.key_map.get_actions(input, &mut output);

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
                    output.push(InputAction::MouseDragged(
                        self.click_type,
                        mouse_position,
                        delta,
                    ));
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
