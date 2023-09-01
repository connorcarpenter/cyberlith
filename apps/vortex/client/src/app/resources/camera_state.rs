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
            camera_3d_offset: Vec2::new(0.0, 100.0),
            camera_3d_rotation: Vec2::ZERO,
            camera_3d_scale: 2.5,
        }
    }
}

impl CameraState {
    pub fn is_2d(&self) -> bool {
        self.is_2d
    }

    pub fn set_is_2d(&mut self, is_2d: bool) {
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
}
