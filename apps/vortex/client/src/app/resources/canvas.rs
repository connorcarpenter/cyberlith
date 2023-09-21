use bevy_ecs::system::Resource;

use math::Vec2;

use render_api::{base::CpuTexture2D, Handle};

use crate::app::resources::{
    animation_manager::AnimationManager, edge_manager::EdgeManager, input_manager::InputManager,
    vertex_manager::VertexManager,
};

#[derive(Resource)]
pub struct Canvas {
    canvas_texture: Option<Handle<CpuTexture2D>>,
    canvas_texture_size: Vec2,
    is_visible: bool,
    next_visible: bool,
    has_focus: bool,
    focus_timer: u8,
    resync_shapes: u8,
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
            resync_shapes: 0,
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

    pub fn canvas_texture_size(&self) -> Vec2 {
        self.canvas_texture_size
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

    pub fn set_focus(
        &mut self,
        input_manager: &mut InputManager,
        vertex_manager: &mut VertexManager,
        edge_manager: &mut EdgeManager,
        animation_manager: &mut AnimationManager,
        focus: bool,
    ) {
        if !focus && self.has_focus && self.focus_timer > 0 {
            self.focus_timer -= 1;
            return;
        }
        self.has_focus = focus;

        Canvas::on_canvas_focus_changed(
            input_manager,
            vertex_manager,
            edge_manager,
            animation_manager,
            focus,
        );
    }

    pub fn set_focused_timed(
        &mut self,
        input_manager: &mut InputManager,
        vertex_manager: &mut VertexManager,
        edge_manager: &mut EdgeManager,
        animation_manager: &mut AnimationManager,
    ) {
        self.has_focus = true;
        self.focus_timer = 1;

        Canvas::on_canvas_focus_changed(
            input_manager,
            vertex_manager,
            edge_manager,
            animation_manager,
            true,
        );
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

    pub fn queue_resync_shapes(&mut self) {
        self.resync_shapes = 2;
    }

    pub fn queue_resync_shapes_light(&mut self) {
        self.resync_shapes = 1;
    }

    pub fn should_sync_shapes(&mut self) -> bool {
        if self.resync_shapes == 0 {
            return false;
        }

        self.resync_shapes -= 1;

        return true;
    }

    pub(crate) fn on_canvas_focus_changed(
        input_manager: &mut InputManager,
        vertex_manager: &mut VertexManager,
        edge_manager: &mut EdgeManager,
        animation_manager: &mut AnimationManager,
        new_focus: bool,
    ) {
        input_manager.queue_resync_selection_ui();
        if !new_focus {
            vertex_manager.reset_last_vertex_dragged();
            edge_manager.reset_last_edge_dragged();
            animation_manager.reset_last_rotation_dragged();
            input_manager.hovered_entity = None;
        }
    }
}
