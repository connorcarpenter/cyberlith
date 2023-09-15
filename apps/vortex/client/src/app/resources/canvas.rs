use bevy_ecs::{
    entity::Entity,
    system::{Query, Resource},
};

use math::Vec2;

use render_api::{
    base::CpuTexture2D,
    components::{Camera, Projection, Transform, Visibility},
    Handle,
};

use vortex_proto::components::{EdgeAngle, FileExtension, Vertex3d};

use crate::app::{
    components::{Edge2dLocal, Edge3dLocal, FaceIcon2d, LocalShape, OwnedByFileLocal},
    resources::{
        camera_manager::CameraManager, camera_state::CameraState, compass::Compass,
        edge_manager::EdgeManager, face_manager::FaceManager, input_manager::InputManager,
        vertex_manager::VertexManager,
    },
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
        focus: bool,
    ) {
        if !focus && self.has_focus && self.focus_timer > 0 {
            self.focus_timer -= 1;
            return;
        }
        self.has_focus = focus;

        Canvas::on_canvas_focus_changed(input_manager, vertex_manager, edge_manager, focus);
    }

    pub fn set_focused_timed(
        &mut self,
        input_manager: &mut InputManager,
        vertex_manager: &mut VertexManager,
        edge_manager: &mut EdgeManager,
    ) {
        self.has_focus = true;
        self.focus_timer = 1;

        Canvas::on_canvas_focus_changed(input_manager, vertex_manager, edge_manager, true);
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

    pub fn sync_shapes(
        &mut self,
        camera_manager: &CameraManager,
        camera_state: &CameraState,
        compass: &Compass,
        current_tab_file_entity: Entity,

        vertex_manager: &VertexManager,
        edge_manager: &EdgeManager,
        input_manager: &mut InputManager,

        camera_q: &Query<(&Camera, &Projection)>,
        local_shape_q: &Query<&LocalShape>,

        visibility_q: &mut Query<&mut Visibility>,
        transform_q: &mut Query<&mut Transform>,
        owned_by_q: &Query<&OwnedByFileLocal>,

        vertex_3d_q: &mut Query<(Entity, &mut Vertex3d)>,
        edge_2d_q: &Query<(Entity, &Edge2dLocal)>,
        edge_3d_q: &Query<(Entity, &Edge3dLocal, Option<&EdgeAngle>)>,
        face_2d_q: &Query<(Entity, &FaceIcon2d)>,
    ) {
        if self.resync_shapes == 0 {
            return;
        }

        let Some(camera_3d) = camera_manager.camera_3d_entity() else {
            return;
        };

        self.resync_shapes -= 1;

        input_manager.queue_resync_hover_ui();
        input_manager.queue_resync_selection_ui();

        let camera_3d_scale = camera_state.camera_3d_scale();

        compass.sync_compass(&camera_3d, camera_state, vertex_3d_q, &transform_q);
        vertex_manager.sync_vertices(
            &camera_3d,
            camera_3d_scale,
            camera_q,
            vertex_3d_q,
            transform_q,
            owned_by_q,
            local_shape_q,
            current_tab_file_entity,
        );
        EdgeManager::sync_2d_edges(
            vertex_manager,
            edge_2d_q,
            transform_q,
            owned_by_q,
            local_shape_q,
            current_tab_file_entity,
            camera_3d_scale,
        );
        edge_manager.sync_3d_edges(
            edge_3d_q,
            transform_q,
            owned_by_q,
            visibility_q,
            local_shape_q,
            current_tab_file_entity,
            camera_3d_scale,
        );
        FaceManager::sync_2d_faces(
            face_2d_q,
            transform_q,
            owned_by_q,
            current_tab_file_entity,
            camera_3d_scale,
        );
        input_manager.sync_hover_shape_scale(transform_q, camera_3d_scale);
    }

    pub(crate) fn on_canvas_focus_changed(
        input_manager: &mut InputManager,
        vertex_manager: &mut VertexManager,
        edge_manager: &mut EdgeManager,
        new_focus: bool,
    ) {
        input_manager.queue_resync_selection_ui();
        if !new_focus {
            vertex_manager.last_vertex_dragged = None;
            edge_manager.last_edge_dragged = None;
            input_manager.hovered_entity = None;
        }
    }
}
