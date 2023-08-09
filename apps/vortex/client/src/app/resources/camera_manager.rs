use bevy_ecs::{entity::Entity, system::{Query, Resource}};
use bevy_log::{info, warn};

use math::{EulerRot, Quat, Vec2, Vec3};
use render_api::components::{
    Camera, OrthographicProjection, Projection, RenderLayer, Transform, Viewport,
};

#[derive(Clone, Copy)]
pub enum CameraAngle {
    Side,
    Front,
    Top,
    Ingame(u8),
}

#[derive(Resource)]
pub struct CameraManager {
    is_2d: bool,
    pub camera_2d: Option<Entity>,
    pub layer_2d: RenderLayer,
    pub camera_3d: Option<Entity>,
    pub layer_3d: RenderLayer,
    camera_3d_recalc: bool,
    camera_3d_offset: Vec2,
    camera_3d_rotation: Vec2,
    camera_3d_scale: f32,
}

impl Default for CameraManager {
    fn default() -> Self {
        Self {
            is_2d: true,
            camera_2d: None,
            layer_2d: RenderLayer::default(),
            camera_3d: None,
            layer_3d: RenderLayer::default(),
            camera_3d_recalc: false,
            camera_3d_rotation: Vec2::ZERO,
            camera_3d_scale: 2.5,
            camera_3d_offset: Vec2::new(0.0, 100.0),
        }
    }
}

impl CameraManager {
    pub fn camera_3d_entity(&self) -> Option<Entity> {
        self.camera_3d
    }

    pub fn camera_3d_scale(&self) -> f32 {
        self.camera_3d_scale
    }

    pub fn camera_3d_offset(&self) -> Vec2 {
        self.camera_3d_offset
    }

    pub fn recalculate_3d_view(&mut self) {
        self.camera_3d_recalc = true;
    }

    pub fn update_3d_camera(
        &mut self,
        camera_q: &mut Query<(&mut Camera, &mut Transform)>,
    ) -> bool {
        if !self.camera_3d_recalc {
            return false;
        }

        let Some(camera_3d) = self.camera_3d else {
            return false;
        };

        let Ok((_, mut camera_transform)) = camera_q.get_mut(camera_3d) else {
            return false;
        };

        self.camera_3d_recalc = false;

        camera_transform.rotation = Quat::from_euler(
            EulerRot::YXZ,
            f32::to_radians(self.camera_3d_rotation.x),
            f32::to_radians(self.camera_3d_rotation.y),
            0.0,
        );
        camera_transform.scale = Vec3::splat(1.0 / self.camera_3d_scale);

        let right = camera_transform.right_direction();
        let up = right.cross(camera_transform.view_direction());

        camera_transform.translation = camera_transform.view_direction() * -100.0; // 100 units away from where looking
        let rounded_offset = self.camera_3d_offset.round();
        camera_transform.translation += right * rounded_offset.x;
        camera_transform.translation += up * rounded_offset.y;

        return true;
    }

    pub fn set_2d_mode(&mut self, camera_query: &mut Query<(&mut Camera, &mut Projection)>) {
        if self.is_2d {
            return;
        }
        info!("Switched to Wireframe mode");
        self.is_2d = true;
        self.enable_cameras(camera_query, true);
    }

    pub fn set_3d_mode(&mut self, camera_query: &mut Query<(&mut Camera, &mut Projection)>) {
        if !self.is_2d {
            return;
        }
        info!("Switched to Solid mode");
        self.is_2d = false;
        self.enable_cameras(camera_query, false);
    }

    pub fn set_camera_angle_ingame(&mut self, game_index: u8) {
        let angle = match game_index {
            1 => 30.0,  // seems to be 2:1 diablo isometric angle ?
            2 => 63.43, // 90 - arctan(1/2)
            3 => 69.91,
            4 => 76.39, // seems to be 4:3 warcraft angle ?
            5 => 82.87, // 90 - arctan(1/8)
            _ => {
                warn!("Invalid game index: {}", game_index);
                return;
            }
        };

        let mut rotation = self.camera_3d_rotation;
        rotation.y = angle * -1.0;
        self.set_camera_angle(rotation);
    }

    pub fn set_camera_angle_yaw_rotate(&mut self, counter: bool) {
        let mut rotation = (self.camera_3d_rotation.x / 45.0).round() * 45.0;
        match counter {
            true => {
                rotation += 45.0;
                if rotation > 360.0 {
                    rotation -= 360.0;
                }
            }
            false => {
                rotation -= 45.0;
                if rotation < 0.0 {
                    rotation += 360.0;
                }
            }
        }

        self.set_camera_angle(Vec2::new(rotation, self.camera_3d_rotation.y));
    }

    pub fn set_camera_angle_side(&mut self) {
        self.set_camera_angle(Vec2::new(-90.0, 0.0));
    }

    pub fn set_camera_angle_front(&mut self) {
        self.set_camera_angle(Vec2::new(0.0, 0.0));
    }

    pub fn set_camera_angle_top(&mut self) {
        self.set_camera_angle(Vec2::new(0.0, -90.0));
    }

    pub fn camera_pan(&mut self, delta: Vec2) {
        self.camera_3d_offset += delta / self.camera_3d_scale;

        self.recalculate_3d_view();
    }

    pub fn camera_orbit(&mut self, delta: Vec2) {
        self.camera_3d_rotation.x += delta.x * -0.5;
        if self.camera_3d_rotation.x > 360.0 {
            self.camera_3d_rotation.x -= 360.0;
        } else if self.camera_3d_rotation.x < 0.0 {
            self.camera_3d_rotation.x += 360.0;
        }

        self.camera_3d_rotation.y += delta.y * -0.5;
        if self.camera_3d_rotation.y > 0.0 {
            self.camera_3d_rotation.y = 0.0;
        } else if self.camera_3d_rotation.y < -90.0 {
            self.camera_3d_rotation.y = -90.0;
        }

        self.recalculate_3d_view();
    }

    pub fn camera_zoom(&mut self, zoom_delta: f32) {
        let old_scale = self.camera_3d_scale;
        let new_scale = (old_scale + (zoom_delta * 0.01)).min(8.0).max(1.0);
        let scale_diff = new_scale - old_scale;
        self.camera_3d_scale = new_scale;

        if scale_diff.abs() > 0.0 {
            let old_screen_offset = self.camera_3d_offset * old_scale;
            let new_screen_offset = self.camera_3d_offset * new_scale;

            let offset_diff = new_screen_offset - old_screen_offset;

            self.camera_3d_offset -= offset_diff / new_scale;
        }

        self.recalculate_3d_view();
    }

    fn set_camera_angle(&mut self, angle: Vec2) {
        self.camera_3d_rotation = angle;

        self.recalculate_3d_view();
    }

    fn enable_cameras(
        &self,
        camera_query: &mut Query<(&mut Camera, &mut Projection)>,
        enable_2d: bool,
    ) {
        let enable_3d = !enable_2d;

        if let Some(camera_2d) = self.camera_2d {
            if let Ok((mut camera, _)) = camera_query.get_mut(camera_2d) {
                camera.is_active = enable_2d;
            };
        }
        if let Some(camera_3d) = self.camera_3d {
            if let Ok((mut camera, _)) = camera_query.get_mut(camera_3d) {
                camera.is_active = enable_3d;
            };
        }
    }

    pub fn update_camera_viewports(
        &mut self,
        texture_size: Vec2,
        camera_query: &mut Query<(&mut Camera, &mut Transform, &mut Projection)>,
    ) {
        self.update_2d_camera_viewport(texture_size, camera_query);
        self.update_3d_camera_viewport(texture_size, camera_query);
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

        *transform = Transform::from_xyz(center.x, center.y, 1.0)
            .looking_at(Vec3::new(center.x, center.y, 0.0), Vec3::NEG_Y);
        *projection =
            Projection::Orthographic(OrthographicProjection::new(texture_size.y, 0.0, 10.0));
    }

    fn update_3d_camera_viewport(
        &self,
        texture_size: Vec2,
        camera_query: &mut Query<(&mut Camera, &mut Transform, &mut Projection)>,
    ) {
        let Some(camera_entity) = self.camera_3d else {
            return;
        };
        let Ok((mut camera, _, mut projection)) = camera_query.get_mut(camera_entity) else {
            return;
        };

        camera.viewport = Some(Viewport::new_at_origin(
            texture_size.x as u32,
            texture_size.y as u32,
        ));

        *projection =
            Projection::Orthographic(OrthographicProjection::new(texture_size.y, 0.0, 1000.0));
    }

    pub fn update_visibility(visible: bool, camera_q: &mut Query<(&mut Camera, &mut Transform)>) {
        let cameras_enabled = visible;

        if cameras_enabled {
            info!("Camera are ENABLED");
        } else {
            info!("Camera are DISABLED");
        }

        for (mut camera, _) in camera_q.iter_mut() {
            camera.is_active = cameras_enabled;
        }
    }
}
