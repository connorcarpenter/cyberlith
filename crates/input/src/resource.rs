use std::collections::HashSet;

use bevy_ecs::system::Resource;

use math::Vec2;

use crate::{is_button::IsButton, IncomingEvent, Key, MouseButton};

#[derive(Resource)]
pub struct Input {
    mouse_offset: Vec2,
    mouse_coords: Vec2,
    mouse_buttons: HashSet<MouseButton>,
    keys: HashSet<Key>,
    mouse_scroll_y: f32,
}

impl Input {
    pub fn new() -> Self {
        Self {
            mouse_coords: Vec2::ZERO,
            mouse_buttons: HashSet::new(),
            keys: HashSet::new(),
            mouse_offset: Vec2::ZERO,
            mouse_scroll_y: 0.0,
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
                    button, handled, ..
                } => {
                    if *handled {
                        continue;
                    }
                    self.mouse_buttons.insert(*button);
                }
                IncomingEvent::MouseRelease {
                    button, handled, ..
                } => {
                    if *handled {
                        continue;
                    }
                    self.mouse_buttons.remove(button);
                }
                IncomingEvent::MouseMotion {
                    position, handled, ..
                } => {
                    if *handled {
                        continue;
                    }
                    self.mouse_coords.x = (position.0 as f32) - self.mouse_offset.x;
                    self.mouse_coords.y = (position.1 as f32) - self.mouse_offset.y;
                }
                IncomingEvent::MouseWheel { delta, .. } => {
                    // for now, only pass Y value
                    self.mouse_scroll_y += delta.1 as f32;
                }
                IncomingEvent::MouseEnter => {}
                IncomingEvent::MouseLeave => {}
                IncomingEvent::KeyPress { kind, handled, .. } => {
                    if *handled {
                        continue;
                    }
                    self.keys.insert(*kind);
                }
                IncomingEvent::KeyRelease { kind, handled, .. } => {
                    if *handled {
                        continue;
                    }
                    self.keys.remove(kind);
                }
                IncomingEvent::ModifiersChange { .. } => {}
                IncomingEvent::Text(_) => {}
                IncomingEvent::UserEvent(_) => {}
            }
        }
    }
}
