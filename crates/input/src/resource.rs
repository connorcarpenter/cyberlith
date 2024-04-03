use std::collections::HashSet;

use bevy_ecs::{
    event::EventWriter,
    system::Resource,
};
use bevy_log::info;

use clipboard::ClipboardManager;
use instant::Instant;
use math::Vec2;

use crate::{
    gamepad::{
        Axis, GamepadAxis, GamepadButton, GamepadButtonType, GamepadId, GamepadInfo, Gamepads,
        ALL_AXIS_TYPES, ALL_BUTTON_TYPES,
    },
    is_button::IsButton,
    CursorIcon, GamepadSettings, IncomingEvent, InputEvent, Joystick, Key, MouseButton,
};

#[derive(Resource)]
pub struct Input {
    mouse_offset: Vec2,
    mouse_coords: Vec2,
    mouse_delta: Vec2,
    mouse_scroll_y: f32,
    pressed_mouse_buttons: HashSet<MouseButton>,
    pressed_keys: HashSet<Key>,
    enabled: bool,

    last_mouse_position: Vec2,
    has_canvas_props: bool,

    gamepad_settings: GamepadSettings,
    gamepads: Gamepads,
    gamepad_axis: Axis<GamepadAxis>,
    gamepad_button_axis: Axis<GamepadButton>,
    pressed_gamepad_buttons: HashSet<GamepadButton>,

    cursor_change: Option<CursorIcon>,
    quick_clicks: u8,
    last_click_instant: Instant,
    last_click_button: MouseButton,
}

impl Input {
    pub fn new() -> Self {
        Self {
            mouse_coords: Vec2::ZERO,
            pressed_mouse_buttons: HashSet::new(),
            mouse_offset: Vec2::ZERO,
            mouse_scroll_y: 0.0,
            last_mouse_position: Vec2::ZERO,
            pressed_keys: HashSet::new(),
            enabled: false,
            has_canvas_props: false,
            mouse_delta: Vec2::ZERO,

            gamepad_settings: GamepadSettings::default(),
            gamepads: Gamepads::default(),
            gamepad_axis: Axis::default(),
            gamepad_button_axis: Axis::default(),
            pressed_gamepad_buttons: HashSet::new(),

            cursor_change: None,
            quick_clicks: 0,
            last_click_instant: Instant::now(),
            last_click_button: MouseButton::Left,
        }
    }

    pub fn set_cursor_icon(&mut self, cursor_icon: CursorIcon) {
        self.cursor_change = Some(cursor_icon);
    }

    pub fn take_cursor_icon(&mut self) -> Option<CursorIcon> {
        self.cursor_change.take()
    }

    pub fn mouse_position(&self) -> &Vec2 {
        &self.mouse_coords
    }

    pub fn is_pressed<T: IsButton>(&self, button: T) -> bool {
        button.is_pressed(
            &self.pressed_mouse_buttons,
            &self.pressed_keys,
            &self.pressed_gamepad_buttons,
        )
    }

    pub fn has_canvas_properties(&self) -> bool {
        self.has_canvas_props
    }

    pub fn update_canvas_properties(&mut self, offset_x: f32, offset_y: f32) {
        self.mouse_offset.x = offset_x;
        self.mouse_offset.y = offset_y;
        self.has_canvas_props = true;
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn set_mouse_coords(&mut self, position: &(f64, f64)) {
        self.mouse_coords.x = (position.0 as f32) - self.mouse_offset.x;
        self.mouse_coords.y = (position.1 as f32) - self.mouse_offset.y;
    }

    fn set_mouse_delta(&mut self, last_mouse_position: Vec2, mouse_position: Vec2) {
        self.mouse_delta = mouse_position - last_mouse_position;
    }

    // should only be used in `render_gl` crate
    pub fn recv_events(
        &mut self,
        clipboard: &mut ClipboardManager,
        event_writer: &mut EventWriter<InputEvent>,
        events: &Vec<IncomingEvent>
    ) {
        if !self.enabled {
            return;
        }
        for event in events {
            match event {
                IncomingEvent::MousePress(button, position, modifiers) => {
                    if !self.pressed_mouse_buttons.contains(button) {
                        self.set_mouse_coords(position);
                        self.pressed_mouse_buttons.insert(*button);

                        if self.last_click_button == *button
                            && self.last_click_instant.elapsed().as_millis() < 400
                        {
                            // TODO: put doubleclick time in settings
                            self.quick_clicks += 1;
                        } else {
                            self.quick_clicks = 1;
                        }
                        self.last_click_instant = Instant::now();
                        self.last_click_button = *button;
                        match self.quick_clicks {
                            2 => {
                                event_writer.send(InputEvent::MouseDoubleClicked(
                                    *button,
                                    self.mouse_coords,
                                    modifiers.clone(),
                                ));
                            }
                            3 => {
                                event_writer.send(InputEvent::MouseTripleClicked(
                                    *button,
                                    self.mouse_coords,
                                    modifiers.clone(),
                                ));
                            }
                            _ => {
                                event_writer.send(InputEvent::MouseClicked(
                                    *button,
                                    self.mouse_coords,
                                    modifiers.clone(),
                                ));
                            }
                        }
                    }
                }
                IncomingEvent::MouseRelease(button, _position, _modifiers) => {
                    if self.pressed_mouse_buttons.contains(button) {
                        event_writer.send(InputEvent::MouseReleased(*button));
                        self.pressed_mouse_buttons.remove(button);
                    }
                }
                IncomingEvent::MouseMotion(_button, _delta, position, modifiers) => {
                    self.set_mouse_coords(position);

                    if self.mouse_coords.x as i16 != self.last_mouse_position.x as i16
                        || self.mouse_coords.y as i16 != self.last_mouse_position.y as i16
                    {
                        // mouse moved!
                        self.set_mouse_delta(self.last_mouse_position, self.mouse_coords);
                        self.last_mouse_position = self.mouse_coords;

                        for mouse_button in self.pressed_mouse_buttons.iter() {
                            event_writer.send(InputEvent::MouseDragged(
                                *mouse_button,
                                self.mouse_coords,
                                self.mouse_delta,
                                *modifiers,
                            ));
                        }

                        event_writer.send(InputEvent::MouseMoved(self.mouse_coords));
                    }
                }
                IncomingEvent::MouseWheel(delta, _position, _modifiers) => {
                    // for now, only pass Y value
                    self.mouse_scroll_y += delta.1 as f32;

                    // mouse wheel zoom..
                    if self.mouse_scroll_y > 0.1 || self.mouse_scroll_y < -0.1 {
                        event_writer.send(InputEvent::MouseMiddleScrolled(self.mouse_scroll_y));
                        self.mouse_scroll_y = 0.0;
                    }
                }
                IncomingEvent::KeyPress(kind, modifiers) => {
                    if modifiers.ctrl {
                        match kind {
                            Key::C => {
                                event_writer.send(InputEvent::Copy);
                                continue;
                            }
                            Key::X => {
                                event_writer.send(InputEvent::Cut);
                                continue;
                            }
                            Key::V => {
                                if let Some(text) = clipboard.get_contents() {
                                    event_writer.send(InputEvent::Paste(text));
                                }
                                continue;
                            }
                            _ => {}
                        }
                    }

                    if !self.pressed_keys.contains(kind) {
                        event_writer.send(InputEvent::KeyPressed(*kind, *modifiers));
                        self.pressed_keys.insert(*kind);
                    }
                }
                IncomingEvent::KeyRelease(kind, _modifiers) => {
                    if self.pressed_keys.contains(kind) {
                        event_writer.send(InputEvent::KeyReleased(*kind));
                        self.pressed_keys.remove(kind);
                    }
                }
                IncomingEvent::Text(c) => {
                    event_writer.send(InputEvent::Text(*c));
                }
            }
        }
    }

    // gamepad stuff

    pub fn joystick_position(&self, joystick: Joystick) -> Vec2 {
        let Joystick {
            gamepad,
            joystick_type,
        } = joystick;

        let x_axis = joystick_type.x_axis();
        let y_axis = joystick_type.y_axis();

        let x = self
            .gamepad_axis
            .get(GamepadAxis::new(gamepad, x_axis))
            .unwrap_or(0.0);
        let y = self
            .gamepad_axis
            .get(GamepadAxis::new(gamepad, y_axis))
            .unwrap_or(0.0);

        // INVERT Y
        cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                use crate::JoystickType;
                let y = if JoystickType::Left == joystick_type { -y } else { y };
            }
            else {
                let y = -y;
            }
        }

        Vec2::new(x, y)
    }

    pub fn gamepad_settings(&self) -> &GamepadSettings {
        &self.gamepad_settings
    }

    pub fn gamepad_settings_mut(&mut self) -> &mut GamepadSettings {
        &mut self.gamepad_settings
    }

    pub fn gamepads_iter(&self) -> impl Iterator<Item = GamepadId> + '_ {
        self.gamepads.iter()
    }

    pub fn gamepad_first(&self) -> Option<GamepadId> {
        self.gamepads_iter().next()
    }

    pub(crate) fn gamepad_axis_get(&self, axis: GamepadAxis) -> Option<f32> {
        self.gamepad_axis.get(axis)
    }

    pub(crate) fn gamepad_button_press(&mut self, input: GamepadButton) {
        self.pressed_gamepad_buttons.insert(input);
    }

    pub(crate) fn gamepad_button_release(&mut self, input: GamepadButton) {
        self.pressed_gamepad_buttons.remove(&input);
    }

    pub(crate) fn gamepad_button_reset(&mut self, input: GamepadButton) {
        self.pressed_gamepad_buttons.remove(&input);
    }

    pub(crate) fn recv_gilrs_gamepad_connect(
        &mut self,
        event_writer: &mut EventWriter<InputEvent>,
        id: GamepadId,
        info: GamepadInfo
    ) {
        event_writer.send(InputEvent::GamepadConnected(id));

        self.gamepads.register(id, info.clone());
        info!("{:?} Connected", id);

        for button_type in &ALL_BUTTON_TYPES {
            let gamepad_button = GamepadButton::new(id, *button_type);
            self.gamepad_button_reset(gamepad_button);
            self.gamepad_button_axis.set(gamepad_button, 0.0);
        }
        for axis_type in &ALL_AXIS_TYPES {
            self.gamepad_axis.set(GamepadAxis::new(id, *axis_type), 0.0);
        }
    }

    pub(crate) fn recv_gilrs_gamepad_disconnect(
        &mut self,
        event_writer: &mut EventWriter<InputEvent>,
        id: GamepadId
    ) {
        event_writer.send(InputEvent::GamepadDisconnected(id));

        self.gamepads.deregister(id);
        info!("{:?} Disconnected", id);

        for button_type in &ALL_BUTTON_TYPES {
            let gamepad_button = GamepadButton::new(id, *button_type);
            self.gamepad_button_reset(gamepad_button);
            self.gamepad_button_axis.remove(gamepad_button);
        }
        for axis_type in &ALL_AXIS_TYPES {
            self.gamepad_axis.remove(GamepadAxis::new(id, *axis_type));
        }
    }

    pub(crate) fn recv_gilrs_button_press(
        &mut self,
        event_writer: &mut EventWriter<InputEvent>,
        id: GamepadId,
        button: GamepadButtonType
    ) {
        event_writer.send(InputEvent::GamepadButtonPressed(id, button));
    }

    pub(crate) fn recv_gilrs_button_release(
        &mut self,
        event_writer: &mut EventWriter<InputEvent>,
        id: GamepadId,
        button: GamepadButtonType
    ) {
        event_writer.send(InputEvent::GamepadButtonReleased(id, button));
    }

    pub(crate) fn gamepad_axis_set(
        &mut self,
        event_writer: &mut EventWriter<InputEvent>,
        axis: GamepadAxis,
        val: f32
    ) {
        self.gamepad_axis.set(axis, val);
        let joystick_type = axis.axis_type.to_joystick();
        let joystick_position = self.joystick_position(Joystick::new(axis.gamepad, joystick_type));
        event_writer.send(InputEvent::GamepadJoystickMoved(
            axis.gamepad,
            joystick_type,
            joystick_position,
        ));
    }

    pub(crate) fn gamepad_button_axis_get(&self, button: GamepadButton) -> Option<f32> {
        self.gamepad_button_axis.get(button)
    }

    pub(crate) fn gamepad_button_axis_set(&mut self, button: GamepadButton, val: f32) {
        self.gamepad_button_axis.set(button, val);
    }
}
