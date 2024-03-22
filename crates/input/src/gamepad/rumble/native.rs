use std::time::Duration;

use gilrs::{ff, ff::Repeat};
use gilrs::ff::{BaseEffect, BaseEffectType, Replay};

use crate::gamepad::{rumble::RumbleError, converter::convert_gamepad_id, gilrs::GilrsWrapper};
use crate::GamepadRumbleIntensity;
use super::common::{GamepadRumbleRequest};

pub(crate) fn handle_rumble_request(
    rumble_request: GamepadRumbleRequest,
    input_gilrs: &mut GilrsWrapper,
) -> Result<(), RumbleError> {
    let gilrs = input_gilrs.gilrs_mut();
    let GamepadRumbleRequest {
        duration,
        intensity,
        gamepad,
    } = rumble_request;

    let (gamepad_id, _gamepad) = gilrs
        .gamepads()
        .find(|(pad_id, _)| convert_gamepad_id(*pad_id) == gamepad)
        .ok_or(RumbleError::GamepadNotFound)?;

    let mut effect_builder = ff::EffectBuilder::new();

    for effect in get_base_effects(intensity, duration) {
        effect_builder.add_effect(effect);
        effect_builder.repeat(Repeat::For(duration.into()));
    }

    let effect = effect_builder
        .gamepads(&[gamepad_id])
        .finish(gilrs)
        .map_err(|e| RumbleError::GilrsError(e))?;
    effect.play().map_err(|e| RumbleError::GilrsError(e))?;

    input_gilrs.add_rumble(&convert_gamepad_id(gamepad_id), duration, intensity, Some(effect));

    Ok(())
}

pub(crate) fn set_total_rumbles(
    _input_gilrs: &mut GilrsWrapper,
    _gamepad_ids: Vec<crate::GamepadId>,
) -> Result<(), RumbleError> {
    // do nothing, gilrs takes care of this on native for us!
    Ok(())
}

fn get_base_effects(
    GamepadRumbleIntensity {
        weak_motor,
        strong_motor,
    }: GamepadRumbleIntensity,
    duration: Duration,
) -> Vec<BaseEffect> {
    let mut effects = Vec::new();
    if strong_motor > 0. {
        effects.push(BaseEffect {
            kind: BaseEffectType::Strong {
                magnitude: to_gilrs_magnitude(strong_motor),
            },
            scheduling: Replay {
                play_for: duration.into(),
                ..Default::default()
            },
            ..Default::default()
        });
    }
    if weak_motor > 0. {
        effects.push(BaseEffect {
            kind: BaseEffectType::Strong {
                magnitude: to_gilrs_magnitude(weak_motor),
            },
            ..Default::default()
        });
    }
    effects
}

/// gilrs uses magnitudes from 0 to [`u16::MAX`], while ours go from `0.0` to `1.0` ([`f32`])
fn to_gilrs_magnitude(ratio: f32) -> u16 {
    (ratio * u16::MAX as f32) as u16
}