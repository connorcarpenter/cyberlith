/// Errors that occur when setting axis settings for gamepad input.
#[derive(Debug, PartialEq)]
pub enum AxisSettingsError {
    /// The given parameter `livezone_lowerbound` was not in range -1.0..=0.0.
    LiveZoneLowerBoundOutOfRange(f32),
    /// The given parameter `deadzone_lowerbound` was not in range -1.0..=0.0.
    DeadZoneLowerBoundOutOfRange(f32),
    /// The given parameter `deadzone_lowerbound` was not in range -1.0..=0.0.
    DeadZoneUpperBoundOutOfRange(f32),
    /// The given parameter `deadzone_lowerbound` was not in range -1.0..=0.0.
    LiveZoneUpperBoundOutOfRange(f32),
    /// Parameter `livezone_lowerbound` was not less than or equal to parameter `deadzone_lowerbound`.
    LiveZoneLowerBoundGreaterThanDeadZoneLowerBound {
        /// The value of the `livezone_lowerbound` parameter.
        livezone_lowerbound: f32,
        /// The value of the `deadzone_lowerbound` parameter.
        deadzone_lowerbound: f32,
    },
    ///  Parameter `deadzone_upperbound` was not less than or equal to parameter `livezone_upperbound`.
    DeadZoneUpperBoundGreaterThanLiveZoneUpperBound {
        /// The value of the `livezone_upperbound` parameter.
        livezone_upperbound: f32,
        /// The value of the `deadzone_upperbound` parameter.
        deadzone_upperbound: f32,
    },
    /// The given parameter was not in range 0.0..=2.0.
    Threshold(f32),
}

/// Errors that occur when setting button settings for gamepad input.
#[derive(Debug, PartialEq)]
pub enum ButtonSettingsError {
    /// The given parameter was not in range 0.0..=1.0.
    ReleaseThresholdOutOfRange(f32),
    /// The given parameter was not in range 0.0..=1.0.
    PressThresholdOutOfRange(f32),
    /// Parameter `release_threshold` was not less than or equal to `press_threshold`.
    ReleaseThresholdGreaterThanPressThreshold {
        /// The value of the `press_threshold` parameter.
        press_threshold: f32,
        /// The value of the `release_threshold` parameter.
        release_threshold: f32,
    },
}
