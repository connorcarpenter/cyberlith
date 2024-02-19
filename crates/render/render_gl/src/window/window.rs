use std::sync::{Arc, RwLock};

use bevy_log::info;
use winit::{
    event::{Event as WinitEvent, TouchPhase, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    *,
};
use winit::event_loop::EventLoopWindowTarget;
use winit::keyboard::{KeyCode, PhysicalKey};

use input::{IncomingEvent, Key, Modifiers, MouseButton};
use render_api::{
    components::Viewport,
    resources::{SurfaceSettings, WindowSettings},
};

#[cfg(target_arch = "wasm32")]
use winit::platform::web::EventLoopExtWebSys;

use crate::{window::{FrameInput, FrameOutput, OutgoingEvent, WindowError, WindowedContext}, runner::{StopSignal}};

///
/// Window and event handling.
/// Use [Window::new] to create a new window or [Window::from_winit_window] which provides full control over the creation of the window.
///
pub struct Window<T: 'static + Clone> {
    window: winit::window::Window,
    event_loop: EventLoop<T>,
    #[cfg(target_arch = "wasm32")]
    closure: wasm_bindgen::closure::Closure<dyn FnMut(web_sys::Event)>,
    gl: WindowedContext,
    #[allow(dead_code)]
    maximized: bool,
}

impl Window<()> {
    ///
    /// Constructs a new Window with the given [settings].
    ///
    ///
    /// [settings]: WindowSettings
    pub fn new(window_settings: WindowSettings) -> Result<Window<()>, WindowError> {
        let event_loop = EventLoop::new().map_err(|event_loop_err| {
            WindowError::WinitEventLoopError(event_loop_err)
        })?;
        Self::from_event_loop(window_settings, event_loop)
    }
}

impl<T: 'static + Clone> Window<T> {
    /// Exactly the same as [`Window::new()`] except with the ability to supply
    /// an existing [`EventLoop`]. Use the event loop's [proxy] to push custom
    /// events into the render loop (from any thread). Not available for web.
    ///
    /// [proxy]: winit::event_loop::EventLoopProxy
    #[cfg(not(target_arch = "wasm32"))]
    pub fn from_event_loop(
        window_settings: WindowSettings,
        event_loop: EventLoop<T>,
    ) -> Result<Self, WindowError> {
        let winit_window = if let Some((width, height)) = window_settings.max_size {
            WindowBuilder::new()
                .with_title(&window_settings.title)
                .with_min_inner_size(dpi::LogicalSize::new(
                    window_settings.min_size.0,
                    window_settings.min_size.1,
                ))
                .with_inner_size(dpi::LogicalSize::new(width as f64, height as f64))
                .with_max_inner_size(dpi::LogicalSize::new(width as f64, height as f64))
        } else {
            WindowBuilder::new()
                .with_min_inner_size(dpi::LogicalSize::new(
                    window_settings.min_size.0,
                    window_settings.min_size.1,
                ))
                .with_title(&window_settings.title)
                .with_maximized(true)
        }
        .build(&event_loop)?;
        Self::from_winit_window(
            winit_window,
            event_loop,
            window_settings.surface_settings,
            window_settings.max_size.is_none(),
        )
    }

    /// Exactly the same as [`Window::new()`] except with the ability to supply
    /// an existing [`EventLoop`]. Use the event loop's [proxy] to push custom
    /// events into the render loop (from any thread). Not available for web.
    ///
    /// [proxy]: winit::event_loop::EventLoopProxy
    #[cfg(target_arch = "wasm32")]
    pub fn from_event_loop(
        window_settings: WindowSettings,
        event_loop: EventLoop<T>,
    ) -> Result<Self, WindowError> {
        use wasm_bindgen::JsCast;
        use winit::{dpi::LogicalSize, platform::web::WindowBuilderExtWebSys};

        let websys_window = web_sys::window().ok_or(WindowError::WindowCreation)?;
        let document = websys_window
            .document()
            .ok_or(WindowError::DocumentMissing)?;

        let canvas = {
            document
                .get_elements_by_tag_name("canvas")
                .item(0)
                .expect(
                    "settings doesn't contain canvas and DOM doesn't have a canvas element either",
                )
                .dyn_into::<web_sys::HtmlCanvasElement>()
                .map_err(|e| WindowError::CanvasConvertFailed(format!("{:?}", e)))?
        };

        let inner_size = window_settings
            .max_size
            .map(|(width, height)| LogicalSize::new(width as f64, height as f64))
            .unwrap_or_else(|| {
                let browser_window = canvas
                    .owner_document()
                    .and_then(|doc| doc.default_view())
                    .or_else(web_sys::window)
                    .unwrap();
                LogicalSize::new(
                    browser_window.inner_width().unwrap().as_f64().unwrap(),
                    browser_window.inner_height().unwrap().as_f64().unwrap(),
                )
            });

        let window_builder = WindowBuilder::new()
            .with_title(window_settings.title)
            .with_canvas(Some(canvas))
            .with_inner_size(inner_size)
            .with_prevent_default(true);

        let winit_window = window_builder.build(&event_loop)?;

        Self::from_winit_window(
            winit_window,
            event_loop,
            window_settings.surface_settings,
            window_settings.max_size.is_none(),
        )
    }

    ///
    /// Creates a new window from a [winit](https://crates.io/crates/winit) window and event loop with the given surface settings, giving the user full
    /// control over the creation of the window.
    /// This method takes ownership of the winit window and event loop, if this is not desired, use a [WindowedContext] or [HeadlessContext](crate::HeadlessContext) instead.
    ///
    pub fn from_winit_window(
        winit_window: window::Window,
        event_loop: EventLoop<T>,
        mut surface_settings: SurfaceSettings,
        maximized: bool,
    ) -> Result<Self, WindowError> {
        let mut gl = WindowedContext::from_winit_window(&winit_window, surface_settings);
        if gl.is_err() {
            surface_settings.multisamples = 0;
            gl = WindowedContext::from_winit_window(&winit_window, surface_settings);
        }

        #[cfg(target_arch = "wasm32")]
        let closure = {
            use wasm_bindgen::JsCast;
            use winit::platform::web::WindowExtWebSys;
            let closure =
                wasm_bindgen::closure::Closure::wrap(Box::new(move |event: web_sys::Event| {
                    event.prevent_default();
                }) as Box<dyn FnMut(_)>);
            winit_window
                .canvas()
                .unwrap()
                .add_event_listener_with_callback("contextmenu", closure.as_ref().unchecked_ref())
                .expect("failed to listen to canvas context menu");
            closure
        };

        Ok(Self {
            window: winit_window,
            event_loop,
            gl: gl?,
            #[cfg(target_arch = "wasm32")]
            closure,
            maximized,
        })
    }

    pub fn wait_for_stop(&self) {

    }

    ///
    /// Start the main render loop which calls the `callback` closure each frame.
    ///
    pub fn render_loop<F: 'static + FnMut(FrameInput<T>) -> FrameOutput>(
        #[allow(unused_mut)]
        mut self,
        stop_signal: Arc<RwLock<StopSignal>>,
        mut callback: F
    ) {
        #[cfg(not(target_arch = "wasm32"))]
        let mut last_time = std::time::Instant::now();
        #[cfg(target_arch = "wasm32")]
        let mut last_time = instant::Instant::now();

        let mut accumulated_time = 0.0;
        let mut events = Vec::new();
        let mut cursor_pos = None;
        let mut finger_id = None;
        let mut secondary_cursor_pos = None;
        let mut secondary_finger_id = None;
        let mut modifiers = Modifiers::default();
        let mut first_frame = true;
        let mut mouse_pressed = None;
        let stop_signal = stop_signal.clone();
        let loop_func = move |event: WinitEvent::<T>, elwt: &EventLoopWindowTarget<T>| {
            let stop_signal = stop_signal.clone();
            match event {
                WinitEvent::UserEvent(t) => {
                    events.push(IncomingEvent::UserEvent(t));
                }
                WinitEvent::LoopExiting => {
                    #[cfg(target_arch = "wasm32")]
                    {
                        use wasm_bindgen::JsCast;
                        use winit::platform::web::WindowExtWebSys;
                        self.window
                            .canvas()
                            .unwrap()
                            .remove_event_listener_with_callback(
                                "contextmenu",
                                self.closure.as_ref().unchecked_ref(),
                            )
                            .unwrap();
                    }

                    if let Ok(mut stop_signal) = stop_signal.write() {
                        stop_signal.stopped = true;
                    } else {
                        panic!("failed to write stop signal");
                    }
                }
                WinitEvent::WindowEvent { ref event, .. } => match event {
                    WindowEvent::RedrawRequested => {
                        #[cfg(not(target_arch = "wasm32"))]
                        let now = std::time::Instant::now();
                        #[cfg(target_arch = "wasm32")]
                        let now = instant::Instant::now();

                        let duration = now.duration_since(last_time);
                        last_time = now;
                        let elapsed_time =
                            duration.as_secs() as f64 * 1000.0 + duration.subsec_nanos() as f64 * 1e-6;
                        accumulated_time += elapsed_time;

                        #[cfg(target_arch = "wasm32")]
                        if self.maximized {
                            use winit::platform::web::WindowExtWebSys;

                            let html_canvas = self.window.canvas().unwrap();
                            let browser_window = html_canvas
                                .owner_document()
                                .and_then(|doc| doc.default_view())
                                .or_else(web_sys::window)
                                .unwrap();

                            self.window.request_inner_size(dpi::LogicalSize {
                                width: browser_window.inner_width().unwrap().as_f64().unwrap(),
                                height: browser_window.inner_height().unwrap().as_f64().unwrap(),
                            });
                        }

                        let (physical_width, physical_height): (u32, u32) =
                            self.window.inner_size().into();
                        let device_pixel_ratio = self.window.scale_factor();
                        let (logical_width, logical_height): (u32, u32) = self
                            .window
                            .inner_size()
                            .to_logical::<f64>(device_pixel_ratio)
                            .into();
                        let frame_input = FrameInput {
                            incoming_events: events.drain(..).collect(),
                            outgoing_events: Vec::new(),
                            elapsed_time,
                            accumulated_time,
                            logical_size: Viewport::new_at_origin(logical_width, logical_height),
                            physical_size: Viewport::new_at_origin(physical_width, physical_height),
                            device_pixel_ratio,
                            first_frame,
                        };
                        first_frame = false;
                        let frame_output = callback(frame_input);

                        for event in frame_output.events.unwrap() {
                            match event {
                                OutgoingEvent::CursorChanged(cursor_icon) => {
                                    self.window.set_cursor_icon(cursor_icon);
                                }
                                OutgoingEvent::Exit => {
                                    elwt.exit();
                                }
                            }
                        }

                        if frame_output.exit {
                            elwt.exit();
                        } else {
                            if frame_output.swap_buffers {
                                #[cfg(not(target_arch = "wasm32"))]
                                self.gl.swap_buffers().unwrap();
                            }
                            if frame_output.wait_next_event {
                                elwt.set_control_flow(ControlFlow::Wait);
                            } else {
                                elwt.set_control_flow(ControlFlow::Poll);
                                self.window.request_redraw();
                            }
                        }
                    }
                    WindowEvent::Resized(physical_size) => {
                        self.gl.resize(*physical_size);
                    }
                    WindowEvent::CloseRequested => {
                        elwt.exit();
                        info!("close requested");
                    },
                    WindowEvent::KeyboardInput { event, .. } => {
                        if let PhysicalKey::Code(keycode) = event.physical_key {

                            let state = event.state == event::ElementState::Pressed;
                            if let Some(kind) = translate_virtual_key_code(keycode) {
                                events.push(if state {
                                    IncomingEvent::KeyPress {
                                        kind,
                                        modifiers,
                                        handled: false,
                                    }
                                } else {
                                    IncomingEvent::KeyRelease {
                                        kind,
                                        modifiers,
                                        handled: false,
                                    }
                                });
                            } else if keycode == KeyCode::ControlLeft
                                || keycode == KeyCode::ControlRight
                            {
                                modifiers.ctrl = state;
                                modifiers.command = state;
                                events.push(IncomingEvent::ModifiersChange { modifiers });
                            } else if keycode == KeyCode::AltLeft
                                || keycode == KeyCode::AltRight
                            {
                                modifiers.alt = state;
                                events.push(IncomingEvent::ModifiersChange { modifiers });
                            } else if keycode == KeyCode::ShiftLeft
                                || keycode == KeyCode::ShiftRight
                            {
                                modifiers.shift = state;
                                events.push(IncomingEvent::ModifiersChange { modifiers });
                            }
                        }
                    }
                    WindowEvent::MouseWheel { delta, .. } => {
                        if let Some(position) = cursor_pos {
                            match delta {
                                winit::event::MouseScrollDelta::LineDelta(x, y) => {
                                    let line_height = 24.0; // TODO
                                    events.push(IncomingEvent::MouseWheel {
                                        delta: (
                                            (*x * line_height) as f64,
                                            (*y * line_height) as f64,
                                        ),
                                        position,
                                        modifiers,
                                        handled: false,
                                    });
                                }
                                winit::event::MouseScrollDelta::PixelDelta(delta) => {
                                    let d = delta.to_logical(self.window.scale_factor());
                                    events.push(IncomingEvent::MouseWheel {
                                        delta: (d.x, d.y),
                                        position,
                                        modifiers,
                                        handled: false,
                                    });
                                }
                            }
                        }
                    }
                    WindowEvent::MouseInput { state, button, .. } => {
                        if let Some(position) = cursor_pos {
                            let button = match button {
                                event::MouseButton::Left => Some(MouseButton::Left),
                                event::MouseButton::Middle => Some(MouseButton::Middle),
                                event::MouseButton::Right => Some(MouseButton::Right),
                                _ => None,
                            };
                            if let Some(b) = button {
                                events.push(if *state == event::ElementState::Pressed {
                                    mouse_pressed = Some(b);
                                    IncomingEvent::MousePress {
                                        button: b,
                                        position,
                                        modifiers,
                                        handled: false,
                                    }
                                } else {
                                    mouse_pressed = None;
                                    IncomingEvent::MouseRelease {
                                        button: b,
                                        position,
                                        modifiers,
                                        handled: false,
                                    }
                                });
                            }
                        }
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        let p = position.to_logical(self.window.scale_factor());
                        let delta = if let Some(last_pos) = cursor_pos {
                            (p.x - last_pos.0, p.y - last_pos.1)
                        } else {
                            (0.0, 0.0)
                        };
                        events.push(IncomingEvent::MouseMotion {
                            button: mouse_pressed,
                            delta,
                            position: (p.x, p.y),
                            modifiers,
                            handled: false,
                        });
                        cursor_pos = Some((p.x, p.y));
                    }
                    // WindowEvent::KeyboardInput { event, .. } => {
                    //     if let keyboard::Key::Character(character) = event.logical_key.clone() {
                    //         let ch = character.as_str();
                    //         let ch = ch.chars().next().unwrap();
                    //         if is_printable_char(ch) && !modifiers.ctrl && !modifiers.command {
                    //             events.push(IncomingEvent::Text(ch.to_string()));
                    //         }
                    //     }
                    // }
                    WindowEvent::CursorEntered { .. } => {
                        events.push(IncomingEvent::MouseEnter);
                    }
                    WindowEvent::CursorLeft { .. } => {
                        mouse_pressed = None;
                        events.push(IncomingEvent::MouseLeave);
                    }
                    WindowEvent::Touch(touch) => {
                        let position = touch
                            .location
                            .to_logical::<f64>(self.window.scale_factor())
                            .into();
                        match touch.phase {
                            TouchPhase::Started => {
                                if finger_id.is_none() {
                                    events.push(IncomingEvent::MousePress {
                                        button: MouseButton::Left,
                                        position,
                                        modifiers,
                                        handled: false,
                                    });
                                    cursor_pos = Some(position);
                                    finger_id = Some(touch.id);
                                } else if secondary_finger_id.is_none() {
                                    secondary_cursor_pos = Some(position);
                                    secondary_finger_id = Some(touch.id);
                                }
                            }
                            TouchPhase::Ended | TouchPhase::Cancelled => {
                                if finger_id.map(|id| id == touch.id).unwrap_or(false) {
                                    events.push(IncomingEvent::MouseRelease {
                                        button: MouseButton::Left,
                                        position,
                                        modifiers,
                                        handled: false,
                                    });
                                    cursor_pos = None;
                                    finger_id = None;
                                } else if secondary_finger_id
                                    .map(|id| id == touch.id)
                                    .unwrap_or(false)
                                {
                                    secondary_cursor_pos = None;
                                    secondary_finger_id = None;
                                }
                            }
                            TouchPhase::Moved => {
                                if finger_id.map(|id| id == touch.id).unwrap_or(false) {
                                    let last_pos = cursor_pos.unwrap();
                                    if let Some(p) = secondary_cursor_pos {
                                        events.push(IncomingEvent::MouseWheel {
                                            position,
                                            modifiers,
                                            handled: false,
                                            delta: (
                                                (position.0 - p.0).abs() - (last_pos.0 - p.0).abs(),
                                                (position.1 - p.1).abs() - (last_pos.1 - p.1).abs(),
                                            ),
                                        });
                                    } else {
                                        events.push(IncomingEvent::MouseMotion {
                                            button: Some(MouseButton::Left),
                                            position,
                                            modifiers,
                                            handled: false,
                                            delta: (
                                                position.0 - last_pos.0,
                                                position.1 - last_pos.1,
                                            ),
                                        });
                                    }
                                    cursor_pos = Some(position);
                                } else if secondary_finger_id
                                    .map(|id| id == touch.id)
                                    .unwrap_or(false)
                                {
                                    let last_pos = secondary_cursor_pos.unwrap();
                                    if let Some(p) = cursor_pos {
                                        events.push(IncomingEvent::MouseWheel {
                                            position: p,
                                            modifiers,
                                            handled: false,
                                            delta: (
                                                (position.0 - p.0).abs() - (last_pos.0 - p.0).abs(),
                                                (position.1 - p.1).abs() - (last_pos.1 - p.1).abs(),
                                            ),
                                        });
                                    }
                                    secondary_cursor_pos = Some(position);
                                }
                            }
                        }
                    }
                    _ => (),
                },
                _ => (),
            }
        };

        #[cfg(not(target_arch = "wasm32"))]
        if let Err(e) = self.event_loop.run(loop_func) {
            panic!("Error in event loop: {}", e);
        }

        #[cfg(target_arch = "wasm32")]
        EventLoop::<T>::spawn(self.event_loop, loop_func);
    }

    ///
    /// Return the current logical size of the window.
    ///
    pub fn size(&self) -> (u32, u32) {
        self.window
            .inner_size()
            .to_logical::<f64>(self.window.scale_factor())
            .into()
    }

    ///
    /// Returns the current viewport of the window in physical pixels (the size of the screen returned from [FrameInput::screen]).
    ///
    pub fn viewport(&self) -> Viewport {
        let (w, h): (u32, u32) = self.window.inner_size().into();
        Viewport::new_at_origin(w, h)
    }

    ///
    /// Returns an event loop proxy that can be used to send a `T` into the
    /// render loop using the proxy's [`send_event`] method. The event can be
    /// handled in the render loop by matching [`Event::UserEvent`].
    ///
    /// [`Event::UserEvent`]: crate::control::Event::UserEvent
    /// [`send_event`]: winit::event_loop::EventLoopProxy::send_event
    pub fn event_loop_proxy(&self) -> winit::event_loop::EventLoopProxy<T> {
        self.event_loop.create_proxy()
    }
}

fn is_printable_char(chr: char) -> bool {
    let is_in_private_use_area = ('\u{e000}'..='\u{f8ff}').contains(&chr)
        || ('\u{f0000}'..='\u{ffffd}').contains(&chr)
        || ('\u{100000}'..='\u{10fffd}').contains(&chr);

    !is_in_private_use_area && !chr.is_ascii_control()
}

fn translate_virtual_key_code(key: KeyCode) -> Option<Key> {

    Some(match key {
        KeyCode::ArrowDown => Key::ArrowDown,
        KeyCode::ArrowLeft => Key::ArrowLeft,
        KeyCode::ArrowRight => Key::ArrowRight,
        KeyCode::ArrowUp => Key::ArrowUp,

        KeyCode::Escape => Key::Escape,
        KeyCode::Tab => Key::Tab,
        KeyCode::Backspace => Key::Backspace,
        KeyCode::Enter => Key::Enter,
        KeyCode::Space => Key::Space,

        KeyCode::Insert => Key::Insert,
        KeyCode::Delete => Key::Delete,
        KeyCode::Home => Key::Home,
        KeyCode::End => Key::End,
        KeyCode::PageUp => Key::PageUp,
        KeyCode::PageDown => Key::PageDown,

        KeyCode::Digit0 | KeyCode::Numpad0 => Key::Num0,
        KeyCode::Digit1 | KeyCode::Numpad1 => Key::Num1,
        KeyCode::Digit2 | KeyCode::Numpad2 => Key::Num2,
        KeyCode::Digit3 | KeyCode::Numpad3 => Key::Num3,
        KeyCode::Digit4 | KeyCode::Numpad4 => Key::Num4,
        KeyCode::Digit5 | KeyCode::Numpad5 => Key::Num5,
        KeyCode::Digit6 | KeyCode::Numpad6 => Key::Num6,
        KeyCode::Digit7 | KeyCode::Numpad7 => Key::Num7,
        KeyCode::Digit8 | KeyCode::Numpad8 => Key::Num8,
        KeyCode::Digit9 | KeyCode::Numpad9 => Key::Num9,

        KeyCode::KeyA => Key::A,
        KeyCode::KeyB => Key::B,
        KeyCode::KeyC => Key::C,
        KeyCode::KeyD => Key::D,
        KeyCode::KeyE => Key::E,
        KeyCode::KeyF => Key::F,
        KeyCode::KeyG => Key::G,
        KeyCode::KeyH => Key::H,
        KeyCode::KeyI => Key::I,
        KeyCode::KeyJ => Key::J,
        KeyCode::KeyK => Key::K,
        KeyCode::KeyL => Key::L,
        KeyCode::KeyM => Key::M,
        KeyCode::KeyN => Key::N,
        KeyCode::KeyO => Key::O,
        KeyCode::KeyP => Key::P,
        KeyCode::KeyQ => Key::Q,
        KeyCode::KeyR => Key::R,
        KeyCode::KeyS => Key::S,
        KeyCode::KeyT => Key::T,
        KeyCode::KeyU => Key::U,
        KeyCode::KeyV => Key::V,
        KeyCode::KeyW => Key::W,
        KeyCode::KeyX => Key::X,
        KeyCode::KeyY => Key::Y,
        KeyCode::KeyZ => Key::Z,

        _ => {
            return None;
        }
    })
}
