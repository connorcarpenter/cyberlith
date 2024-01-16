use bevy_log::info;
use math::Vec2;

#[derive(Clone, Copy)]
pub struct CameraState {
    is_2d: bool,
    camera_3d_offset: Vec2,
    camera_3d_rotation: Vec2,
    camera_3d_scale: f32,
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            is_2d: true,
            camera_3d_offset: Vec2::new(0.0, 0.0),
            // this faces the camera towards the front of the model
            camera_3d_rotation: Vec2::new(180.0, 0.0),
            camera_3d_scale: 1.0,
        }
    }
}

impl CameraState {
    pub fn is_2d(&self) -> bool {
        self.is_2d
    }

    fn set_is_2d(&mut self, is_2d: bool) {
        self.is_2d = is_2d;
    }

    pub fn camera_3d_offset(&self) -> Vec2 {
        self.camera_3d_offset
    }

    pub fn set_camera_3d_offset(&mut self, offset: Vec2) {
        self.camera_3d_offset = offset;
    }

    pub fn camera_3d_rotation(&self) -> Vec2 {
        self.camera_3d_rotation
    }

    pub fn set_camera_3d_rotation(&mut self, rotation: Vec2) {
        self.camera_3d_rotation = rotation;
    }

    pub fn camera_3d_scale(&self) -> f32 {
        self.camera_3d_scale
    }

    pub fn set_camera_3d_scale(&mut self, scale: f32) {
        self.camera_3d_scale = scale;
    }

    pub fn set_2d_mode(&mut self) {
        if self.is_2d() {
            return;
        }
        info!("Switched to Wireframe mode");
        self.set_is_2d(true);
    }

    pub fn set_3d_mode(&mut self) {
        if !self.is_2d() {
            return;
        }
        info!("Switched to Solid mode");
        self.set_is_2d(false);
    }
}
