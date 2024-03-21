//! The generic axis type.

use std::{hash::Hash, collections::HashMap};
use crate::GamepadId;

/// An array of every [`GamepadAxisType`] variant.
pub(crate) const ALL_AXIS_TYPES: [GamepadAxisType; 6] = [
    GamepadAxisType::LeftStickX,
    GamepadAxisType::LeftStickY,
    GamepadAxisType::LeftTrigger,
    GamepadAxisType::RightStickX,
    GamepadAxisType::RightStickY,
    GamepadAxisType::RightTrigger,
];

/// A type of a [`GamepadAxis`].
///
/// ## Usage
///
/// This is used to determine which axis has changed its value when receiving a
/// [`GamepadAxisChangedEvent`]. It is also used in the [`GamepadAxis`]
/// which in turn is used to create the [`Axis<GamepadAxis>`] `bevy` resource.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum GamepadAxisType {
    /// The horizontal value of the left stick.
    LeftStickX,
    /// The vertical value of the left stick.
    LeftStickY,
    /// The value of the left trigger button.
    LeftTrigger,

    /// The horizontal value of the right stick.
    RightStickX,
    /// The vertical value of the right stick.
    RightStickY,
    /// The value of the right trigger button.
    RightTrigger,

    /// Non-standard support for other axis types (i.e. HOTAS sliders, potentiometers, etc).
    Other(u8),
}

/// An axis of a [`GamepadId`].
///
/// ## Usage
///
/// It is used as the generic `T` value of an [`Axis`] to create `bevy` resources. These
/// resources store the data of the axes of a gamepad and can be accessed inside of a system.
///
/// ## Updating
///
/// The gamepad axes resources are updated inside of the [`gamepad_axis_event_system`].
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct GamepadAxis {
    /// The gamepad on which the axis is located on.
    pub gamepad: GamepadId,
    /// The type of the axis.
    pub axis_type: GamepadAxisType,
}

impl GamepadAxis {
    /// Creates a new [`GamepadAxis`].
    pub fn new(gamepad: GamepadId, axis_type: GamepadAxisType) -> Self {
        Self { gamepad, axis_type }
    }
}

/// Stores the position data of the input devices of type `T`.
///
/// The values are stored as `f32`s, using [`Axis::set`].
/// Use [`Axis::get`] to retrieve the value clamped between [`Axis::MIN`] and [`Axis::MAX`]
/// inclusive, or unclamped using [`Axis::get_unclamped`].
#[derive(Debug)]
pub(crate) struct Axis<T> {
    /// The position data of the input devices.
    axis_data: HashMap<T, f32>,
}

impl<T> Default for Axis<T>
where
    T: Copy + Eq + Hash,
{
    fn default() -> Self {
        Axis {
            axis_data: HashMap::default(),
        }
    }
}

impl<T> Axis<T>
where
    T: Copy + Eq + Hash,
{
    /// The smallest possible axis value.
    pub const MIN: f32 = -1.0;

    /// The largest possible axis value.
    pub const MAX: f32 = 1.0;

    /// Sets the position data of the `input_device` to `position_data`.
    ///
    /// If the `input_device`:
    /// - was present before, the position data is updated, and the old value is returned.
    /// - wasn't present before, `None` is returned.
    pub fn set(&mut self, input_device: T, position_data: f32) -> Option<f32> {
        self.axis_data.insert(input_device, position_data)
    }

    /// Returns the position data of the provided `input_device`.
    ///
    /// This will be clamped between [`Axis::MIN`] and [`Axis::MAX`] inclusive.
    pub fn get(&self, input_device: T) -> Option<f32> {
        self.axis_data
            .get(&input_device)
            .copied()
            .map(|value| value.clamp(Self::MIN, Self::MAX))
    }

    // /// Returns the unclamped position data of the provided `input_device`.
    // ///
    // /// This value may be outside of the [`Axis::MIN`] and [`Axis::MAX`] range.
    // ///
    // /// Use for things like camera zoom, where you want devices like mouse wheels to be able to
    // /// exceed the normal range. If being able to move faster on one input device
    // /// than another would give an unfair advantage, you should likely use [`Axis::get`] instead.
    // pub fn get_unclamped(&self, input_device: T) -> Option<f32> {
    //     self.axis_data.get(&input_device).copied()
    // }

    /// Removes the position data of the `input_device`, returning the position data if the input device was previously set.
    pub fn remove(&mut self, input_device: T) -> Option<f32> {
        self.axis_data.remove(&input_device)
    }

    // /// Returns an iterator of all the input devices that have position data
    // pub fn devices(&self) -> impl ExactSizeIterator<Item = &T> {
    //     self.axis_data.keys()
    // }
}