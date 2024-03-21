//! Handle user specified rumble request events.
use std::{collections::HashMap, time::Duration};

use bevy_ecs::{
    prelude::Res,
    system::{NonSendMut, ResMut, Resource},
};
use bevy_log::{debug, warn};

use gilrs::ff::{self, BaseEffect, BaseEffectType, Repeat, Replay};

use render_api::resources::Time;

use crate::{
    gamepad::{converter::convert_gamepad_id, gilrs::GilrsWrapper},
    GamepadId,
};

#[derive(Resource)]
pub struct RumbleManager {
    requests: Vec<GamepadRumbleRequest>,
}

impl Default for RumbleManager {
    fn default() -> Self {
        Self {
            requests: Vec::new(),
        }
    }
}

impl RumbleManager {
    // User API
    pub fn add_rumble(
        &mut self,
        gamepad: GamepadId,
        duration: Duration,
        intensity: GamepadRumbleIntensity,
    ) {
        self.requests.push(GamepadRumbleRequest::new(
            duration,
            intensity,
            gamepad,
        ));
    }

    // will be used as a system
    pub(crate) fn update(
        time: Res<Time>,
        mut input_gilrs: NonSendMut<GilrsWrapper>,
        mut rumble_manager: ResMut<RumbleManager>,
    ) {
        let current_time = time.get_elapsed_ms();
        // Remove outdated rumble effects.
        for rumbles in input_gilrs.rumbles_mut().rumbles.values_mut() {
            // `ff::Effect` uses RAII, dropping = deactivating
            rumbles.retain(|RunningRumble { deadline, .. }| {
                (deadline.as_millis() as f32) >= current_time
            });
        }
        input_gilrs
            .rumbles_mut()
            .rumbles
            .retain(|_gamepad, rumbles| !rumbles.is_empty());

        // handle all new rumble requests
        let new_rumble_events = rumble_manager.take_new_rumble_events();
        for rumble_request in new_rumble_events {
            let gamepad = rumble_request.gamepad();
            let rumble_result = Self::handle_rumble_request(rumble_request, &mut input_gilrs);

            // handle errors
            match rumble_result {
                Ok(()) => {}
                Err(RumbleError::GilrsError(err)) => {
                    if let ff::Error::FfNotSupported(_) = err {
                        debug!(
                            "Tried to rumble {gamepad:?}, but it doesn't support force feedback"
                        );
                    } else {
                        warn!(
                        "Tried to handle rumble request for {gamepad:?} but an error occurred: {err}"
                        );
                    }
                }
                Err(RumbleError::GamepadNotFound) => {
                    warn!("Tried to handle rumble request {gamepad:?} but it doesn't exist!");
                }
            };
        }
    }

    fn take_new_rumble_events(&mut self) -> Vec<GamepadRumbleRequest> {
        std::mem::take(&mut self.requests)
    }

    fn handle_rumble_request(
        rumble_request: GamepadRumbleRequest,
        input_gilrs: &mut GilrsWrapper,
    ) -> Result<(), RumbleError> {
        let gilrs = input_gilrs.gilrs_mut();
        let GamepadRumbleRequest {
            duration,
            intensity,
            gamepad,
        } = rumble_request;

        let (gamepad_id, _) = gilrs
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

        let gamepad_rumbles = input_gilrs
            .rumbles_mut()
            .rumbles
            .entry(convert_gamepad_id(gamepad_id))
            .or_default();
        let deadline = Duration::from_millis(20) + duration;
        gamepad_rumbles.push(RunningRumble { deadline, effect });

        Ok(())
    }
}

struct GamepadRumbleRequest {
    /// Add a rumble to the given gamepad.
    ///
    /// Simultaneous rumble effects add up to the sum of their strengths.
    ///
    /// Consequently, if two rumbles at half intensity are added at the same
    /// time, their intensities will be added up, and the controller will rumble
    /// at full intensity until one of the rumbles finishes, then the rumble
    /// will continue at the intensity of the remaining event.

    /// How long the gamepad should rumble.
    duration: Duration,
    /// How intense the rumble should be.
    intensity: GamepadRumbleIntensity,
    /// The gamepad to rumble.
    gamepad: GamepadId,
}

impl GamepadRumbleRequest {
    fn new(duration: Duration, intensity: GamepadRumbleIntensity, gamepad: GamepadId) -> Self {
        Self {
            duration,
            intensity,
            gamepad,
        }
    }

    fn gamepad(&self) -> GamepadId {
        self.gamepad
    }
}

/// The intensity at which a gamepad's force-feedback motors may rumble.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GamepadRumbleIntensity {
    /// The rumble intensity of the strong gamepad motor.
    ///
    /// Ranges from `0.0` to `1.0`.
    ///
    /// By convention, this is usually a low-frequency motor on the left-hand
    /// side of the gamepad, though it may vary across platforms and hardware.
    pub strong_motor: f32,
    /// The rumble intensity of the weak gamepad motor.
    ///
    /// Ranges from `0.0` to `1.0`.
    ///
    /// By convention, this is usually a high-frequency motor on the right-hand
    /// side of the gamepad, though it may vary across platforms and hardware.
    pub weak_motor: f32,
}

impl GamepadRumbleIntensity {
    /// Rumble both gamepad motors at maximum intensity.
    pub const MAX: Self = GamepadRumbleIntensity {
        strong_motor: 1.0,
        weak_motor: 1.0,
    };

    /// Rumble the weak motor at maximum intensity.
    pub const WEAK_MAX: Self = GamepadRumbleIntensity {
        strong_motor: 0.0,
        weak_motor: 1.0,
    };

    /// Rumble the strong motor at maximum intensity.
    pub const STRONG_MAX: Self = GamepadRumbleIntensity {
        strong_motor: 1.0,
        weak_motor: 0.0,
    };

    /// Creates a new rumble intensity with weak motor intensity set to the given value.
    ///
    /// Clamped within the `0.0` to `1.0` range.
    pub const fn weak_motor(intensity: f32) -> Self {
        Self {
            weak_motor: intensity,
            strong_motor: 0.0,
        }
    }

    /// Creates a new rumble intensity with strong motor intensity set to the given value.
    ///
    /// Clamped within the `0.0` to `1.0` range.
    pub const fn strong_motor(intensity: f32) -> Self {
        Self {
            strong_motor: intensity,
            weak_motor: 0.0,
        }
    }
}

/// A rumble effect that is currently in effect.
struct RunningRumble {
    /// Duration from app startup when this effect will be finished
    deadline: Duration,
    /// A ref-counted handle to the specific force-feedback effect
    ///
    /// Dropping it will cause the effect to stop
    #[allow(dead_code)]
    effect: ff::Effect,
}

#[derive(Debug)]
enum RumbleError {
    GamepadNotFound,
    GilrsError(ff::Error),
}

/// Contains the gilrs rumble effects that are currently running for each gamepad
#[derive(Default)]
pub(crate) struct RunningRumbleEffects {
    /// If multiple rumbles are running at the same time, their resulting rumble
    /// will be the saturated sum of their strengths up until [`u16::MAX`]
    rumbles: HashMap<GamepadId, Vec<RunningRumble>>,
}

/// gilrs uses magnitudes from 0 to [`u16::MAX`], while ours go from `0.0` to `1.0` ([`f32`])
fn to_gilrs_magnitude(ratio: f32) -> u16 {
    (ratio * u16::MAX as f32) as u16
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
