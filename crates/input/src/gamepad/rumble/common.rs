//! Handle user specified rumble request events.
use std::{collections::HashMap, time::Duration};

use bevy_ecs::system::{NonSendMut, ResMut, Resource};
use logging::warn;

use gilrs::ff::{self, Effect};

use instant::Instant;

use crate::{
    gamepad::{gilrs::GilrsWrapper, rumble::handle_rumble_request},
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
        self.requests
            .push(GamepadRumbleRequest::new(duration, intensity, gamepad));
    }

    // will be used as a system
    pub(crate) fn update(
        mut input_gilrs: NonSendMut<GilrsWrapper>,
        mut rumble_manager: ResMut<RumbleManager>,
    ) {
        // handle all new rumble requests
        let new_rumble_events = rumble_manager.take_new_rumble_events();
        for rumble_request in new_rumble_events {
            let gamepad = rumble_request.gamepad();
            let rumble_result = handle_rumble_request(rumble_request, &mut input_gilrs);

            // handle errors
            match rumble_result {
                Ok(()) => {}
                Err(RumbleError::GilrsError(err)) => {
                    if let ff::Error::FfNotSupported(_) = err {
                        warn!("Tried to rumble {gamepad:?}, but it doesn't support force feedback");
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

        // update existing rumbles
        input_gilrs.update_rumbles();
    }

    fn take_new_rumble_events(&mut self) -> Vec<GamepadRumbleRequest> {
        std::mem::take(&mut self.requests)
    }
}

pub(crate) struct GamepadRumbleRequest {
    /// Add a rumble to the given gamepad.
    ///
    /// Simultaneous rumble effects add up to the sum of their strengths.
    ///
    /// Consequently, if two rumbles at half intensity are added at the same
    /// time, their intensities will be added up, and the controller will rumble
    /// at full intensity until one of the rumbles finishes, then the rumble
    /// will continue at the intensity of the remaining event.

    /// How long the gamepad should rumble.
    pub(crate) duration: Duration,
    /// How intense the rumble should be.
    pub(crate) intensity: GamepadRumbleIntensity,
    /// The gamepad to rumble.
    pub(crate) gamepad: GamepadId,
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

    pub fn is_none(&self) -> bool {
        self.strong_motor == 0.0 && self.weak_motor == 0.0
    }
}

/// A rumble effect that is currently in effect.
pub(crate) struct RunningRumble {
    /// Duration from app startup when this effect will be finished
    pub(crate) deadline: Instant,
    pub(crate) intensity: GamepadRumbleIntensity,
    /// A ref-counted handle to the specific force-feedback effect
    ///
    /// Dropping it will cause the effect to stop
    #[allow(dead_code)]
    pub(crate) effect: Option<Effect>,
}

#[derive(Debug)]
pub(crate) enum RumbleError {
    GamepadNotFound,
    #[allow(unused)]
    GilrsError(ff::Error),
}

/// Contains the gilrs rumble effects that are currently running for each gamepad
pub(crate) struct RunningRumbleEffects {
    /// If multiple rumbles are running at the same time, their resulting rumble
    /// will be the saturated sum of their strengths up until [`u16::MAX`]
    rumbles: HashMap<GamepadId, GamepadRunningRumbleEffects>,
}

impl Default for RunningRumbleEffects {
    fn default() -> Self {
        Self {
            rumbles: HashMap::new(),
        }
    }
}

impl RunningRumbleEffects {
    // unused in native, used in wasm
    #[allow(unused)]
    pub fn get_current_rumble(
        &self,
        gamepad_id: &GamepadId,
    ) -> Option<(Duration, GamepadRumbleIntensity)> {
        if let Some(rumbles) = self.rumbles.get(gamepad_id) {
            let now = Instant::now();
            let duration = rumbles.last_deadline.until(&now);
            let current_rumble = rumbles.current_rumble;
            if duration.as_millis() == 0 || current_rumble.is_none() {
                return None;
            }
            return Some((duration, rumbles.current_rumble));
        } else {
            return None;
        }
    }

    pub(crate) fn add_rumble(
        &mut self,
        id: &GamepadId,
        duration: Duration,
        intensity: GamepadRumbleIntensity,
        effect: Option<Effect>,
    ) {
        self.rumbles
            .entry(*id)
            .or_insert_with(GamepadRunningRumbleEffects::default)
            .add_rumble(duration, intensity, effect);
    }

    // returns list of gamepads that should be updated
    pub(crate) fn update(&mut self) -> Option<Vec<GamepadId>> {
        let mut output = None;

        let now = Instant::now();
        // Remove outdated rumble effects.
        for (id, gamepad_effects) in self.rumbles.iter_mut() {
            if gamepad_effects.update(&now) {
                if output.is_none() {
                    output = Some(Vec::new());
                }
                output.as_mut().unwrap().push(*id);
            }
        }

        output
    }

    pub(crate) fn cleanup(&mut self) {
        self.rumbles.retain(|_gamepad, rumbles| !rumbles.is_empty());
    }
}

pub(crate) struct GamepadRunningRumbleEffects {
    rumbles: Vec<RunningRumble>,
    current_rumble: GamepadRumbleIntensity,
    last_deadline: Instant,
    rumble_is_dirty: bool,
}

impl Default for GamepadRunningRumbleEffects {
    fn default() -> Self {
        Self {
            rumbles: Vec::new(),
            current_rumble: GamepadRumbleIntensity {
                strong_motor: 0.0,
                weak_motor: 0.0,
            },
            last_deadline: Instant::now(),
            rumble_is_dirty: false,
        }
    }
}

impl GamepadRunningRumbleEffects {
    pub(crate) fn add_rumble(
        &mut self,
        duration: Duration,
        intensity: GamepadRumbleIntensity,
        effect: Option<Effect>,
    ) {
        let duration_millis = duration.as_millis();
        let mut real_deadline = Instant::now();
        real_deadline.add_millis(duration_millis as u32);

        if real_deadline > self.last_deadline {
            self.last_deadline = real_deadline.clone();
            self.rumble_is_dirty = true;
        }

        let mut used_deadline = real_deadline.clone();
        used_deadline.add_millis(20);

        self.rumbles.push(RunningRumble {
            deadline: used_deadline,
            effect,
            intensity,
        });

        self.add_rumble_intensity(intensity);
    }

    // returns whether new rumble should be set
    pub(crate) fn update(&mut self, now: &Instant) -> bool {
        let old_rumbles = std::mem::take(&mut self.rumbles);

        for rumble in old_rumbles {
            if rumble.deadline > *now {
                self.rumbles.push(rumble);
            } else {
                let intensity = rumble.intensity;
                self.remove_rumble_intensity(intensity);
            }
        }

        // check if rumbling has changed
        if self.rumble_is_dirty {
            self.rumble_is_dirty = false;

            return true;
        } else {
            return false;
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.rumbles.is_empty()
    }

    fn add_rumble_intensity(&mut self, intensity: GamepadRumbleIntensity) {
        self.current_rumble.strong_motor += intensity.strong_motor;
        self.current_rumble.weak_motor += intensity.weak_motor;
        self.rumble_is_dirty = true;
    }

    fn remove_rumble_intensity(&mut self, intensity: GamepadRumbleIntensity) {
        self.current_rumble.strong_motor -= intensity.strong_motor;
        self.current_rumble.weak_motor -= intensity.weak_motor;
        self.rumble_is_dirty = true;
    }
}
