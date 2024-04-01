use std::time::Duration;

use js_sys::{Array, Function, Object};
use wasm_bindgen::JsValue;
use web_sys::Gamepad as WebGamepad;

use crate::{
    gamepad::{
        converter::convert_gamepad_id,
        gilrs::GilrsWrapper,
        rumble::{GamepadRumbleRequest, RumbleError},
        GamepadId,
    },
    GamepadRumbleIntensity,
};

pub(crate) fn handle_rumble_request(
    rumble_request: GamepadRumbleRequest,
    input_gilrs: &mut GilrsWrapper,
) -> Result<(), RumbleError> {
    let GamepadRumbleRequest {
        duration,
        intensity,
        gamepad: gamepad_id,
    } = rumble_request;

    input_gilrs.add_rumble(&gamepad_id, duration, intensity, None);

    Ok(())
}

pub(crate) fn set_total_rumbles(
    input_gilrs: &mut GilrsWrapper,
    gamepad_ids: Vec<GamepadId>,
) -> Result<(), RumbleError> {
    for gamepad_id in gamepad_ids {
        if let Some((current_rumble_duration, current_rumble_intensity)) =
            input_gilrs.get_current_rumble(&gamepad_id)
        {
            set_total_rumble(
                input_gilrs,
                &gamepad_id,
                current_rumble_duration,
                current_rumble_intensity,
            )?;
        }
    }

    Ok(())
}

fn set_total_rumble(
    input_gilrs: &mut GilrsWrapper,
    gamepad_id: &GamepadId,
    duration: Duration,
    intensity: GamepadRumbleIntensity,
) -> Result<(), RumbleError> {
    let gilrs = input_gilrs.gilrs_mut();
    let GamepadRumbleIntensity {
        strong_motor,
        weak_motor,
    } = intensity;
    // get gamepad name
    let (_, gamepad) = gilrs
        .gamepads()
        .find(|(pad_id, _)| convert_gamepad_id(*pad_id) == *gamepad_id)
        .ok_or(RumbleError::GamepadNotFound)?;

    let gamepad_name = gamepad.os_name();

    // get gamepad from wasm
    let window = web_sys::window().expect("no global `window` exists");
    let navigator = window.navigator();
    let gamepads = navigator.get_gamepads().unwrap();
    let mut gamepad = None;
    for gamepad_js in gamepads.iter() {
        if gamepad_js.is_null() {
            continue;
        } else {
            let web_gamepad = WebGamepad::from(gamepad_js);
            if web_gamepad.id() == gamepad_name {
                gamepad = Some(web_gamepad);
            }
        }
    }
    let Some(gamepad) = gamepad else {
        panic!("no gamepad found!");
    };

    let gamepad_js = JsValue::from(gamepad);
    let vibration_actuator_js =
        js_sys::Reflect::get(&gamepad_js, &JsValue::from_str("vibrationActuator")).unwrap();
    let play_effect_function_js =
        js_sys::Reflect::get(&vibration_actuator_js, &JsValue::from("playEffect")).unwrap();
    let play_effect_function: Function = play_effect_function_js
        .try_into()
        .expect("Failed to cast JsValue to Function");

    // set up playEffect args
    let play_effect_args = Array::new();

    play_effect_args.push(&JsValue::from_str("dual-rumble"));

    let rumble_vars_js_obj = JsValue::from(Object::new());
    js_sys::Reflect::set(
        &rumble_vars_js_obj,
        &JsValue::from("startDelay"),
        &JsValue::from(0),
    )
    .unwrap();
    js_sys::Reflect::set(
        &rumble_vars_js_obj,
        &JsValue::from("duration"),
        &JsValue::from(duration.as_millis() as u32),
    )
    .unwrap();
    js_sys::Reflect::set(
        &rumble_vars_js_obj,
        &JsValue::from("weakMagnitude"),
        &JsValue::from(weak_motor),
    )
    .unwrap();
    js_sys::Reflect::set(
        &rumble_vars_js_obj,
        &JsValue::from("strongMagnitude"),
        &JsValue::from(strong_motor),
    )
    .unwrap();
    play_effect_args.push(&rumble_vars_js_obj);

    // call playEffect function with args
    js_sys::Reflect::apply(
        &play_effect_function,
        &vibration_actuator_js,
        &play_effect_args,
    )
    .expect("Failed to call playEffect function");

    Ok(())
}
