use winit::window::CursorIcon;

use input::IncomingEvent;
use render_api::{components::Viewport, WindowResolution};

use crate::core::*;

#[derive(Clone, Debug)]
pub enum OutgoingEvent {
    CursorChanged(CursorIcon),
}

///
/// Input from the window to the rendering (and whatever else needs it) each frame.
///
#[derive(Clone, Debug)]
pub struct FrameInput<T: 'static + Clone> {
    /// A list of [events](crate::Event) which has occurred since last frame.
    pub incoming_events: Vec<IncomingEvent<T>>,

    /// A list of Events which should be sent onwards
    pub outgoing_events: Vec<OutgoingEvent>,

    /// Milliseconds since last frame.
    pub elapsed_time: f64,

    /// Milliseconds accumulated time since start.
    pub accumulated_time: f64,

    /// Viewport of the window in physical pixels (not counting pixel ratio)
    pub physical_size: Viewport,

    /// Viewport of the window in logical pixels = size / pixel ratio
    pub logical_size: Viewport,

    /// Number of physical pixels for each logical pixel.
    pub device_pixel_ratio: f64,

    /// Whether or not this is the first frame.
    pub first_frame: bool,
}

impl<T: 'static + Clone> FrameInput<T> {

    pub fn screen(&self) -> RenderTarget {
        RenderTarget::screen(self.logical_size.width, self.logical_size.height)
    }

    pub fn window_resolution(&self) -> WindowResolution {
        WindowResolution {
            physical_size: self.physical_size,
            logical_size: self.logical_size,
            device_pixel_ratio: self.device_pixel_ratio,
        }
    }
}

///
/// Output from the rendering to the window each frame.
///
#[derive(Clone, Debug)]
pub struct FrameOutput {
    ///
    /// If this is true:
    /// - On desktop, the window is closed and the renderloop is stopped.
    /// - On web, the render loop is stopped, the event handlers are removed and the `Window` dropped. Note that the canvas is not removed.
    ///
    pub exit: bool,

    ///
    /// Swaps the back and front buffer if this is true.
    /// Set this to true if something have been rendered this frame and you want to display it.
    /// Set it to false if nothing have been rendered this frame, for example if nothing has changed,
    /// and you want to reuse the image from an old frame.
    /// Currently ignored on web, since it does not use double buffering.
    ///
    pub swap_buffers: bool,

    ///
    /// Whether to stop the render loop until next event.
    ///
    pub wait_next_event: bool,

    pub events: Option<Vec<OutgoingEvent>>,
}

impl Default for FrameOutput {
    fn default() -> Self {
        Self {
            events: None,
            exit: false,
            swap_buffers: true,
            wait_next_event: false,
        }
    }
}

impl From<FrameInput<()>> for FrameOutput {
    fn from(frame_input: FrameInput<()>) -> Self {
        let mut output = Self::default();
        output.events = Some(frame_input.outgoing_events);
        output
    }
}
