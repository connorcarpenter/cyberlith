use bevy_ecs::system::Resource;

use math::Vec2;

use render_api::{base::CpuTexture2D, Handle};
use crate::app::resources::edge_manager::EdgeManager;

use crate::app::resources::shape_manager::ShapeManager;
use crate::app::resources::vertex_manager::VertexManager;

#[derive(Resource)]
pub struct Canvas {
    canvas_texture: Option<Handle<CpuTexture2D>>,
    canvas_texture_size: Vec2,
    is_visible: bool,
    next_visible: bool,
    has_focus: bool,
    focus_timer: u8,
}

impl Default for Canvas {
    fn default() -> Self {
        Self {
            next_visible: false,
            is_visible: false,
            canvas_texture: None,
            canvas_texture_size: Vec2::new(1280.0, 720.0),
            has_focus: false,
            focus_timer: 0,
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

    pub fn has_focus(&self) -> bool {
        self.has_focus
    }

    pub fn set_focus(&mut self, shape_manager: &mut ShapeManager, vertex_manager: &mut VertexManager, edge_manager: &mut EdgeManager, focus: bool) {
        if !focus && self.has_focus && self.focus_timer > 0 {
            self.focus_timer -= 1;
            return;
        }
        self.has_focus = focus;

        shape_manager.on_canvas_focus_changed(vertex_manager, edge_manager, focus);
    }

    pub fn set_focused_timed(&mut self, shape_manager: &mut ShapeManager, vertex_manager: &mut VertexManager, edge_manager: &mut EdgeManager) {
        self.has_focus = true;
        self.focus_timer = 1;

        shape_manager.on_canvas_focus_changed(vertex_manager, edge_manager, true);
    }

    pub(crate) fn is_position_inside(&self, pos: Vec2) -> bool {
        // check if position is inside self.canvas_texture_size
        if pos.x < 2.0 || pos.y < 2.0 {
            return false;
        }
        if pos.x > self.canvas_texture_size.x - 2.0 || pos.y > self.canvas_texture_size.y - 2.0 {
            return false;
        }
        return true;
    }
}
