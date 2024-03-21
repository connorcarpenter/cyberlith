use std::collections::HashMap;

use crate::{gamepad::error::{AxisSettingsError, ButtonSettingsError}, GamepadAxis, GamepadButton};

#[derive(Default, Debug)]
pub struct GamepadSettings {
    /// The default button settings.
    pub default_button_settings: ButtonSettings,
    /// The default axis settings.
    pub default_axis_settings: AxisSettings,
    /// The default button axis settings.
    pub default_button_axis_settings: ButtonAxisSettings,
    /// The user defined button settings.
    pub button_settings: HashMap<GamepadButton, ButtonSettings>,
    /// The user defined axis settings.
    pub axis_settings: HashMap<GamepadAxis, AxisSettings>,
    /// The user defined button axis settings.
    pub button_axis_settings: HashMap<GamepadButton, ButtonAxisSettings>,
}

impl GamepadSettings {

    pub fn get_button_settings(&self, button: GamepadButton) -> &ButtonSettings {
        self.button_settings
            .get(&button)
            .unwrap_or(&self.default_button_settings)
    }

    pub fn get_axis_settings(&self, axis: GamepadAxis) -> &AxisSettings {
        self.axis_settings
            .get(&axis)
            .unwrap_or(&self.default_axis_settings)
    }

    pub fn get_button_axis_settings(&self, button: GamepadButton) -> &ButtonAxisSettings {
        self.button_axis_settings
            .get(&button)
            .unwrap_or(&self.default_button_axis_settings)
    }
}

/// Manages settings for gamepad buttons.
///
/// It is used inside of [`GamepadSettings`] to define the threshold for a gamepad button
/// to be considered pressed or released. A button is considered pressed if the `press_threshold`
/// value is surpassed and released if the `release_threshold` value is undercut.
///
/// Allowed values: `0.0 <= ``release_threshold`` <= ``press_threshold`` <= 1.0`
#[derive(Debug, Clone)]
pub struct ButtonSettings {
    press_threshold: f32,
    release_threshold: f32,
}

impl Default for ButtonSettings {
    fn default() -> Self {
        ButtonSettings {
            press_threshold: 0.75,
            release_threshold: 0.65,
        }
    }
}

impl ButtonSettings {
    /// Creates a new [`ButtonSettings`] instance.
    ///
    /// # Parameters
    ///
    /// + `press_threshold` is the button input value above which the button is considered pressed.
    /// + `release_threshold` is the button input value below which the button is considered released.
    ///
    /// Restrictions:
    /// + `0.0 <= ``release_threshold`` <= ``press_threshold`` <= 1.0`
    ///
    /// # Errors
    ///
    /// If the restrictions are not met, returns one of
    /// `GamepadSettingsError::ButtonReleaseThresholdOutOfRange`,
    /// `GamepadSettingsError::ButtonPressThresholdOutOfRange`, or
    /// `GamepadSettingsError::ButtonReleaseThresholdGreaterThanPressThreshold`.
    pub fn new(
        press_threshold: f32,
        release_threshold: f32,
    ) -> Result<ButtonSettings, ButtonSettingsError> {
        if !(0.0..=1.0).contains(&release_threshold) {
            Err(ButtonSettingsError::ReleaseThresholdOutOfRange(
                release_threshold,
            ))
        } else if !(0.0..=1.0).contains(&press_threshold) {
            Err(ButtonSettingsError::PressThresholdOutOfRange(
                press_threshold,
            ))
        } else if release_threshold > press_threshold {
            Err(
                ButtonSettingsError::ReleaseThresholdGreaterThanPressThreshold {
                    press_threshold,
                    release_threshold,
                },
            )
        } else {
            Ok(ButtonSettings {
                press_threshold,
                release_threshold,
            })
        }
    }

    /// Returns `true` if the button is pressed.
    ///
    /// A button is considered pressed if the `value` passed is greater than or equal to the press threshold.
    pub fn is_pressed(&self, value: f32) -> bool {
        value >= self.press_threshold
    }

    /// Returns `true` if the button is released.
    ///
    /// A button is considered released if the `value` passed is lower than or equal to the release threshold.
    pub fn is_released(&self, value: f32) -> bool {
        value <= self.release_threshold
    }

    /// Get the button input threshold above which the button is considered pressed.
    pub fn press_threshold(&self) -> f32 {
        self.press_threshold
    }

    /// Try to set the button input threshold above which the button is considered pressed.
    ///
    /// # Errors
    ///
    /// If the value passed is outside the range [release threshold..=1.0], returns either
    /// `GamepadSettingsError::ButtonPressThresholdOutOfRange` or
    /// `GamepadSettingsError::ButtonReleaseThresholdGreaterThanPressThreshold`.
    pub fn try_set_press_threshold(&mut self, value: f32) -> Result<(), ButtonSettingsError> {
        if (self.release_threshold..=1.0).contains(&value) {
            self.press_threshold = value;
            Ok(())
        } else if !(0.0..1.0).contains(&value) {
            Err(ButtonSettingsError::PressThresholdOutOfRange(value))
        } else {
            Err(
                ButtonSettingsError::ReleaseThresholdGreaterThanPressThreshold {
                    press_threshold: value,
                    release_threshold: self.release_threshold,
                },
            )
        }
    }

    /// Try to set the button input threshold above which the button is considered pressed.
    /// If the value passed is outside the range [release threshold..=1.0], the value will not be changed.
    ///
    /// Returns the new value of the press threshold.
    pub fn set_press_threshold(&mut self, value: f32) -> f32 {
        self.try_set_press_threshold(value).ok();
        self.press_threshold
    }

    /// Get the button input threshold below which the button is considered released.
    pub fn release_threshold(&self) -> f32 {
        self.release_threshold
    }

    /// Try to set the button input threshold below which the button is considered released.
    ///
    /// # Errors
    ///
    /// If the value passed is outside the range [0.0..=press threshold], returns
    /// `ButtonSettingsError::ReleaseThresholdOutOfRange` or
    /// `ButtonSettingsError::ReleaseThresholdGreaterThanPressThreshold`.
    pub fn try_set_release_threshold(&mut self, value: f32) -> Result<(), ButtonSettingsError> {
        if (0.0..=self.press_threshold).contains(&value) {
            self.release_threshold = value;
            Ok(())
        } else if !(0.0..1.0).contains(&value) {
            Err(ButtonSettingsError::ReleaseThresholdOutOfRange(value))
        } else {
            Err(
                ButtonSettingsError::ReleaseThresholdGreaterThanPressThreshold {
                    press_threshold: self.press_threshold,
                    release_threshold: value,
                },
            )
        }
    }

    /// Try to set the button input threshold below which the button is considered released. If the
    /// value passed is outside the range [0.0..=press threshold], the value will not be changed.
    ///
    /// Returns the new value of the release threshold.
    pub fn set_release_threshold(&mut self, value: f32) -> f32 {
        self.try_set_release_threshold(value).ok();
        self.release_threshold
    }
}

/// Settings for a [`GamepadAxis`].
///
/// It is used inside of the [`GamepadSettings`] to define the sensitivity range and
/// threshold for an axis.
/// Values that are higher than `livezone_upperbound` will be rounded up to 1.0.
/// Values that are lower than `livezone_lowerbound` will be rounded down to -1.0.
/// Values that are in-between `deadzone_lowerbound` and `deadzone_upperbound` will be rounded
/// to 0.0.
/// Otherwise, values will not be rounded.
///
/// The valid range is `[-1.0, 1.0]`.
#[derive(Debug, Clone, PartialEq)]
pub struct AxisSettings {
    /// Values that are higher than `livezone_upperbound` will be rounded up to 1.0.
    livezone_upperbound: f32,
    /// Positive values that are less than `deadzone_upperbound` will be rounded down to 0.0.
    deadzone_upperbound: f32,
    /// Negative values that are greater than `deadzone_lowerbound` will be rounded up to 0.0.
    deadzone_lowerbound: f32,
    /// Values that are lower than `livezone_lowerbound` will be rounded down to -1.0.
    livezone_lowerbound: f32,
    /// `threshold` defines the minimum difference between old and new values to apply the changes.
    threshold: f32,
}

impl Default for AxisSettings {
    fn default() -> Self {
        AxisSettings {
            livezone_upperbound: 1.0,
            deadzone_upperbound: 0.05,
            deadzone_lowerbound: -0.05,
            livezone_lowerbound: -1.0,
            threshold: 0.01,
        }
    }
}

impl AxisSettings {
    /// Creates a new [`AxisSettings`] instance.
    ///
    /// # Arguments
    ///
    /// + `livezone_lowerbound` - the value below which inputs will be rounded down to -1.0.
    /// + `deadzone_lowerbound` - the value above which negative inputs will be rounded up to 0.0.
    /// + `deadzone_upperbound` - the value below which positive inputs will be rounded down to 0.0.
    /// + `livezone_upperbound` - the value above which inputs will be rounded up to 1.0.
    /// + `threshold` - the minimum value by which input must change before the change is registered.
    ///
    /// Restrictions:
    ///
    /// + `-1.0 <= livezone_lowerbound <= deadzone_lowerbound <= 0.0`
    /// + `0.0 <= deadzone_upperbound <= livezone_upperbound <= 1.0`
    /// + `0.0 <= threshold <= 2.0`
    ///
    /// # Errors
    ///
    /// Returns an [`AxisSettingsError`] if any restrictions on the zone values are not met.
    /// If the zone restrictions are met, but the `threshold` value restrictions are not met,
    /// returns [`AxisSettingsError::Threshold`].
    pub fn new(
        livezone_lowerbound: f32,
        deadzone_lowerbound: f32,
        deadzone_upperbound: f32,
        livezone_upperbound: f32,
        threshold: f32,
    ) -> Result<AxisSettings, AxisSettingsError> {
        if !(-1.0..=0.0).contains(&livezone_lowerbound) {
            Err(AxisSettingsError::LiveZoneLowerBoundOutOfRange(
                livezone_lowerbound,
            ))
        } else if !(-1.0..=0.0).contains(&deadzone_lowerbound) {
            Err(AxisSettingsError::DeadZoneLowerBoundOutOfRange(
                deadzone_lowerbound,
            ))
        } else if !(0.0..=1.0).contains(&deadzone_upperbound) {
            Err(AxisSettingsError::DeadZoneUpperBoundOutOfRange(
                deadzone_upperbound,
            ))
        } else if !(0.0..=1.0).contains(&livezone_upperbound) {
            Err(AxisSettingsError::LiveZoneUpperBoundOutOfRange(
                livezone_upperbound,
            ))
        } else if livezone_lowerbound > deadzone_lowerbound {
            Err(
                AxisSettingsError::LiveZoneLowerBoundGreaterThanDeadZoneLowerBound {
                    livezone_lowerbound,
                    deadzone_lowerbound,
                },
            )
        } else if deadzone_upperbound > livezone_upperbound {
            Err(
                AxisSettingsError::DeadZoneUpperBoundGreaterThanLiveZoneUpperBound {
                    livezone_upperbound,
                    deadzone_upperbound,
                },
            )
        } else if !(0.0..=2.0).contains(&threshold) {
            Err(AxisSettingsError::Threshold(threshold))
        } else {
            Ok(Self {
                livezone_lowerbound,
                deadzone_lowerbound,
                deadzone_upperbound,
                livezone_upperbound,
                threshold,
            })
        }
    }

    /// Get the value above which inputs will be rounded up to 1.0.
    pub fn livezone_upperbound(&self) -> f32 {
        self.livezone_upperbound
    }

    /// Try to set the value above which inputs will be rounded up to 1.0.
    ///
    /// # Errors
    ///
    /// If the value passed is less than the dead zone upper bound,
    /// returns `AxisSettingsError::DeadZoneUpperBoundGreaterThanLiveZoneUpperBound`.
    /// If the value passed is not in range [0.0..=1.0], returns `AxisSettingsError::LiveZoneUpperBoundOutOfRange`.
    pub fn try_set_livezone_upperbound(&mut self, value: f32) -> Result<(), AxisSettingsError> {
        if !(0.0..=1.0).contains(&value) {
            Err(AxisSettingsError::LiveZoneUpperBoundOutOfRange(value))
        } else if value < self.deadzone_upperbound {
            Err(
                AxisSettingsError::DeadZoneUpperBoundGreaterThanLiveZoneUpperBound {
                    livezone_upperbound: value,
                    deadzone_upperbound: self.deadzone_upperbound,
                },
            )
        } else {
            self.livezone_upperbound = value;
            Ok(())
        }
    }

    /// Try to set the value above which inputs will be rounded up to 1.0.
    /// If the value passed is negative or less than `deadzone_upperbound`,
    /// the value will not be changed.
    ///
    /// Returns the new value of `livezone_upperbound`.
    pub fn set_livezone_upperbound(&mut self, value: f32) -> f32 {
        self.try_set_livezone_upperbound(value).ok();
        self.livezone_upperbound
    }

    /// Get the value below which positive inputs will be rounded down to 0.0.
    pub fn deadzone_upperbound(&self) -> f32 {
        self.deadzone_upperbound
    }

    /// Try to set the value below which positive inputs will be rounded down to 0.0.
    ///
    /// # Errors
    ///
    /// If the value passed is greater than the live zone upper bound,
    /// returns `AxisSettingsError::DeadZoneUpperBoundGreaterThanLiveZoneUpperBound`.
    /// If the value passed is not in range [0.0..=1.0], returns `AxisSettingsError::DeadZoneUpperBoundOutOfRange`.
    pub fn try_set_deadzone_upperbound(&mut self, value: f32) -> Result<(), AxisSettingsError> {
        if !(0.0..=1.0).contains(&value) {
            Err(AxisSettingsError::DeadZoneUpperBoundOutOfRange(value))
        } else if self.livezone_upperbound < value {
            Err(
                AxisSettingsError::DeadZoneUpperBoundGreaterThanLiveZoneUpperBound {
                    livezone_upperbound: self.livezone_upperbound,
                    deadzone_upperbound: value,
                },
            )
        } else {
            self.deadzone_upperbound = value;
            Ok(())
        }
    }

    /// Try to set the value below which positive inputs will be rounded down to 0.0.
    /// If the value passed is negative or greater than `livezone_upperbound`,
    /// the value will not be changed.
    ///
    /// Returns the new value of `deadzone_upperbound`.
    pub fn set_deadzone_upperbound(&mut self, value: f32) -> f32 {
        self.try_set_deadzone_upperbound(value).ok();
        self.deadzone_upperbound
    }

    /// Get the value below which negative inputs will be rounded down to -1.0.
    pub fn livezone_lowerbound(&self) -> f32 {
        self.livezone_lowerbound
    }

    /// Try to set the value below which negative inputs will be rounded down to -1.0.
    ///
    /// # Errors
    ///
    /// If the value passed is less than the dead zone lower bound,
    /// returns `AxisSettingsError::LiveZoneLowerBoundGreaterThanDeadZoneLowerBound`.
    /// If the value passed is not in range [-1.0..=0.0], returns `AxisSettingsError::LiveZoneLowerBoundOutOfRange`.
    pub fn try_set_livezone_lowerbound(&mut self, value: f32) -> Result<(), AxisSettingsError> {
        if !(-1.0..=0.0).contains(&value) {
            Err(AxisSettingsError::LiveZoneLowerBoundOutOfRange(value))
        } else if value > self.deadzone_lowerbound {
            Err(
                AxisSettingsError::LiveZoneLowerBoundGreaterThanDeadZoneLowerBound {
                    livezone_lowerbound: value,
                    deadzone_lowerbound: self.deadzone_lowerbound,
                },
            )
        } else {
            self.livezone_lowerbound = value;
            Ok(())
        }
    }

    /// Try to set the value below which negative inputs will be rounded down to -1.0.
    /// If the value passed is positive or greater than `deadzone_lowerbound`,
    /// the value will not be changed.
    ///
    /// Returns the new value of `livezone_lowerbound`.
    pub fn set_livezone_lowerbound(&mut self, value: f32) -> f32 {
        self.try_set_livezone_lowerbound(value).ok();
        self.livezone_lowerbound
    }

    /// Get the value above which inputs will be rounded up to 0.0.
    pub fn deadzone_lowerbound(&self) -> f32 {
        self.deadzone_lowerbound
    }

    /// Try to set the value above which inputs will be rounded up to 0.0.
    ///
    /// # Errors
    ///
    /// If the value passed is less than the live zone lower bound,
    /// returns `AxisSettingsError::LiveZoneLowerBoundGreaterThanDeadZoneLowerBound`.
    /// If the value passed is not in range [-1.0..=0.0], returns `AxisSettingsError::DeadZoneLowerBoundOutOfRange`.
    pub fn try_set_deadzone_lowerbound(&mut self, value: f32) -> Result<(), AxisSettingsError> {
        if !(-1.0..=0.0).contains(&value) {
            Err(AxisSettingsError::DeadZoneLowerBoundOutOfRange(value))
        } else if self.livezone_lowerbound > value {
            Err(
                AxisSettingsError::LiveZoneLowerBoundGreaterThanDeadZoneLowerBound {
                    livezone_lowerbound: self.livezone_lowerbound,
                    deadzone_lowerbound: value,
                },
            )
        } else {
            self.deadzone_lowerbound = value;
            Ok(())
        }
    }

    /// Try to set the value above which inputs will be rounded up to 0.0.
    /// If the value passed is less than -1.0 or less than `livezone_lowerbound`,
    /// the value will not be changed.
    ///
    /// Returns the new value of `deadzone_lowerbound`.
    pub fn set_deadzone_lowerbound(&mut self, value: f32) -> f32 {
        self.try_set_deadzone_lowerbound(value).ok();
        self.deadzone_lowerbound
    }

    /// Get the minimum value by which input must change before the change is registered.
    pub fn threshold(&self) -> f32 {
        self.threshold
    }

    /// Try to set the minimum value by which input must change before the change is registered.
    ///
    /// # Errors
    ///
    /// If the value passed is not within [0.0..=2.0], returns `GamepadSettingsError::AxisThreshold`.
    pub fn try_set_threshold(&mut self, value: f32) -> Result<(), AxisSettingsError> {
        if !(0.0..=2.0).contains(&value) {
            Err(AxisSettingsError::Threshold(value))
        } else {
            self.threshold = value;
            Ok(())
        }
    }

    /// Try to set the minimum value by which input must change before the changes will be applied.
    /// If the value passed is not within [0.0..=2.0], the value will not be changed.
    ///
    /// Returns the new value of threshold.
    pub fn set_threshold(&mut self, value: f32) -> f32 {
        self.try_set_threshold(value).ok();
        self.threshold
    }

    /// Clamps the `raw_value` according to the `AxisSettings`.
    pub fn clamp(&self, new_value: f32) -> f32 {
        if self.deadzone_lowerbound <= new_value && new_value <= self.deadzone_upperbound {
            0.0
        } else if new_value >= self.livezone_upperbound {
            1.0
        } else if new_value <= self.livezone_lowerbound {
            -1.0
        } else {
            new_value
        }
    }

    /// Determines whether the change from `old_value` to `new_value` should
    /// be registered as a change, according to the [`AxisSettings`].
    fn should_register_change(&self, new_value: f32, old_value: Option<f32>) -> bool {
        if old_value.is_none() {
            return true;
        }

        f32::abs(new_value - old_value.unwrap()) > self.threshold
    }

    /// Filters the `new_value` based on the `old_value`, according to the [`AxisSettings`].
    ///
    /// Returns the clamped `new_value` if the change exceeds the settings threshold,
    /// and `None` otherwise.
    pub fn filter(&self, new_value: f32, old_value: Option<f32>) -> Option<f32> {
        let new_value = self.clamp(new_value);

        if self.should_register_change(new_value, old_value) {
            return Some(new_value);
        }
        None
    }
}

/// Settings for a [`GamepadButton`].
///
/// It is used inside of the [`GamepadSettings`] to define the sensitivity range and
/// threshold for a button axis.
///
/// ## Logic
///
/// - Values that are higher than or equal to `high` will be rounded to 1.0.
/// - Values that are lower than or equal to `low` will be rounded to 0.0.
/// - Otherwise, values will not be rounded.
///
/// The valid range is from 0.0 to 1.0, inclusive.
///
/// ## Updating
///
/// The current value of a button is received through the [`GamepadButtonChangedEvent`].
#[derive(Debug, Clone)]
pub struct ButtonAxisSettings {
    /// The high value at which to apply rounding.
    pub high: f32,
    /// The low value at which to apply rounding.
    pub low: f32,
    /// The threshold to apply rounding.
    pub threshold: f32,
}

impl Default for ButtonAxisSettings {
    fn default() -> Self {
        ButtonAxisSettings {
            high: 0.95,
            low: 0.05,
            threshold: 0.01,
        }
    }
}

impl ButtonAxisSettings {
    /// Clamps the `raw_value` according to the specified settings.
    ///
    /// If the `raw_value` is:
    /// - lower than or equal to `low` it will be rounded to 0.0.
    /// - higher than or equal to `high` it will be rounded to 1.0.
    /// - Otherwise it will not be rounded.
    fn clamp(&self, raw_value: f32) -> f32 {
        if raw_value <= self.low {
            return 0.0;
        }
        if raw_value >= self.high {
            return 1.0;
        }

        raw_value
    }

    /// Determines whether the change from an `old_value` to a `new_value` should
    /// be registered as a change event, according to the specified settings.
    fn should_register_change(&self, new_value: f32, old_value: Option<f32>) -> bool {
        if old_value.is_none() {
            return true;
        }

        f32::abs(new_value - old_value.unwrap()) > self.threshold
    }

    /// Filters the `new_value` based on the `old_value`, according to the [`ButtonAxisSettings`].
    ///
    /// Returns the clamped `new_value`, according to the [`ButtonAxisSettings`], if the change
    /// exceeds the settings threshold, and `None` otherwise.
    pub fn filter(&self, new_value: f32, old_value: Option<f32>) -> Option<f32> {
        let new_value = self.clamp(new_value);

        if self.should_register_change(new_value, old_value) {
            return Some(new_value);
        }
        None
    }
}