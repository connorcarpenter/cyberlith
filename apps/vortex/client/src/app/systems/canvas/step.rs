use bevy_ecs::system::{Query, ResMut};

use render_api::components::{Camera, Transform};
use vortex_proto::components::Vertex3d;

use crate::app::resources::{canvas::Canvas, canvas_manager::CanvasManager};

pub fn step(
    mut canvas: ResMut<Canvas>,
    mut canvas_manager: ResMut<CanvasManager>,
    mut camera_q: Query<(&mut Camera, &mut Transform)>,
    mut vertex_3d_q: Query<&mut Vertex3d>,
) {
    if canvas.update_visibility() {
        CanvasManager::update_visibility(canvas.is_visible(), &mut camera_q);
    }
    canvas_manager.update_3d_camera(&mut camera_q, &mut vertex_3d_q);
}
