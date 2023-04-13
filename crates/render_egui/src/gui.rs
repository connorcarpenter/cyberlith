use std::{cell::RefCell, ops::Deref};

use bevy_ecs::system::Resource;
use bevy_log::info;
use egui::{output::OutputEvent, FullOutput, PlatformOutput};

use egui_glow::{glow, Painter};

use render_api::components::Viewport;
use render_glow::{
    core::Context,
    renderer::{IncomingEvent, Modifiers},
    window::{FrameInput, OutgoingEvent},
};

use crate::{glow_to_egui_key, glow_to_egui_modifiers, glow_to_egui_mouse_button};

#[derive(Resource)]
pub struct EguiContext(pub egui::Context);

impl EguiContext {
    pub fn inner(&self) -> &egui::Context {
        &self.0
    }
}

impl Default for EguiContext {
    fn default() -> Self {
        Self(egui::Context::default())
    }
}

///
/// Integration of [egui](https://crates.io/crates/egui), an immediate mode GUI.
///
pub struct GUI {
    painter: RefCell<Painter>,
    output: RefCell<Option<egui::FullOutput>>,
    viewport: Viewport,
    modifiers: Modifiers,
    last_cursor_icon: egui::CursorIcon,
}

impl Default for GUI {
    fn default() -> Self {
        let context = Context::get().deref().clone();
        GUI {
            last_cursor_icon: egui::CursorIcon::Default,
            painter: RefCell::new(Painter::new(context, "", None).unwrap()),
            output: RefCell::new(None),
            viewport: Viewport::new_at_origin(1, 1),
            modifiers: Modifiers::default(),
        }
    }
}

impl GUI {
    ///
    /// Initialises a new frame of the GUI and handles events.
    /// Construct the GUI (Add panels, widgets etc.) using the [egui::Context] in the callback function.
    /// This function returns whether or not the GUI has changed, ie. if it consumes any events, and therefore needs to be rendered again.
    ///
    pub fn pre_update<T: 'static + Clone>(
        &mut self,
        egui_context: &egui::Context,
        frame_input: &mut FrameInput<T>,
    ) {
        let events: &mut [IncomingEvent<T>] = frame_input.incoming_events.as_mut_slice();
        let accumulated_time_in_ms: f64 = frame_input.accumulated_time;
        let viewport: Viewport = frame_input.viewport;
        let device_pixel_ratio: f64 = frame_input.device_pixel_ratio;

        self.viewport = viewport;
        let egui_input = egui::RawInput {
            screen_rect: Some(egui::Rect {
                min: egui::Pos2 {
                    x: viewport.x as f32 / device_pixel_ratio as f32,
                    y: viewport.y as f32 / device_pixel_ratio as f32,
                },
                max: egui::Pos2 {
                    x: viewport.x as f32 / device_pixel_ratio as f32
                        + viewport.width as f32 / device_pixel_ratio as f32,
                    y: viewport.y as f32 / device_pixel_ratio as f32
                        + viewport.height as f32 / device_pixel_ratio as f32,
                },
            }),
            pixels_per_point: Some(device_pixel_ratio as f32),
            time: Some(accumulated_time_in_ms * 0.001),
            modifiers: glow_to_egui_modifiers(&self.modifiers),
            events: events
                .iter()
                .filter_map(|event| match event {
                    IncomingEvent::KeyPress {
                        kind,
                        modifiers,
                        handled,
                    } => {
                        if !handled {
                            Some(egui::Event::Key {
                                key: glow_to_egui_key(kind),
                                pressed: true,
                                modifiers: glow_to_egui_modifiers(modifiers),
                                repeat: false,
                            })
                        } else {
                            None
                        }
                    }
                    IncomingEvent::KeyRelease {
                        kind,
                        modifiers,
                        handled,
                    } => {
                        if !handled {
                            Some(egui::Event::Key {
                                key: glow_to_egui_key(kind),
                                pressed: false,
                                modifiers: glow_to_egui_modifiers(modifiers),
                                repeat: false,
                            })
                        } else {
                            None
                        }
                    }
                    IncomingEvent::MousePress {
                        button,
                        position,
                        modifiers,
                        handled,
                    } => {
                        if !handled {
                            Some(egui::Event::PointerButton {
                                pos: egui::Pos2 {
                                    x: position.0 as f32,
                                    y: position.1 as f32,
                                },
                                button: glow_to_egui_mouse_button(button),
                                pressed: true,
                                modifiers: glow_to_egui_modifiers(modifiers),
                            })
                        } else {
                            None
                        }
                    }
                    IncomingEvent::MouseRelease {
                        button,
                        position,
                        modifiers,
                        handled,
                    } => {
                        if !handled {
                            Some(egui::Event::PointerButton {
                                pos: egui::Pos2 {
                                    x: position.0 as f32,
                                    y: position.1 as f32,
                                },
                                button: glow_to_egui_mouse_button(button),
                                pressed: false,
                                modifiers: glow_to_egui_modifiers(modifiers),
                            })
                        } else {
                            None
                        }
                    }
                    IncomingEvent::MouseMotion {
                        position, handled, ..
                    } => {
                        if !handled {
                            Some(egui::Event::PointerMoved(egui::Pos2 {
                                x: position.0 as f32,
                                y: position.1 as f32,
                            }))
                        } else {
                            None
                        }
                    }
                    IncomingEvent::Text(text) => Some(egui::Event::Text(text.clone())),
                    IncomingEvent::MouseLeave => Some(egui::Event::PointerGone),
                    IncomingEvent::MouseWheel { delta, handled, .. } => {
                        if !handled {
                            Some(egui::Event::Scroll(egui::Vec2::new(
                                delta.0 as f32,
                                delta.1 as f32,
                            )))
                        } else {
                            None
                        }
                    }
                    _ => None,
                })
                .collect::<Vec<_>>(),
            ..Default::default()
        };

        egui_context.begin_frame(egui_input);
    }

    ///
    /// Initialises a new frame of the GUI and handles events.
    /// Construct the GUI (Add panels, widgets etc.) using the [egui::Context] in the callback function.
    /// This function returns whether or not the GUI has changed, ie. if it consumes any events, and therefore needs to be rendered again.
    ///
    pub fn post_update<T: 'static + Clone>(
        &mut self,
        egui_context: &egui::Context,
        frame_input: &mut FrameInput<T>,
    ) -> bool {
        let mut end_frame = egui_context.end_frame();

        // Output Events
        self.handle_egui_output(
            &mut frame_input.outgoing_events,
            &mut end_frame.platform_output,
        );

        *self.output.borrow_mut() = Some(end_frame);

        // Input Events
        for event in frame_input.incoming_events.iter_mut() {
            if let IncomingEvent::ModifiersChange { modifiers } = event {
                self.modifiers = *modifiers;
            }
            if egui_context.wants_pointer_input() {
                match event {
                    IncomingEvent::MousePress {
                        ref mut handled, ..
                    } => {
                        *handled = true;
                    }
                    IncomingEvent::MouseRelease {
                        ref mut handled, ..
                    } => {
                        *handled = true;
                    }
                    IncomingEvent::MouseWheel {
                        ref mut handled, ..
                    } => {
                        *handled = true;
                    }
                    IncomingEvent::MouseMotion {
                        ref mut handled, ..
                    } => {
                        *handled = true;
                    }
                    _ => {}
                }
            }

            if egui_context.wants_keyboard_input() {
                match event {
                    IncomingEvent::KeyRelease {
                        ref mut handled, ..
                    } => {
                        *handled = true;
                    }
                    IncomingEvent::KeyPress {
                        ref mut handled, ..
                    } => {
                        *handled = true;
                    }
                    _ => {}
                }
            }
        }
        egui_context.wants_pointer_input() || egui_context.wants_keyboard_input()
    }

    ///
    /// Render the GUI defined in the [update](Self::update) function.
    /// Must be called in the callback given as input to a [RenderTarget], [ColorTarget] or [DepthTarget] write method.
    ///
    pub fn render(&self, egui_context: &egui::Context) {
        let Some(output) = self
            .output
            .borrow_mut()
            .take() else {
            info!("No output to render");
            return;
        };
        let clipped_meshes = egui_context.tessellate(output.shapes);
        let scale = egui_context.pixels_per_point();
        self.painter.borrow_mut().paint_and_update_textures(
            [self.viewport.width, self.viewport.height],
            scale,
            &clipped_meshes,
            &output.textures_delta,
        );
        #[cfg(not(target_arch = "wasm32"))]
        #[allow(unsafe_code)]
        unsafe {
            use glow::HasContext as _;
            self.painter.borrow().gl().disable(glow::FRAMEBUFFER_SRGB);
        }
    }

    pub fn add_texture(&mut self, native: glow::Texture) -> egui::TextureId {
        self.painter.borrow_mut().register_native_texture(native)
    }

    pub fn remove_texture(&mut self, native_id: egui::TextureId) {
        self.painter.borrow_mut().free_texture(native_id);
    }

    fn handle_egui_output(
        &mut self,
        outgoing_events: &mut Vec<OutgoingEvent>,
        egui_output: &mut PlatformOutput,
    ) {
        self.set_cursor_icon(outgoing_events, egui_output.cursor_icon);
    }

    fn set_cursor_icon(
        &mut self,
        outgoing_events: &mut Vec<OutgoingEvent>,
        cursor_icon: egui::CursorIcon,
    ) {
        if self.last_cursor_icon != cursor_icon {
            self.last_cursor_icon = cursor_icon;
            if let Some(winit_cursor_icon) = Self::translate_cursor(cursor_icon) {
                outgoing_events.push(OutgoingEvent::CursorChanged(winit_cursor_icon));
            }
        }

        // if self.current_cursor_icon == Some(cursor_icon) {
        //     // Prevent flickering near frame boundary when Windows OS tries to control cursor icon for window resizing.
        //     // On other platforms: just early-out to save CPU.
        //     return;
        // }
        //
        // let is_pointer_in_window = self.pointer_pos_in_points.is_some();
        // if is_pointer_in_window {
        //     self.current_cursor_icon = Some(cursor_icon);
        // } else {
        //     // Remember to set the cursor again once the cursor returns to the screen:
        //     self.current_cursor_icon = None;
        // }
    }

    fn translate_cursor(cursor_icon: egui::CursorIcon) -> Option<winit::window::CursorIcon> {
        match cursor_icon {
            egui::CursorIcon::None => None,

            egui::CursorIcon::Alias => Some(winit::window::CursorIcon::Alias),
            egui::CursorIcon::AllScroll => Some(winit::window::CursorIcon::AllScroll),
            egui::CursorIcon::Cell => Some(winit::window::CursorIcon::Cell),
            egui::CursorIcon::ContextMenu => Some(winit::window::CursorIcon::ContextMenu),
            egui::CursorIcon::Copy => Some(winit::window::CursorIcon::Copy),
            egui::CursorIcon::Crosshair => Some(winit::window::CursorIcon::Crosshair),
            egui::CursorIcon::Default => Some(winit::window::CursorIcon::Default),
            egui::CursorIcon::Grab => Some(winit::window::CursorIcon::Grab),
            egui::CursorIcon::Grabbing => Some(winit::window::CursorIcon::Grabbing),
            egui::CursorIcon::Help => Some(winit::window::CursorIcon::Help),
            egui::CursorIcon::Move => Some(winit::window::CursorIcon::Move),
            egui::CursorIcon::NoDrop => Some(winit::window::CursorIcon::NoDrop),
            egui::CursorIcon::NotAllowed => Some(winit::window::CursorIcon::NotAllowed),
            egui::CursorIcon::PointingHand => Some(winit::window::CursorIcon::Hand),
            egui::CursorIcon::Progress => Some(winit::window::CursorIcon::Progress),

            egui::CursorIcon::ResizeHorizontal => Some(winit::window::CursorIcon::EwResize),
            egui::CursorIcon::ResizeNeSw => Some(winit::window::CursorIcon::NeswResize),
            egui::CursorIcon::ResizeNwSe => Some(winit::window::CursorIcon::NwseResize),
            egui::CursorIcon::ResizeVertical => Some(winit::window::CursorIcon::NsResize),

            egui::CursorIcon::ResizeEast => Some(winit::window::CursorIcon::EResize),
            egui::CursorIcon::ResizeSouthEast => Some(winit::window::CursorIcon::SeResize),
            egui::CursorIcon::ResizeSouth => Some(winit::window::CursorIcon::SResize),
            egui::CursorIcon::ResizeSouthWest => Some(winit::window::CursorIcon::SwResize),
            egui::CursorIcon::ResizeWest => Some(winit::window::CursorIcon::WResize),
            egui::CursorIcon::ResizeNorthWest => Some(winit::window::CursorIcon::NwResize),
            egui::CursorIcon::ResizeNorth => Some(winit::window::CursorIcon::NResize),
            egui::CursorIcon::ResizeNorthEast => Some(winit::window::CursorIcon::NeResize),
            egui::CursorIcon::ResizeColumn => Some(winit::window::CursorIcon::ColResize),
            egui::CursorIcon::ResizeRow => Some(winit::window::CursorIcon::RowResize),

            egui::CursorIcon::Text => Some(winit::window::CursorIcon::Text),
            egui::CursorIcon::VerticalText => Some(winit::window::CursorIcon::VerticalText),
            egui::CursorIcon::Wait => Some(winit::window::CursorIcon::Wait),
            egui::CursorIcon::ZoomIn => Some(winit::window::CursorIcon::ZoomIn),
            egui::CursorIcon::ZoomOut => Some(winit::window::CursorIcon::ZoomOut),
        }
    }
}
