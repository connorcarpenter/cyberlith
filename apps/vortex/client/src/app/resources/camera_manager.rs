use bevy_ecs::{
    entity::Entity,
    system::{Query, Resource},
};
use bevy_log::warn;

use math::{EulerRot, Quat, Vec2, Vec3};
use render_api::components::{
    Camera, OrthographicProjection, Projection, RenderLayer, Transform, Viewport,
};

use crate::app::resources::camera_state::CameraState;

#[derive(Clone, Copy)]
pub enum CameraAngle {
    Side,
    Front,
    Top,
    Ingame(u8),
}

#[derive(Resource)]
pub struct CameraManager {
    pub camera_2d: Option<Entity>,
    pub layer_2d: RenderLayer,
    pub camera_3d: Option<Entity>,
    pub layer_3d: RenderLayer,
    camera_3d_recalc: bool,
}

impl Default for CameraManager {
    fn default() -> Self {
        Self {
            camera_2d: None,
            layer_2d: RenderLayer::default(),
            camera_3d: None,
            layer_3d: RenderLayer::default(),
            camera_3d_recalc: false,
        }
    }
}

impl CameraManager {
    pub const MIN_SCALE: f32 = 1.0;
    pub const MAX_SCALE: f32 = 8.0;

    pub fn camera_3d_entity(&self) -> Option<Entity> {
        self.camera_3d
    }

    pub fn recalculate_3d_view(&mut self) {
        self.camera_3d_recalc = true;
    }

    pub fn update_3d_camera(
        &mut self,
        camera_state: &CameraState,
        camera_q: &mut Query<(&mut Camera, &mut Projection, &mut Transform)>,
    ) -> bool {
        if !self.camera_3d_recalc {
            return false;
        }

        let Some(camera_3d) = self.camera_3d else {
            return false;
        };

        let camera_3d_rotation = camera_state.camera_3d_rotation();
        let camera_3d_scale = camera_state.camera_3d_scale();
        let camera_3d_offset = camera_state.camera_3d_offset();

        self.enable_cameras(camera_q, camera_state.is_2d());

        let Ok((_, _, mut camera_transform)) = camera_q.get_mut(camera_3d) else {
            return false;
        };

        self.camera_3d_recalc = false;

        set_camera_transform(
            &mut camera_transform,
            camera_3d_rotation,
            camera_3d_scale,
            camera_3d_offset,
        );

        return true;
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

        *projection = Projection::Orthographic(OrthographicProjection::new(0.0, 1000.0));
    }

    pub fn set_camera_angle_ingame(&mut self, camera_state: &mut CameraState, game_index: u8) {
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

        let mut rotation = camera_state.camera_3d_rotation();
        rotation.y = angle * -1.0;
        self.set_camera_angle(camera_state, rotation);
    }

    pub fn set_camera_angle_yaw_rotate(&mut self, camera_state: &mut CameraState, counter: bool) {
        let camera_3d_rotation = camera_state.camera_3d_rotation();
        let mut rotation = (camera_3d_rotation.x / 45.0).round() * 45.0;
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

        self.set_camera_angle(camera_state, Vec2::new(rotation, camera_3d_rotation.y));
    }

    pub fn set_camera_angle_side(&mut self, camera_state: &mut CameraState) {
        self.set_camera_angle(camera_state, Vec2::new(-90.0, 0.0));
    }

    pub fn set_camera_angle_front(&mut self, camera_state: &mut CameraState) {
        self.set_camera_angle(camera_state, Vec2::new(180.0, 0.0));
    }

    pub fn set_camera_angle_top(&mut self, camera_state: &mut CameraState) {
        self.set_camera_angle(camera_state, Vec2::new(180.0, 90.0));
    }

    pub fn camera_pan(&mut self, camera_state: &mut CameraState, delta: Vec2) {
        let mut camera_3d_offset = camera_state.camera_3d_offset();
        camera_3d_offset += delta / camera_state.camera_3d_scale();
        camera_state.set_camera_3d_offset(camera_3d_offset);

        self.recalculate_3d_view();
    }

    pub fn camera_orbit(&mut self, camera_state: &mut CameraState, delta: Vec2) {
        let mut camera_3d_rotation = camera_state.camera_3d_rotation();

        camera_3d_rotation.x += delta.x;
        if camera_3d_rotation.x > 360.0 {
            camera_3d_rotation.x -= 360.0;
        } else if camera_3d_rotation.x < 0.0 {
            camera_3d_rotation.x += 360.0;
        }

        camera_3d_rotation.y += delta.y;
        if camera_3d_rotation.y < 0.0 {
            camera_3d_rotation.y = 0.0;
        } else if camera_3d_rotation.y > 90.0 {
            camera_3d_rotation.y = 90.0;
        }

        camera_state.set_camera_3d_rotation(camera_3d_rotation);

        self.recalculate_3d_view();
    }

    pub fn camera_zoom(&mut self, camera_state: &mut CameraState, zoom_delta: f32) {
        let old_scale = camera_state.camera_3d_scale();
        let new_scale = (old_scale + (zoom_delta * 0.01))
            .min(Self::MAX_SCALE)
            .max(Self::MIN_SCALE);
        let scale_diff = new_scale - old_scale;
        camera_state.set_camera_3d_scale(new_scale);

        let mut camera_3d_offset = camera_state.camera_3d_offset();

        if scale_diff.abs() > 0.0 {
            let old_screen_offset = camera_3d_offset * old_scale;
            let new_screen_offset = camera_3d_offset * new_scale;

            let offset_diff = new_screen_offset - old_screen_offset;

            camera_3d_offset -= offset_diff / new_scale;
            camera_state.set_camera_3d_offset(camera_3d_offset);
        }

        self.recalculate_3d_view();
    }

    fn set_camera_angle(&mut self, camera_state: &mut CameraState, angle: Vec2) {
        camera_state.set_camera_3d_rotation(angle);

        self.recalculate_3d_view();
    }

    pub fn enable_cameras(
        &self,
        camera_q: &mut Query<(&mut Camera, &mut Projection, &mut Transform)>,
        enable_2d: bool,
    ) {
        let enable_3d = !enable_2d;

        if let Some(camera_2d) = self.camera_2d {
            if let Ok((mut camera, _, _)) = camera_q.get_mut(camera_2d) {
                camera.is_active = enable_2d;
            };
        }
        if let Some(camera_3d) = self.camera_3d {
            if let Ok((mut camera, _, _)) = camera_q.get_mut(camera_3d) {
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

        *transform = Transform::from_xyz(center.x, center.y, 1000.0)
            .looking_at(Vec3::new(center.x, center.y, 0.0), Vec3::NEG_Y);
        *projection = Projection::Orthographic(OrthographicProjection::new(0.0, 2000.0));
    }

    pub fn update_visibility(
        visible: bool,
        camera_q: &mut Query<(&mut Camera, &mut Projection, &mut Transform)>,
    ) {
        let cameras_enabled = visible;

        // if cameras_enabled {
        //     info!("Camera are ENABLED");
        // } else {
        //     info!("Camera are DISABLED");
        // }

        for (mut camera, _, _) in camera_q.iter_mut() {
            camera.is_active = cameras_enabled;
        }
    }
}

pub fn set_camera_transform(
    camera_transform: &mut Transform,
    camera_3d_rotation: Vec2,
    camera_3d_scale: f32,
    camera_3d_offset: Vec2,
) {
    // Rotation

    camera_transform.look_to(Vec3::X, Vec3::Z);

    let rotate_by = Quat::from_euler(
        EulerRot::ZYX,
        f32::to_radians(camera_3d_rotation.x),
        f32::to_radians(camera_3d_rotation.y),
        0.0,
    );
    camera_transform.rotation = rotate_by * camera_transform.rotation;

    // Scale

    camera_transform.scale = Vec3::splat(1.0 / camera_3d_scale);

    // Translation

    //info!("rotation: {:?}, offset: {:?}", camera_3d_rotation, camera_3d_offset);

    let view_right_dir = camera_transform.view_right();
    let view_down_dir = camera_transform.view_down();
    let view_forward_dir = camera_transform.view_forward();

    //info!("view_right_dir: {:?}, view_up_dir: {:?}", view_right_dir, view_down_dir);

    let rounded_offset = camera_3d_offset.round();

    // we subtract here because we want to move the camera in the opposite direction
    camera_transform.translation = Vec3::ZERO;
    camera_transform.translation -= view_forward_dir * 100.0; // 100.0 away from camera target
    camera_transform.translation -= view_right_dir * rounded_offset.x;
    camera_transform.translation -= view_down_dir * rounded_offset.y;
}
