use bevy_ecs::prelude::Resource;
use bevy_log::info;

use math::Vec2;

use crate::IncomingEvent;

#[derive(Resource)]
pub struct Input {
    mouse_coords: Vec2,
}

impl Input {
    pub fn new() -> Self {
        Self {
            mouse_coords: Vec2::ZERO,
        }
    }

    pub fn mouse(&self) -> &Vec2 {
        &self.mouse_coords
    }

    pub fn recv_events(&mut self, events: &Vec<IncomingEvent<()>>) {
        for event in events {
            match event {
                IncomingEvent::MousePress { .. } => {}
                IncomingEvent::MouseRelease { .. } => {}
                IncomingEvent::MouseMotion {
                    position, handled, ..
                } => {
                    if *handled {
                        continue;
                    }
                    self.mouse_coords.x = position.0 as f32;
                    self.mouse_coords.y = position.1 as f32;
                    info!("mouse moved, new coords: {:?}", self.mouse_coords);
                }
                IncomingEvent::MouseWheel { .. } => {}
                IncomingEvent::MouseEnter => {}
                IncomingEvent::MouseLeave => {}
                IncomingEvent::KeyPress { .. } => {}
                IncomingEvent::KeyRelease { .. } => {}
                IncomingEvent::ModifiersChange { .. } => {}
                IncomingEvent::Text(_) => {}
                IncomingEvent::UserEvent(_) => {}
            }
        }
    }
}