use std::collections::{HashMap, HashSet};

use bevy_ecs::system::Resource;

use math::Vec2;

use crate::{is_button::IsButton, IncomingEvent, Key, MouseButton, InputAction};

#[derive(Resource)]
pub struct Input {
    mouse_offset: Vec2,
    mouse_coords: Vec2,
    mouse_buttons: HashMap<MouseButton, Vec2>,
    mouse_scroll_y: f32,
    last_mouse_position: Vec2,
    keys: HashSet<Key>,
    outgoing_actions: Vec<InputAction>,
}

impl Input {
    pub fn new() -> Self {
        Self {
            mouse_coords: Vec2::ZERO,
            mouse_buttons: HashMap::new(),
            mouse_offset: Vec2::ZERO,
            mouse_scroll_y: 0.0,
            last_mouse_position: Vec2::ZERO,
            keys: HashSet::new(),
            outgoing_actions: Vec::new(),
        }
    }

    pub fn set_mouse_offset(&mut self, x: f32, y: f32) {
        self.mouse_offset.x = x;
        self.mouse_offset.y = y;
    }

    pub fn mouse_position(&self) -> &Vec2 {
        &self.mouse_coords
    }

    pub fn consume_mouse_scroll(&mut self) -> f32 {
        let scroll = self.mouse_scroll_y;
        self.mouse_scroll_y = 0.0;
        scroll
    }

    pub fn is_pressed<T: IsButton>(&self, button: T) -> bool {
        button.is_pressed(&self.mouse_buttons, &self.keys)
    }

    pub fn recv_events(&mut self, events: &Vec<IncomingEvent<()>>) {
        for event in events {
            match event {
                IncomingEvent::MousePress {
                    button, handled, position, ..
                } => {
                    if *handled {
                        continue;
                    }
                    if !self.mouse_buttons.contains_key(button) {
                        let mouse_coords_x = (position.0 as f32) - self.mouse_offset.x;
                        let mouse_coords_y = (position.1 as f32) - self.mouse_offset.y;
                        let mouse_coords = Vec2::new(mouse_coords_x, mouse_coords_y);
                        self.outgoing_actions.push(InputAction::MouseClick(*button, mouse_coords));
                        self.mouse_buttons.insert(*button, mouse_coords);
                    }
                }
                IncomingEvent::MouseRelease {
                    button, handled, ..
                } => {
                    if *handled {
                        continue;
                    }
                    if self.mouse_buttons.contains_key(button) {
                        self.outgoing_actions.push(InputAction::MouseRelease(*button));
                        self.mouse_buttons.remove(button);
                    }
                }
                IncomingEvent::MouseMotion {
                    position, handled, ..
                } => {
                    if *handled {
                        continue;
                    }
                    self.mouse_coords.x = (position.0 as f32) - self.mouse_offset.x;
                    self.mouse_coords.y = (position.1 as f32) - self.mouse_offset.y;

                    if self.mouse_coords.x as i16 != self.last_mouse_position.x as i16
                        || self.mouse_coords.y as i16 != self.last_mouse_position.y as i16
                    {
                        // mouse moved!
                        self.last_mouse_position = self.mouse_coords;

                        for (mouse_button, click_start) in self.mouse_buttons.iter() {
                            let delta = self.mouse_coords - *click_start;
                            self.outgoing_actions.push(InputAction::MouseDragged(*mouse_button, self.mouse_coords, delta));
                        }

                        self.outgoing_actions.push(InputAction::MouseMoved);
                    }
                }
                IncomingEvent::MouseWheel { delta, .. } => {
                    // for now, only pass Y value
                    self.mouse_scroll_y += delta.1 as f32;

                    // mouse wheel zoom..
                    if self.mouse_scroll_y > 0.1 || self.mouse_scroll_y < -0.1 {
                        self.mouse_scroll_y = 0.0;
                        self.outgoing_actions.push(InputAction::MiddleMouseScroll(self.mouse_scroll_y));
                    }
                }
                IncomingEvent::KeyPress { kind, handled, .. } => {
                    if *handled {
                        continue;
                    }
                    if !self.keys.contains(kind) {
                        self.outgoing_actions.push(InputAction::KeyPress(*kind));
                        self.keys.insert(*kind);
                    }
                }
                IncomingEvent::KeyRelease { kind, handled, .. } => {
                    if *handled {
                        continue;
                    }
                    if self.keys.contains(kind) {
                        self.outgoing_actions.push(InputAction::KeyRelease(*kind));
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

    pub fn take_actions(&mut self) -> Vec<InputAction> {
        std::mem::take(&mut self.outgoing_actions)
    }
}
