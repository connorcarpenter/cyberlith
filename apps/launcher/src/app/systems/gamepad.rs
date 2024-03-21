use std::time::Duration;

use bevy_ecs::{system::{Res, ResMut}, event::EventReader};
use bevy_log::info;

use game_engine::input::{InputEvent, Key, MouseButton, GamepadButtonType, GamepadRumbleIntensity, GamepadButton, Input, RumbleManager, Joystick, JoystickType};

pub(crate) fn gamepad_system(
    winit_input: Res<Input>,
    mut winit_input_events: EventReader<InputEvent>,
    mut rumble_manager: ResMut<RumbleManager>,
) {
    // winit events
    for event in winit_input_events.read() {
        match event {
            InputEvent::MouseClicked(_, _) => {}
            InputEvent::MouseReleased(_) => {}
            InputEvent::MouseMoved(_) => {}
            InputEvent::MouseDragged(_, _, _) => {}
            InputEvent::MouseMiddleScrolled(_) => {}
            InputEvent::KeyPressed(_) => {}
            InputEvent::KeyReleased(_) => {}
            InputEvent::GamepadConnected(id) => {
                info!("INPUT: {:?} connected", id);
                info!("---");
            }
            InputEvent::GamepadDisconnected(id) => {
                info!("INPUT: {:?} disconnected", id);
                info!("---");
            }
            InputEvent::GamepadButtonPressed(_id, btn) => {
                info!("INPUT: {:?} pressed", btn);
                info!("---");
            }
            InputEvent::GamepadButtonReleased(_id, btn) => {
                info!("INPUT: {:?} released", btn);
                info!("---");
            }
            InputEvent::GamepadJoystickMoved(_id, joystick, value) => {
                info!("INPUT: {:?} move to {}", joystick, value);
                // info!("---");
            }
        }
    }

    // winit state
    let space_pressed = winit_input.is_pressed(Key::Space);
    let mouse_pos = winit_input.mouse_position();
    let left_btn_pressed = winit_input.is_pressed(MouseButton::Left);

    // gamepad state
    for gamepad_id in winit_input.gamepads_iter() {
        if winit_input.is_pressed(GamepadButton::new(gamepad_id, GamepadButtonType::RightBumper)) {
            //info!("{:?} RightBumper pressed", gamepad);
            rumble_manager.add_rumble(
                gamepad_id,
                Duration::from_secs(1),
                GamepadRumbleIntensity::strong_motor(0.1),
            );
        } else if !winit_input.is_pressed(GamepadButton::new(gamepad_id, GamepadButtonType::RightBumper))
        {
            //info!("{:?} RightBumper not pressed", gamepad);
        }

        let left_joystick_pos = winit_input.joystick_position(Joystick::new(gamepad_id, JoystickType::Left));
        let right_joystick_pos = winit_input.joystick_position(Joystick::new(gamepad_id, JoystickType::Right));

        // if left_joystick_pos.abs() > 0.1 {
        //     info!("INPUT: {:?} left joystick position: {:?}", gamepad_id, left_joystick_pos);
        // }
        // if right_joystick_pos.abs() > 0.1 {
        //     info!("INPUT: {:?} right joystick position: {:?}", gamepad_id, right_joystick_pos);
        // }
    }
}