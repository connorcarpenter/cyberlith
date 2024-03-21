use std::collections::HashSet;

use bevy_ecs::{
    event::EventWriter,
    system::{ResMut, Resource},
};

use math::Vec2;

use crate::{is_button::IsButton, IncomingEvent, InputEvent, Key, MouseButton};

#[derive(Resource)]
pub struct Input {
    mouse_offset: Vec2,
    mouse_coords: Vec2,
    mouse_delta: Vec2,
    mouse_scroll_y: f32,
    pressed_mouse_buttons: HashSet<MouseButton>,
    pressed_keys: HashSet<Key>,
    outgoing_actions: Vec<InputEvent>,
    enabled: bool,

    last_mouse_position: Vec2,
    has_canvas_props: bool,

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
            outgoing_actions: Vec::new(),
            enabled: false,
            has_canvas_props: false,
            mouse_delta: Vec2::ZERO,
        }
    }

    // will be used as system
    pub fn update(mut input: ResMut<Input>, mut event_writer: EventWriter<InputEvent>) {
        let events = std::mem::take(&mut input.outgoing_actions);
        for event in events {
            event_writer.send(event);
        }
    }

    pub fn mouse_position(&self) -> &Vec2 {
        &self.mouse_coords
    }

    pub fn is_pressed<T: IsButton>(&self, button: T) -> bool {
        button.is_pressed(&self.pressed_mouse_buttons, &self.pressed_keys)
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
    pub fn recv_events(&mut self, events: &Vec<IncomingEvent>) {
        if !self.enabled {
            return;
        }
        for event in events {
            match event {
                IncomingEvent::MousePress {
                    button,
                    position,
                    ..
                } => {
                    if !self.pressed_mouse_buttons.contains(button) {
                        self.set_mouse_coords(position);
                        self.outgoing_actions
                            .push(InputEvent::MouseClicked(*button, self.mouse_coords));
                        self.pressed_mouse_buttons.insert(*button);
                    }
                }
                IncomingEvent::MouseRelease {
                    button, ..
                } => {
                    if self.pressed_mouse_buttons.contains(button) {
                        self.outgoing_actions
                            .push(InputEvent::MouseReleased(*button));
                        self.pressed_mouse_buttons.remove(button);
                    }
                }
                IncomingEvent::MouseMotion {
                    position, ..
                } => {
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

                        self.outgoing_actions.push(InputEvent::MouseMoved(self.mouse_coords));
                    }
                }
                IncomingEvent::MouseWheel { delta, .. } => {
                    // for now, only pass Y value
                    self.mouse_scroll_y += delta.1 as f32;

                    // mouse wheel zoom..
                    if self.mouse_scroll_y > 0.1 || self.mouse_scroll_y < -0.1 {
                        self.outgoing_actions
                            .push(InputEvent::MouseMiddleScrolled(self.mouse_scroll_y));
                        self.mouse_scroll_y = 0.0;
                    }
                }
                IncomingEvent::KeyPress { kind, .. } => {
                    if !self.pressed_keys.contains(kind) {
                        self.outgoing_actions.push(InputEvent::KeyPressed(*kind));
                        self.pressed_keys.insert(*kind);
                    }
                }
                IncomingEvent::KeyRelease { kind, .. } => {
                    if self.pressed_keys.contains(kind) {
                        self.outgoing_actions.push(InputEvent::KeyReleased(*kind));
                        self.pressed_keys.remove(kind);
                    }
                }
                _ => {}
            }
        }
    }
}
