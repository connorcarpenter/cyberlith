use bevy_ecs::system::Resource;

/// Selects the level of hardware graphics acceleration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HardwareAcceleration {
    /// Require graphics acceleration.
    Required,
    /// Prefer graphics acceleration, but fall back to software.
    Preferred,
    /// Do NOT use graphics acceleration.
    /// On some platforms (MacOS) this is ignored and treated the same as
    /// [Self::Preferred].
    /// On web, "willReadFrequently" is set to true.
    Off,
}

/// Settings controlling the behavior of the surface on where to draw, to present it on the screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub struct SurfaceSettings {
    /// Sets the number of bits in the depth buffer.
    /// A value of 0 means no depth buffer.
    /// The default value is 24.
    /// On web, this can only be off (0) or on (>0).
    pub depth_buffer: u8,
    /// Sets the number of bits in the stencil buffer.
    /// A value of 0 means no stencil buffer.
    /// The default value is 0.
    /// On web, this can only be off (0) or on (>0).
    pub stencil_buffer: u8,
    /// Specify whether or not hardware acceleration is preferred, required, or
    /// off. The default is [HardwareAcceleration::Preferred].
    pub hardware_acceleration: HardwareAcceleration,
}

impl Default for SurfaceSettings {
    fn default() -> Self {
        Self {
            depth_buffer: 24,
            stencil_buffer: 0,
            hardware_acceleration: HardwareAcceleration::Required,
        }
    }
}

///
/// Window settings.
///
#[derive(Debug, Clone, PartialEq, Eq, Resource)]
pub struct WindowSettings {
    /// The title of the window.
    ///
    /// On web this has no effect.
    pub title: String,
    /// The minimum size of the window `(width, height)`, in logical pixels.
    ///
    /// On web this has no effect.
    pub min_size: (u32, u32),
    /// The maximum and initial size of the window `(width, height)`, in logical pixels.
    /// If `None` is specified, the window is maximized.
    ///
    /// On web, the size will be applied to the [canvas][WindowSettings::canvas], in logical pixels.
    /// If `None` is specified, the canvas will be resized to the same size as
    /// the owner `Window`'s inner width and height.
    pub max_size: Option<(u32, u32)>,

    /// Settings related to the surface on where to draw.
    pub surface_settings: SurfaceSettings,
}
impl Default for WindowSettings {
    fn default() -> Self {
        Self {
            title: "".to_string(),
            min_size: (2, 2),
            max_size: None,
            surface_settings: SurfaceSettings::default(),
        }
    }
}
