use std::time::Duration;
use bevy_ecs::event::{EventReader, EventWriter};
use bevy_ecs::system::{NonSend, Res};
use bevy_log::info;

use game_engine::input::{gamepad::{GamepadButtonType, GamepadConnectionEvent, GamepadButtonInput, GamepadRumbleIntensity, GamepadButtonChangedEvent, GamepadAxisChangedEvent, GamepadAxisType, GamepadAxis, Axis, GamepadButton, Gamepads, InputGilrs}, Input};
use game_engine::input::{InputEvent, Key, MouseButton};
use game_engine::input::gamepad::GamepadRumbleRequest;

pub(crate) fn gamepad_system(
    winit_input: Res<Input>,
    mut winit_input_events: EventReader<InputEvent>,

    gamepads: Res<Gamepads>,
    button_inputs: NonSend<InputGilrs>,
    axes: Res<Axis<GamepadAxis>>,
    mut gilrs_cnct_events: EventReader<GamepadConnectionEvent>,
    mut gilrs_axis_events: EventReader<GamepadAxisChangedEvent>,
    mut gilrs_bttn_events: EventReader<GamepadButtonChangedEvent>,
    mut gilrs_inpt_events: EventReader<GamepadButtonInput>,
    mut rumble_requests: EventWriter<GamepadRumbleRequest>,
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

    // gamepad events
    for connection_event in gilrs_cnct_events.read() {
        info!("{:?}", connection_event);
    }
    for axis_changed_event in gilrs_axis_events.read() {
        info!(
            "{:?} of {:?} is changed to {}",
            axis_changed_event.axis_type, axis_changed_event.gamepad, axis_changed_event.value
        );
    }
    for button_changed_event in gilrs_bttn_events.read() {
        info!(
            "{:?} of {:?} is changed to {}",
            button_changed_event.button_type,
            button_changed_event.gamepad,
            button_changed_event.value
        );
    }
    for button_input_event in gilrs_inpt_events.read() {
        info!("{:?}", button_input_event);
    }

    // gamepad state
    for gamepad in gamepads.iter() {
        if button_inputs.is_pressed(GamepadButton::new(gamepad, GamepadButtonType::RightBumper)) {
            info!("{:?} RightBumper pressed", gamepad);
            rumble_requests.send(GamepadRumbleRequest::Add {
                gamepad,
                intensity: GamepadRumbleIntensity::strong_motor(0.1),
                duration: Duration::from_secs(5),
            });
        } else if !button_inputs.is_pressed(GamepadButton::new(gamepad, GamepadButtonType::RightBumper))
        {
            //info!("{:?} RightBumper not pressed", gamepad);
        }

        let right_trigger = axes
            .get(GamepadAxis::new(
                gamepad,
                GamepadAxisType::RightTrigger,
            ))
            .unwrap();
        if right_trigger > 0.01 {
            info!("{:?} RightTrigger value is {}", gamepad, right_trigger);
        }

        let left_stick_x = axes
            .get(GamepadAxis::new(gamepad, GamepadAxisType::LeftStickX))
            .unwrap();
        if left_stick_x.abs() > 0.01 {
            info!("{:?} LeftStickX value is {}", gamepad, left_stick_x);
        }
    }
}