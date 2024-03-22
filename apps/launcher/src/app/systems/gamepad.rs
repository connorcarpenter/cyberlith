use std::time::Duration;

use bevy_ecs::{
    event::EventReader,
    system::{Res, ResMut},
};
use bevy_log::info;

use game_engine::input::{
    GamepadButton, GamepadButtonType, GamepadRumbleIntensity, Input, InputEvent, Joystick,
    JoystickType, Key, MouseButton, RumbleManager,
};
use game_engine::math;

pub(crate) fn gamepad_system(
    input: Res<Input>,
    mut input_events: EventReader<InputEvent>,
    mut rumble_manager: ResMut<RumbleManager>,
) {
    // winit events
    for event in input_events.read() {
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
            InputEvent::GamepadButtonPressed(id, btn) => {
                info!("INPUT: {:?} pressed", btn);
                info!("---");

                if *btn == GamepadButtonType::RightBumper {

                    // TESTING
                    let duration = 1000; //duration.as_millis() as u32;
                    let weak_motor = game_engine::random::gen_range_f32(0.0, 1.0);
                    let strong_motor = game_engine::random::gen_range_f32(0.0, 1.0);
                    // TESTING

                    rumble_manager.add_rumble(
                        *id,
                        Duration::from_millis(duration),
                        GamepadRumbleIntensity {
                            strong_motor,
                            weak_motor,
                        },
                    );
                }
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
    let space_pressed = input.is_pressed(Key::Space);
    let mouse_pos = input.mouse_position();
    let left_btn_pressed = input.is_pressed(MouseButton::Left);

    // gamepad state
    for gamepad_id in input.gamepads_iter() {
        if input.is_pressed(GamepadButton::new(
            gamepad_id,
            GamepadButtonType::RightBumper,
        )) {
            //info!("{:?} RightBumper pressed", gamepad);
            // rumble_manager.add_rumble(
            //     gamepad_id,
            //     Duration::from_secs(1),
            //     GamepadRumbleIntensity::strong_motor(0.1),
            // );
        } else if !input.is_pressed(GamepadButton::new(
            gamepad_id,
            GamepadButtonType::RightBumper,
        )) {
            //info!("{:?} RightBumper not pressed", gamepad);
        }

        let left_joystick_pos =
            input.joystick_position(Joystick::new(gamepad_id, JoystickType::Left));
        let right_joystick_pos =
            input.joystick_position(Joystick::new(gamepad_id, JoystickType::Right));

        // if left_joystick_pos.abs() > 0.1 {
        //     info!("INPUT: {:?} left joystick position: {:?}", gamepad_id, left_joystick_pos);
        // }
        // if right_joystick_pos.abs() > 0.1 {
        //     info!("INPUT: {:?} right joystick position: {:?}", gamepad_id, right_joystick_pos);
        // }
    }
}
