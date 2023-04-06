use std::{cell::RefCell, ops::Deref};

use bevy_ecs::system::Resource;

use egui_glow::{glow, Painter};

use render_api::base::Viewport;
use render_glow::{
    core::Context,
    renderer::{Event, Modifiers},
    window::FrameInput,
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
}

impl Default for GUI {
    fn default() -> Self {
        let context = Context::get().deref().clone();
        GUI {
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
        let events: &mut [Event<T>] = frame_input.events.as_mut_slice();
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
                    Event::KeyPress {
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
                    Event::KeyRelease {
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
                    Event::MousePress {
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
                    Event::MouseRelease {
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
                    Event::MouseMotion {
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
                    Event::Text(text) => Some(egui::Event::Text(text.clone())),
                    Event::MouseLeave => Some(egui::Event::PointerGone),
                    Event::MouseWheel { delta, handled, .. } => {
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
        events: &mut [Event<T>],
    ) -> bool {
        *self.output.borrow_mut() = Some(egui_context.end_frame());

        for event in events.iter_mut() {
            if let Event::ModifiersChange { modifiers } = event {
                self.modifiers = *modifiers;
            }
            if egui_context.wants_pointer_input() {
                match event {
                    Event::MousePress {
                        ref mut handled, ..
                    } => {
                        *handled = true;
                    }
                    Event::MouseRelease {
                        ref mut handled, ..
                    } => {
                        *handled = true;
                    }
                    Event::MouseWheel {
                        ref mut handled, ..
                    } => {
                        *handled = true;
                    }
                    Event::MouseMotion {
                        ref mut handled, ..
                    } => {
                        *handled = true;
                    }
                    _ => {}
                }
            }

            if egui_context.wants_keyboard_input() {
                match event {
                    Event::KeyRelease {
                        ref mut handled, ..
                    } => {
                        *handled = true;
                    }
                    Event::KeyPress {
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
        let output = self
            .output
            .borrow_mut()
            .take()
            .expect("need to call GUI::update before GUI::render");
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
}
