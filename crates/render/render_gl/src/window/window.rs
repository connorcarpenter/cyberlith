use bevy_app::App;
use cfg_if::cfg_if;
use winit::{
    dpi,
    event::{
        ElementState, Event as WinitEvent, MouseButton as WinitMouseButton, TouchPhase,
        VirtualKeyCode, WindowEvent,
    },
    event_loop::{ControlFlow, EventLoop},
    window,
    window::WindowBuilder,
};

use input::{IncomingEvent, Key, MouseButton};
use render_api::{
    components::Viewport,
    resources::{SurfaceSettings, WindowSettings},
};

#[cfg(not(target_arch = "wasm32"))]
use winit::platform::run_return::EventLoopExtRunReturn;

#[cfg(not(target_arch = "wasm32"))]
use crate::core::Context;

use kernel::ExitActionContainer;
#[cfg(target_arch = "wasm32")]
use winit::platform::web::EventLoopExtWebSys;

use crate::window::{
    render_loop_context::RenderLoopContext, FrameInput, FrameOutput, OutgoingEvent, WindowError,
    WindowedContext,
};

static mut WINDOW_CONTAINER: Option<Window> = None;

///
/// Window and event handling.
/// Use [Window::new] to create a new window or [Window::from_winit_window] which provides full control over the creation of the window.
///
pub struct Window {
    window: winit::window::Window,
    event_loop_opt: Option<EventLoop<()>>,
    #[cfg(target_arch = "wasm32")]
    closure: wasm_bindgen::closure::Closure<dyn FnMut(web_sys::Event)>,
    gl: WindowedContext,
    #[allow(dead_code)]
    maximized: bool,
}

impl Window {
    pub(crate) fn run_render_loop(settings: WindowSettings, app: App) {
        // Create a Window
        let window = Self::take_or_new(settings);

        // Run the main render loop
        let window_opt = Window::render_loop(window, app);

        // store for later
        if let Some(window) = window_opt {
            Window::set(window);
        }
    }

    fn inner_render_loop(app: &mut App, new_frame_input: FrameInput) -> FrameOutput {
        // Insert FrameInput
        app.world_mut().insert_non_send_resource(new_frame_input);

        // update app
        app.update();

        // Remove FrameInput
        let old_frame_input = app
            .world_mut()
            .remove_non_send_resource::<FrameInput>()
            .unwrap();

        // Returns default frame output to end the frame
        FrameOutput::from(old_frame_input)
    }
}

impl Window {
    pub fn take_or_new(window_settings: WindowSettings) -> Window {
        unsafe {
            #[allow(static_mut_refs)]
            if WINDOW_CONTAINER.is_none() {
                // info!("creating new window");
                return Self::new(window_settings).unwrap();
            }
            // info!("using existing window");
            #[allow(static_mut_refs)]
            return WINDOW_CONTAINER.take().unwrap();
        }
    }

    pub fn set(window: Window) {
        unsafe {
            #[allow(static_mut_refs)]
            if WINDOW_CONTAINER.is_some() {
                panic!("Window container already set");
            }
            WINDOW_CONTAINER = Some(window);
        }
    }

    ///
    /// Constructs a new Window with the given [settings].
    ///
    ///
    /// [settings]: WindowSettings
    pub fn new(window_settings: WindowSettings) -> Result<Window, WindowError> {
        Self::from_event_loop(window_settings, EventLoop::new())
    }

    /// Exactly the same as [`Window::new()`] except with the ability to supply
    /// an existing [`EventLoop`]. Use the event loop's [proxy] to push custom
    /// events into the render loop (from any thread). Not available for web.
    ///
    /// [proxy]: winit::event_loop::EventLoopProxy
    #[cfg(not(target_arch = "wasm32"))]
    fn from_event_loop(
        window_settings: WindowSettings,
        event_loop: EventLoop<()>,
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
    fn from_event_loop(
        window_settings: WindowSettings,
        event_loop: EventLoop<()>,
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
        event_loop: EventLoop<()>,
        surface_settings: SurfaceSettings,
        maximized: bool,
    ) -> Result<Self, WindowError> {
        let mut gl = WindowedContext::from_winit_window(&winit_window, surface_settings);
        if gl.is_err() {
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
                .add_event_listener_with_callback("contextmenu", closure.as_ref().unchecked_ref())
                .expect("failed to listen to canvas context menu");
            closure
        };

        Ok(Self {
            window: winit_window,
            event_loop_opt: Some(event_loop),
            gl: gl?,
            #[cfg(target_arch = "wasm32")]
            closure,
            maximized,
        })
    }

    ///
    /// Start the main render loop which calls the `callback` closure each frame.
    ///
    pub fn render_loop(mut self, mut app: App) -> Option<Self> {
        let mut rlc = RenderLoopContext::new();

        cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                let event_loop = self.event_loop_opt.take().unwrap();
                let gl = self.gl;
                let window = self.window;
                let closure = self.closure;
                let maximized = self.maximized;
            } else {
                let mut event_loop = self.event_loop_opt.take().unwrap();
                let gl = &self.gl;
                let window = &mut self.window;
            }
        }

        let loop_func = move |event: WinitEvent<'_, ()>, _: &_, control_flow: &mut _| {
            match event {
                WinitEvent::LoopDestroyed => {
                    #[cfg(target_arch = "wasm32")]
                    {
                        use wasm_bindgen::JsCast;
                        use winit::platform::web::WindowExtWebSys;
                        window
                            .canvas()
                            .remove_event_listener_with_callback(
                                "contextmenu",
                                closure.as_ref().unchecked_ref(),
                            )
                            .unwrap();
                    }
                }
                WinitEvent::MainEventsCleared => {
                    window.request_redraw();
                }
                WinitEvent::RedrawRequested(_) => {
                    let now = instant::Instant::now();
                    let duration = rlc.last_time.elapsed(&now);
                    rlc.last_time = now;
                    let elapsed_time =
                        duration.as_secs() as f64 * 1000.0 + duration.subsec_nanos() as f64 * 1e-6;
                    rlc.accumulated_time += elapsed_time;

                    #[cfg(target_arch = "wasm32")]
                    if maximized {
                        use winit::platform::web::WindowExtWebSys;

                        let html_canvas = window.canvas();
                        let browser_window = html_canvas
                            .owner_document()
                            .and_then(|doc| doc.default_view())
                            .or_else(web_sys::window)
                            .unwrap();

                        window.set_inner_size(dpi::LogicalSize {
                            width: browser_window.inner_width().unwrap().as_f64().unwrap(),
                            height: browser_window.inner_height().unwrap().as_f64().unwrap(),
                        });
                    }

                    let (physical_width, physical_height): (u32, u32) = window.inner_size().into();
                    let device_pixel_ratio = window.scale_factor();
                    let (logical_width, logical_height): (u32, u32) = window
                        .inner_size()
                        .to_logical::<f64>(device_pixel_ratio)
                        .into();
                    let frame_input = FrameInput {
                        incoming_events: rlc.events.drain(..).collect(),
                        outgoing_events: Vec::new(),
                        elapsed_time_ms: elapsed_time,
                        accumulated_time_ms: rlc.accumulated_time,
                        logical_size: Viewport::new_at_origin(logical_width, logical_height),
                        physical_size: Viewport::new_at_origin(physical_width, physical_height),
                        device_pixel_ratio,
                    };
                    rlc.first_frame = false;
                    let frame_output = Window::inner_render_loop(&mut app, frame_input);

                    for event in frame_output.events.unwrap() {
                        match event {
                            OutgoingEvent::CursorChanged(cursor_icon) => {
                                window.set_cursor_icon(cursor_icon);
                            }
                            OutgoingEvent::Exit => {
                                // info!("exit requested");
                                *control_flow = ControlFlow::Exit;
                            }
                        }
                    }

                    if frame_output.exit || *control_flow == ControlFlow::Exit {
                        *control_flow = ControlFlow::Exit;
                    } else {
                        if frame_output.swap_buffers {
                            #[cfg(not(target_arch = "wasm32"))]
                            gl.swap_buffers().unwrap();
                        }
                        if frame_output.wait_next_event {
                            *control_flow = ControlFlow::Wait;
                        } else {
                            *control_flow = ControlFlow::Poll;
                            window.request_redraw();
                        }
                    }
                }
                WinitEvent::WindowEvent { ref event, .. } => match event {
                    WindowEvent::Resized(physical_size) => {
                        gl.resize(*physical_size);
                    }
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                        ExitActionContainer::set("exit".to_string());
                    }
                    WindowEvent::KeyboardInput { input, .. } => {
                        if let Some(keycode) = input.virtual_keycode {
                            let state = input.state == ElementState::Pressed;
                            if let Some(kind) = translate_virtual_key_code(keycode) {
                                match kind {
                                    Key::LCtrl | Key::RCtrl => {
                                        rlc.modifiers.ctrl = state;
                                        if !cfg!(target_os = "macos") {
                                            rlc.modifiers.command = state;
                                        }
                                    }
                                    Key::LAlt | Key::RAlt => {
                                        rlc.modifiers.alt = state;
                                    }
                                    Key::LShift | Key::RShift => {
                                        rlc.modifiers.shift = state;
                                    }
                                    _ => {}
                                }

                                if state {
                                    rlc.events
                                        .push(IncomingEvent::KeyPress(kind, rlc.modifiers));
                                } else {
                                    rlc.events
                                        .push(IncomingEvent::KeyRelease(kind, rlc.modifiers));
                                }
                            } else if (keycode == VirtualKeyCode::LWin
                                || keycode == VirtualKeyCode::RWin)
                                && cfg!(target_os = "macos")
                            {
                                rlc.modifiers.command = state;
                            }
                        }
                    }
                    WindowEvent::MouseWheel { delta, .. } => {
                        if let Some(position) = rlc.cursor_pos {
                            match delta {
                                winit::event::MouseScrollDelta::LineDelta(x, y) => {
                                    let line_height = 24.0; // TODO
                                    rlc.events.push(IncomingEvent::MouseWheel(
                                        ((*x * line_height) as f64, (*y * line_height) as f64),
                                        position,
                                        rlc.modifiers,
                                    ));
                                }
                                winit::event::MouseScrollDelta::PixelDelta(delta) => {
                                    let d = delta.to_logical(window.scale_factor());
                                    rlc.events.push(IncomingEvent::MouseWheel(
                                        (d.x, d.y),
                                        position,
                                        rlc.modifiers,
                                    ));
                                }
                            }
                        }
                    }
                    WindowEvent::MouseInput { state, button, .. } => {
                        if let Some(position) = rlc.cursor_pos {
                            let button = match button {
                                WinitMouseButton::Left => Some(MouseButton::Left),
                                WinitMouseButton::Middle => Some(MouseButton::Middle),
                                WinitMouseButton::Right => Some(MouseButton::Right),
                                _ => None,
                            };
                            if let Some(button) = button {
                                rlc.events.push(if *state == ElementState::Pressed {
                                    rlc.mouse_pressed = Some(button);
                                    IncomingEvent::MousePress(button, position, rlc.modifiers)
                                } else {
                                    rlc.mouse_pressed = None;
                                    IncomingEvent::MouseRelease(button, position, rlc.modifiers)
                                });
                            }
                        }
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        let p = position.to_logical(window.scale_factor());
                        let delta = if let Some(last_pos) = rlc.cursor_pos {
                            (p.x - last_pos.0, p.y - last_pos.1)
                        } else {
                            (0.0, 0.0)
                        };
                        rlc.events.push(IncomingEvent::MouseMotion(
                            rlc.mouse_pressed,
                            delta,
                            (p.x, p.y),
                            rlc.modifiers,
                        ));
                        rlc.cursor_pos = Some((p.x, p.y));
                    }
                    WindowEvent::ReceivedCharacter(ch) => {
                        if is_printable_char(*ch) && !rlc.modifiers.ctrl && !rlc.modifiers.command {
                            rlc.events.push(IncomingEvent::Text(*ch));
                        }
                    }
                    WindowEvent::CursorLeft { .. } => {
                        rlc.mouse_pressed = None;
                    }
                    WindowEvent::Touch(touch) => {
                        let position = touch
                            .location
                            .to_logical::<f64>(window.scale_factor())
                            .into();
                        match touch.phase {
                            TouchPhase::Started => {
                                if rlc.finger_id.is_none() {
                                    rlc.events.push(IncomingEvent::MousePress(
                                        MouseButton::Left,
                                        position,
                                        rlc.modifiers,
                                    ));
                                    rlc.cursor_pos = Some(position);
                                    rlc.finger_id = Some(touch.id);
                                } else if rlc.secondary_finger_id.is_none() {
                                    rlc.secondary_cursor_pos = Some(position);
                                    rlc.secondary_finger_id = Some(touch.id);
                                }
                            }
                            TouchPhase::Ended | TouchPhase::Cancelled => {
                                if rlc.finger_id.map(|id| id == touch.id).unwrap_or(false) {
                                    rlc.events.push(IncomingEvent::MouseRelease(
                                        MouseButton::Left,
                                        position,
                                        rlc.modifiers,
                                    ));
                                    rlc.cursor_pos = None;
                                    rlc.finger_id = None;
                                } else if rlc
                                    .secondary_finger_id
                                    .map(|id| id == touch.id)
                                    .unwrap_or(false)
                                {
                                    rlc.secondary_cursor_pos = None;
                                    rlc.secondary_finger_id = None;
                                }
                            }
                            TouchPhase::Moved => {
                                if rlc.finger_id.map(|id| id == touch.id).unwrap_or(false) {
                                    let last_pos = rlc.cursor_pos.unwrap();
                                    if let Some(p) = rlc.secondary_cursor_pos {
                                        rlc.events.push(IncomingEvent::MouseWheel(
                                            (
                                                (position.0 - p.0).abs() - (last_pos.0 - p.0).abs(),
                                                (position.1 - p.1).abs() - (last_pos.1 - p.1).abs(),
                                            ),
                                            position,
                                            rlc.modifiers,
                                        ));
                                    } else {
                                        rlc.events.push(IncomingEvent::MouseMotion(
                                            Some(MouseButton::Left),
                                            (position.0 - last_pos.0, position.1 - last_pos.1),
                                            position,
                                            rlc.modifiers,
                                        ));
                                    }
                                    rlc.cursor_pos = Some(position);
                                } else if rlc
                                    .secondary_finger_id
                                    .map(|id| id == touch.id)
                                    .unwrap_or(false)
                                {
                                    let last_pos = rlc.secondary_cursor_pos.unwrap();
                                    if let Some(p) = rlc.cursor_pos {
                                        rlc.events.push(IncomingEvent::MouseWheel(
                                            (
                                                (position.0 - p.0).abs() - (last_pos.0 - p.0).abs(),
                                                (position.1 - p.1).abs() - (last_pos.1 - p.1).abs(),
                                            ),
                                            p,
                                            rlc.modifiers,
                                        ));
                                    }
                                    rlc.secondary_cursor_pos = Some(position);
                                }
                            }
                        }
                    }
                    _ => (),
                },
                _ => (),
            }
        };

        cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                EventLoop::<()>::spawn(event_loop, loop_func);
                return None;
            } else {

                // run
                EventLoop::<()>::run_return(&mut event_loop, loop_func);

                // cleanup

                // put eventloop back
                self.event_loop_opt = Some(event_loop);

                // reset gl context
                // info!("clean up gl context");
                let mut context = Context::get();
                context.unload_programs();

                // return window
                return Some(self);
            }
        }
    }
}

fn is_printable_char(chr: char) -> bool {
    let is_in_private_use_area = ('\u{e000}'..='\u{f8ff}').contains(&chr)
        || ('\u{f0000}'..='\u{ffffd}').contains(&chr)
        || ('\u{100000}'..='\u{10fffd}').contains(&chr);

    !is_in_private_use_area && !chr.is_ascii_control()
}

fn translate_virtual_key_code(key: VirtualKeyCode) -> Option<Key> {
    Some(match key {
        VirtualKeyCode::Down => Key::ArrowDown,
        VirtualKeyCode::Left => Key::ArrowLeft,
        VirtualKeyCode::Right => Key::ArrowRight,
        VirtualKeyCode::Up => Key::ArrowUp,

        VirtualKeyCode::Escape => Key::Escape,
        VirtualKeyCode::Tab => Key::Tab,
        VirtualKeyCode::Back => Key::Backspace,
        VirtualKeyCode::Return | VirtualKeyCode::NumpadEnter => Key::Enter,
        VirtualKeyCode::Space => Key::Space,

        VirtualKeyCode::Insert => Key::Insert,
        VirtualKeyCode::Delete => Key::Delete,
        VirtualKeyCode::Home => Key::Home,
        VirtualKeyCode::End => Key::End,
        VirtualKeyCode::PageUp => Key::PageUp,
        VirtualKeyCode::PageDown => Key::PageDown,

        VirtualKeyCode::Key0 | VirtualKeyCode::Numpad0 => Key::Num0,
        VirtualKeyCode::Key1 | VirtualKeyCode::Numpad1 => Key::Num1,
        VirtualKeyCode::Key2 | VirtualKeyCode::Numpad2 => Key::Num2,
        VirtualKeyCode::Key3 | VirtualKeyCode::Numpad3 => Key::Num3,
        VirtualKeyCode::Key4 | VirtualKeyCode::Numpad4 => Key::Num4,
        VirtualKeyCode::Key5 | VirtualKeyCode::Numpad5 => Key::Num5,
        VirtualKeyCode::Key6 | VirtualKeyCode::Numpad6 => Key::Num6,
        VirtualKeyCode::Key7 | VirtualKeyCode::Numpad7 => Key::Num7,
        VirtualKeyCode::Key8 | VirtualKeyCode::Numpad8 => Key::Num8,
        VirtualKeyCode::Key9 | VirtualKeyCode::Numpad9 => Key::Num9,

        VirtualKeyCode::A => Key::A,
        VirtualKeyCode::B => Key::B,
        VirtualKeyCode::C => Key::C,
        VirtualKeyCode::D => Key::D,
        VirtualKeyCode::E => Key::E,
        VirtualKeyCode::F => Key::F,
        VirtualKeyCode::G => Key::G,
        VirtualKeyCode::H => Key::H,
        VirtualKeyCode::I => Key::I,
        VirtualKeyCode::J => Key::J,
        VirtualKeyCode::K => Key::K,
        VirtualKeyCode::L => Key::L,
        VirtualKeyCode::M => Key::M,
        VirtualKeyCode::N => Key::N,
        VirtualKeyCode::O => Key::O,
        VirtualKeyCode::P => Key::P,
        VirtualKeyCode::Q => Key::Q,
        VirtualKeyCode::R => Key::R,
        VirtualKeyCode::S => Key::S,
        VirtualKeyCode::T => Key::T,
        VirtualKeyCode::U => Key::U,
        VirtualKeyCode::V => Key::V,
        VirtualKeyCode::W => Key::W,
        VirtualKeyCode::X => Key::X,
        VirtualKeyCode::Y => Key::Y,
        VirtualKeyCode::Z => Key::Z,

        VirtualKeyCode::Plus | VirtualKeyCode::NumpadAdd => Key::Plus,
        VirtualKeyCode::Slash | VirtualKeyCode::NumpadDivide => Key::Slash,
        VirtualKeyCode::Period | VirtualKeyCode::NumpadDecimal => Key::Period,
        VirtualKeyCode::Equals | VirtualKeyCode::NumpadEquals => Key::Equals,
        VirtualKeyCode::Asterisk | VirtualKeyCode::NumpadMultiply => Key::Asterisk,
        VirtualKeyCode::Minus | VirtualKeyCode::NumpadSubtract => Key::Minus,
        VirtualKeyCode::Grave => Key::Grave,
        VirtualKeyCode::LBracket => Key::LBracket,
        VirtualKeyCode::RBracket => Key::RBracket,
        VirtualKeyCode::Backslash => Key::Backslash,
        VirtualKeyCode::Semicolon => Key::Semicolon,
        VirtualKeyCode::Apostrophe => Key::Apostrophe,
        VirtualKeyCode::Comma => Key::Comma,

        VirtualKeyCode::VolumeDown => Key::VolumeDown,
        VirtualKeyCode::VolumeUp => Key::VolumeUp,

        VirtualKeyCode::LShift => Key::LShift,
        VirtualKeyCode::LControl => Key::LCtrl,
        VirtualKeyCode::LAlt => Key::LAlt,
        VirtualKeyCode::RShift => Key::RShift,
        VirtualKeyCode::RControl => Key::RCtrl,
        VirtualKeyCode::RAlt => Key::RAlt,

        _ => return None,
    })
}
