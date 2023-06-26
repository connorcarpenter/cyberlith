use bevy_ecs::{entity::Entity, prelude::Resource, system::Query};
use bevy_log::info;

use math::{Vec2, Vec3};
use render_api::{base::CpuTexture2D, components::{Camera, OrthographicProjection, Projection, RenderLayer, Transform, Viewport}, Handle};

#[derive(Resource)]
pub struct CanvasState {
    is_visible: bool,
    is_2d: bool,
    canvas_texture: Option<Handle<CpuTexture2D>>,
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
            canvas_texture: None,
            camera_2d: None,
            camera_3d: None,
            layer_2d: RenderLayer::default(),
            layer_3d: RenderLayer::default(),
        }
    }
}

impl CanvasState {
    pub fn update_all_camera_visibility(&self, camera_q: &mut Query<&mut Camera>) {
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

    pub fn update_camera_viewports(
        &self,
        texture_size: Vec2,
        camera_query: &mut Query<(&mut Camera, &mut Transform, &mut Projection)>,
    ) {
        self.update_2d_camera_viewport(texture_size, camera_query);
        self.update_3d_camera_viewport(texture_size, camera_query);
    }

    pub fn canvas_texture(&self) -> Handle<CpuTexture2D> {
        self.canvas_texture.unwrap()
    }

    pub fn set_canvas_texture(&mut self, texture: Handle<CpuTexture2D>) {
        self.canvas_texture = Some(texture);
    }

    fn update_2d_camera_viewport(
        &self,
        texture_size: Vec2,
        camera_query: &mut Query<(&mut Camera, &mut Transform, &mut Projection)>,
    ) {
        let Some(camera_entity) = self.camera_2d else {
            return;
        };
        let Ok((mut camera, mut transform, mut projection)) = camera_query.get_mut(camera_entity) else {
            return;
        };
        camera.viewport = Some(Viewport::new_at_origin(
            texture_size.x as u32,
            texture_size.y as u32,
        ));

        let center = texture_size * 0.5;

        *transform = Transform::from_xyz(center.x, center.y, -1.0)
            .looking_at(Vec3::new(center.x, center.y, 0.0), Vec3::NEG_Y);
        *projection = Projection::Orthographic(OrthographicProjection {
            height: texture_size.y,
            near: 0.0,
            far: 10.0,
        });
    }

    fn update_3d_camera_viewport(
        &self,
        texture_size: Vec2,
        camera_query: &mut Query<(&mut Camera, &mut Transform, &mut Projection)>,
    ) {
        let Some(camera_entity) = self.camera_3d else {
            return;
        };
        let Ok((mut camera, _, _)) = camera_query.get_mut(camera_entity) else {
            return;
        };
        camera.viewport = Some(Viewport::new_at_origin(
            texture_size.x as u32,
            texture_size.y as u32,
        ));
    }
}