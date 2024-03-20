use std::collections::HashSet;

use bevy_ecs::{
    event::EventWriter,
    system::{ResMut, Resource},
};

use math::Vec2;

use crate::{is_button::IsButton, IncomingEvent, InputEvent, Key, MouseButton};

#[derive(Resource)]
pub struct WinitInput {
    mouse_offset: Vec2,
    mouse_coords: Vec2,
    pressed_mouse_buttons: HashSet<MouseButton>,
    mouse_scroll_y: f32,
    last_mouse_position: Vec2,
    keys: HashSet<Key>,
    outgoing_actions: Vec<InputEvent>,
    enabled: bool,
    has_canvas_props: bool,
    mouse_delta: Vec2,
}

impl WinitInput {
    pub fn new() -> Self {
        Self {
            mouse_coords: Vec2::ZERO,
            pressed_mouse_buttons: HashSet::new(),
            mouse_offset: Vec2::ZERO,
            mouse_scroll_y: 0.0,
            last_mouse_position: Vec2::ZERO,
            keys: HashSet::new(),
            outgoing_actions: Vec::new(),
            enabled: false,
            has_canvas_props: false,
            mouse_delta: Vec2::ZERO,
        }
    }

    // will be used as system
    pub fn update(mut input: ResMut<WinitInput>, mut event_writer: EventWriter<InputEvent>) {
        let events = std::mem::take(&mut input.outgoing_actions);
        for event in events {
            event_writer.send(event);
        }
    }

    pub fn has_canvas_properties(&self) -> bool {
        self.has_canvas_props
    }

    pub fn update_canvas_properties(&mut self, offset_x: f32, offset_y: f32) {
        self.mouse_offset.x = offset_x;
        self.mouse_offset.y = offset_y;
        self.has_canvas_props = true;
    }

    pub fn mouse_position(&self) -> &Vec2 {
        &self.mouse_coords
    }

    pub fn is_pressed<T: IsButton>(&self, button: T) -> bool {
        button.is_pressed(&self.pressed_mouse_buttons, &self.keys)
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
    pub fn recv_events(&mut self, events: &Vec<IncomingEvent<()>>) {
        if !self.enabled {
            return;
        }
        for event in events {
            match event {
                IncomingEvent::MousePress {
                    button,
                    handled,
                    position,
                    ..
                } => {
                    if *handled {
                        continue;
                    }
                    if !self.pressed_mouse_buttons.contains(button) {
                        self.set_mouse_coords(position);
                        self.outgoing_actions
                            .push(InputEvent::MouseClick(*button, self.mouse_coords));
                        self.pressed_mouse_buttons.insert(*button);
                    }
                }
                IncomingEvent::MouseRelease {
                    button, handled, ..
                } => {
                    if *handled {
                        continue;
                    }
                    if self.pressed_mouse_buttons.contains(button) {
                        self.outgoing_actions
                            .push(InputEvent::MouseRelease(*button));
                        self.pressed_mouse_buttons.remove(button);
                    }
                }
                IncomingEvent::MouseMotion {
                    position, handled, ..
                } => {
                    if *handled {
                        continue;
                    }
                    self.set_mouse_coords(position);

                    if self.mouse_coords.x as i16 != self.last_mouse_position.x as i16
                        || self.mouse_coords.y as i16 != self.last_mouse_position.y as i16
                    {
                        // mouse moved!
                        self.set_mouse_delta(self.last_mouse_position, self.mouse_coords);
                        self.last_mouse_position = self.mouse_coords;

                        for mouse_button in self.pressed_mouse_buttons.iter() {
                            self.outgoing_actions.push(InputEvent::MouseDragged(
                                *mouse_button,
                                self.mouse_coords,
                                self.mouse_delta,
                            ));
                        }

                        self.outgoing_actions.push(InputEvent::MouseMoved);
                    }
                }
                IncomingEvent::MouseWheel { delta, .. } => {
                    // for now, only pass Y value
                    self.mouse_scroll_y += delta.1 as f32;

                    // mouse wheel zoom..
                    if self.mouse_scroll_y > 0.1 || self.mouse_scroll_y < -0.1 {
                        self.outgoing_actions
                            .push(InputEvent::MiddleMouseScroll(self.mouse_scroll_y));
                        self.mouse_scroll_y = 0.0;
                    }
                }
                IncomingEvent::KeyPress { kind, handled, .. } => {
                    if *handled {
                        continue;
                    }
                    if !self.keys.contains(kind) {
                        self.outgoing_actions.push(InputEvent::KeyPress(*kind));
                        self.keys.insert(*kind);
                    }
                }
                IncomingEvent::KeyRelease { kind, handled, .. } => {
                    if *handled {
                        continue;
                    }
                    if self.keys.contains(kind) {
                        self.outgoing_actions.push(InputEvent::KeyRelease(*kind));
                        self.keys.remove(kind);
                    }
                }
                IncomingEvent::MouseEnter => {}
                IncomingEvent::MouseLeave => {}
                IncomingEvent::ModifiersChange { .. } => {}
                IncomingEvent::Text(_) => {}
                IncomingEvent::UserEvent(_) => {}
            }
        }
    }
}
