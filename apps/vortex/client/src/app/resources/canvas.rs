use bevy_ecs::prelude::Resource;

use math::Vec2;

use render_api::{base::CpuTexture2D, Handle};

#[derive(Resource)]
pub struct Canvas {
    canvas_texture: Option<Handle<CpuTexture2D>>,
    canvas_texture_size: Vec2,
    is_visible: bool,
    next_visible: bool,
}

impl Default for Canvas {
    fn default() -> Self {
        Self {
            next_visible: false,
            is_visible: false,
            canvas_texture: None,
            canvas_texture_size: Vec2::new(1280.0, 720.0),
        }
    }
}

impl Canvas {
    // returns whether visibility changed
    pub fn update_visibility(&mut self) -> bool {
        if self.is_visible == self.next_visible {
            return false;
        }
        self.is_visible = self.next_visible;
        return true;
    }

    pub fn update_canvas_size(&mut self, texture_size: Vec2) {
        self.canvas_texture_size = texture_size;
    }

    pub fn canvas_texture(&self) -> Handle<CpuTexture2D> {
        self.canvas_texture.unwrap()
    }

    pub fn set_canvas_texture(&mut self, texture_size: Vec2, texture: Handle<CpuTexture2D>) {
        self.canvas_texture = Some(texture);
        self.canvas_texture_size = texture_size;
    }

    pub fn is_visible(&self) -> bool {
        self.is_visible
    }

    pub fn set_visibility(&mut self, visible: bool) {
        self.next_visible = visible;
    }
}
