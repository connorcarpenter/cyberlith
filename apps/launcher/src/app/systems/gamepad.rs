use std::time::Duration;

use bevy_ecs::{system::Res, event::{EventReader, EventWriter}};
use bevy_log::info;

use game_engine::input::{InputEvent, Key, MouseButton, GamepadRumbleRequest, GamepadButtonType, GamepadRumbleIntensity, GamepadAxisType, GamepadAxis, GamepadButton, Input};

pub(crate) fn gamepad_system(
    winit_input: Res<Input>,
    mut winit_input_events: EventReader<InputEvent>,
    mut rumble_requests: EventWriter<GamepadRumbleRequest>,
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
                info!("{:?} connected", id);
            }
            InputEvent::GamepadDisconnected(id) => {
                info!("{:?} disconnected", id);
            }
            InputEvent::GamepadButtonPressed(id, btn) => {
                info!("{:?} {:?} pressed", id, btn);
            }
            InputEvent::GamepadButtonReleased(id, btn) => {
                info!("{:?} {:?} released", id, btn);
            }
            InputEvent::GamepadAxisChanged(id, axis, value) => {
                info!("{:?} {:?} changed to {}", id, axis, value);
            }
        }
    }

    // winit state
    let space_pressed = winit_input.is_pressed(Key::Space);
    let mouse_pos = winit_input.mouse_position();
    let left_btn_pressed = winit_input.is_pressed(MouseButton::Left);

    // gamepad state
    for gamepad in winit_input.gamepads_iter() {
        if winit_input.gamepad_button_is_pressed(GamepadButton::new(gamepad, GamepadButtonType::RightBumper)) {
            info!("{:?} RightBumper pressed", gamepad);
            rumble_requests.send(GamepadRumbleRequest::Add {
                gamepad,
                intensity: GamepadRumbleIntensity::strong_motor(0.1),
                duration: Duration::from_secs(5),
            });
        } else if !winit_input.gamepad_button_is_pressed(GamepadButton::new(gamepad, GamepadButtonType::RightBumper))
        {
            //info!("{:?} RightBumper not pressed", gamepad);
        }

        let right_trigger = winit_input.gamepad_axis_get(GamepadAxis::new(
                gamepad,
                GamepadAxisType::RightTrigger,
            ))
            .unwrap();
        if right_trigger > 0.01 {
            info!("{:?} RightTrigger value is {}", gamepad, right_trigger);
        }

        let left_stick_x = winit_input.gamepad_axis_get(GamepadAxis::new(gamepad, GamepadAxisType::LeftStickX))
            .unwrap();
        if left_stick_x.abs() > 0.01 {
            info!("{:?} LeftStickX value is {}", gamepad, left_stick_x);
        }
    }
}