use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::Resource;
use bevy_ecs::system::Query;
use bevy_log::info;

use render_api::components::{Camera, RenderLayer};

#[derive(Resource)]
pub struct CanvasState {
    is_visible: bool,
    is_2d: bool,
    pub camera_2d: Option<Entity>,
    pub camera_3d: Option<Entity>,
    pub layer_2d: RenderLayer,
    pub layer_3d: RenderLayer,
}

impl Default for CanvasState {
    fn default() -> Self {
        Self {
            is_visible: false,
            is_2d: true,
            camera_2d: None,
            camera_3d: None,
            layer_2d: RenderLayer::default(),
            layer_3d: RenderLayer::default(),
        }
    }
}

impl CanvasState {
    pub fn update_cameras(&self, camera_q: &mut Query<&mut Camera>) {
        let cameras_enabled = self.is_visible;

        if cameras_enabled {
            info!("Camera are ENABLED");
        } else {
            info!("Camera are DISABLED");
        }

        for mut camera in camera_q.iter_mut() {
            camera.is_active = cameras_enabled;
        }
    }
}