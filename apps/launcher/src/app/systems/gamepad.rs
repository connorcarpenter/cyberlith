use bevy_ecs::event::EventReader;
use bevy_ecs::system::{NonSend, Res};
use bevy_log::info;

use game_engine::input::{gamepad::{GamepadButtonType, GamepadAxisType, GamepadAxis, Axis, GamepadButton, Gamepads, InputGilrs}, Input};
use game_engine::input::{InputEvent, Key, MouseButton};

pub(crate) fn gamepad_system(
    winit_input: Res<Input>,
    mut winit_input_events: EventReader<InputEvent>,

    gamepads: Res<Gamepads>,
    button_inputs: NonSend<InputGilrs>,
    button_axes: Res<Axis<GamepadButton>>,
    axes: Res<Axis<GamepadAxis>>,
) {
    // winit events
    for event in winit_input_events.read() {
        match event {
            InputEvent::MouseClick(_, _) => {}
            InputEvent::MouseRelease(_) => {}
            InputEvent::MouseMoved => {}
            InputEvent::MouseDragged(_, _, _) => {}
            InputEvent::MiddleMouseScroll(_) => {}
            InputEvent::KeyPress(_) => {}
            InputEvent::KeyRelease(_) => {}
        }
    }

    // winit state
    let space_pressed = winit_input.is_pressed(Key::Space);
    let mouse_pos = winit_input.mouse_position();
    let left_btn_pressed = winit_input.is_pressed(MouseButton::Left);


    for gamepad in gamepads.iter() {
        if button_inputs.is_pressed(GamepadButton::new(gamepad, GamepadButtonType::South)) {
            info!("{:?} South pressed", gamepad);
        } else if !button_inputs.is_pressed(GamepadButton::new(gamepad, GamepadButtonType::South))
        {
            info!("{:?} South not pressed", gamepad);
        }

        let right_trigger = button_axes
            .get(GamepadButton::new(
                gamepad,
                GamepadButtonType::RightTrigger2,
            ))
            .unwrap();
        if right_trigger.abs() > 0.01 {
            info!("{:?} RightTrigger2 value is {}", gamepad, right_trigger);
        }

        let left_stick_x = axes
            .get(GamepadAxis::new(gamepad, GamepadAxisType::LeftStickX))
            .unwrap();
        if left_stick_x.abs() > 0.01 {
            info!("{:?} LeftStickX value is {}", gamepad, left_stick_x);
        }
    }
}