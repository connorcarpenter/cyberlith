use bevy_log::info;
use math::*;

use super::*;

///
/// Defines the part of the screen/render target that is rendered to.
/// All values should be given in physical pixels.
///
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Viewport {
    /// The distance in pixels from the left edge of the screen/render target.
    pub x: i32,
    /// The distance in pixels from the bottom edge of the screen/render target.
    pub y: i32,
    /// The width of the viewport.
    pub width: u32,
    /// The height of the viewport.
    pub height: u32,
}

impl Viewport {
    ///
    /// New viewport which starts at origin (x and y are both zero).
    ///
    pub fn new_at_origin(width: u32, height: u32) -> Self {
        Self {
            x: 0,
            y: 0,
            width,
            height,
        }
    }

    ///
    /// Returns the aspect ratio of this viewport.
    ///
    pub fn aspect(&self) -> f32 {
        self.width as f32 / self.height as f32
    }

    ///
    /// Returns the intersection between this and the other Viewport.
    ///
    pub fn intersection(&self, other: impl Into<Self>) -> Self {
        let other = other.into();
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let width =
            (self.x + self.width as i32 - x).clamp(0, other.x + other.width as i32 - x) as u32;
        let height =
            (self.y + self.height as i32 - y).clamp(0, other.y + other.height as i32 - y) as u32;
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

///
/// The type of projection used by a camera (orthographic or perspective) including parameters.
///
#[derive(Clone, Debug)]
pub enum ProjectionType {
    /// Orthographic projection
    Orthographic {
        /// Height of the camera film/sensor.
        height: f32,
    },
    /// Perspective projection
    Perspective {
        /// The field of view angle in the vertical direction.
        fov_y_radians: f32,
    },
}

///
/// Represents a camera used for viewing 3D assets.
///
#[derive(Clone, Debug)]
pub struct Camera {
    viewport: Viewport,
    projection_type: ProjectionType,
    z_near: f32,
    z_far: f32,
    position: Vec3,
    target: Vec3,
    up: Vec3,
    view: Mat4,
    projection: Mat4,
    screen2ray: Mat4,
    frustrum: [Vec4; 6],
}

impl Camera {
    ///
    /// New camera which projects the world with an orthographic projection.
    /// See also [set_view](Self::set_view), [set_perspective_projection](Self::set_perspective_projection) and
    /// [set_orthographic_projection](Self::set_orthographic_projection).
    ///
    pub fn new_orthographic(
        viewport: Viewport,
        position: Vec3,
        target: Vec3,
        up: Vec3,
        height: f32,
        z_near: f32,
        z_far: f32,
    ) -> Self {
        let mut camera = Camera::new(viewport);
        camera.set_view(position, target, up);
        camera.set_orthographic_projection(height, z_near, z_far);
        camera
    }

    ///
    /// New camera which projects the world with a perspective projection.
    ///
    pub fn new_perspective(
        viewport: Viewport,
        position: Vec3,
        target: Vec3,
        up: Vec3,
        fov_y_degrees: f32,
        z_near: f32,
        z_far: f32,
    ) -> Self {
        let mut camera = Camera::new(viewport);
        camera.set_view(position, target, up);
        camera.set_perspective_projection(fov_y_degrees, z_near, z_far);

        info!("Camera View: {}", camera.view);
        info!("Camera Projection: {}", camera.projection);

        camera
    }

    ///
    /// Specify the camera to use perspective projection with the given field of view in the y-direction and near and far plane.
    ///
    pub fn set_perspective_projection(
        &mut self,
        fov_y_degrees: f32,
        z_near: f32,
        z_far: f32,
    ) {
        assert!(
            z_near >= 0.0 || z_near < z_far,
            "Wrong perspective camera parameters"
        );
        self.z_near = z_near;
        self.z_far = z_far;
        let fov_y_radians = f32::to_radians(fov_y_degrees);
        self.projection_type = ProjectionType::Perspective { fov_y_radians };
        self.projection = Self::perspective_to_mat4(fov_y_radians, self.viewport.aspect(), self.z_near, self.z_far);
        self.update_screen2ray();
        self.update_frustrum();
    }

    fn perspective_to_mat4(fov_y_radians: f32, aspect: f32, z_near: f32, z_far: f32) -> Mat4 {

        let a = f32::tan(fov_y_radians / 2.0).recip();
        let b = a / aspect;
        let c = (z_far + z_near) / (z_near - z_far);
        let d = (2.0 * z_far * z_near) / (z_near - z_far);

        Mat4::from_cols(
            Vec4::new(b, 0.0, 0.0, 0.0),
            Vec4::new(0.0, a, 0.0, 0.0),
            Vec4::new(0.0, 0.0, c, -1.0),
            Vec4::new(0.0, 0.0, d, 0.0),
        )
    }

    ///
    /// Specify the camera to use orthographic projection with the given height and depth.
    /// The view frustum height is `+/- height/2`.
    /// The view frustum width is calculated as `height * viewport.width / viewport.height`.
    /// The view frustum depth is `z_near` to `z_far`.
    ///
    pub fn set_orthographic_projection(&mut self, height: f32, z_near: f32, z_far: f32) {
        assert!(z_near < z_far, "Wrong orthographic camera parameters");
        self.z_near = z_near;
        self.z_far = z_far;
        let width = height * self.viewport.aspect();
        self.projection_type = ProjectionType::Orthographic { height };
        self.projection = Mat4::orthographic_rh(
            -0.5 * width,
            0.5 * width,
            -0.5 * height,
            0.5 * height,
            z_near,
            z_far,
        );
        self.update_screen2ray();
        self.update_frustrum();
    }

    ///
    /// Set the current viewport.
    /// Returns whether or not the viewport actually changed.
    ///
    pub fn set_viewport(&mut self, viewport: Viewport) -> bool {
        if self.viewport != viewport {
            self.viewport = viewport;
            match self.projection_type {
                ProjectionType::Orthographic { height } => {
                    self.set_orthographic_projection(height, self.z_near, self.z_far);
                }
                ProjectionType::Perspective { fov_y_radians: field_of_view_y } => {
                    self.set_perspective_projection(field_of_view_y, self.z_near, self.z_far);
                }
            }
            true
        } else {
            false
        }
    }

    ///
    /// Change the view of the camera.
    /// The camera is placed at the given position, looking at the given target and with the given up direction.
    ///
    pub fn set_view(&mut self, position: Vec3, target: Vec3, up: Vec3) {
        self.position = position;
        self.target = target;
        self.up = up;
        self.view = Mat4::look_at_rh(
            self.position,
            self.target,
            self.up,
        );
        self.update_screen2ray();
        self.update_frustrum();
    }

    ///
    /// Change the camera view such that it is mirrored in the xz-plane.
    ///
    pub fn mirror_in_xz_plane(&mut self) {
        // TODO: verify that this is correct, I have not tested it
        self.view = self.view * Mat4::from_scale(Vec3::new(1.0, -1.0, 1.0));
        self.update_screen2ray();
        self.update_frustrum();
    }

    ///
    /// Returns whether or not the given bounding box is within the camera frustum.
    /// It returns false if it is fully outside and true if it is inside or intersects.
    ///
    pub fn in_frustum(&self, aabb: &AxisAlignedBoundingBox) -> bool {
        if aabb.is_infinite() {
            return true;
        }
        // check box outside/inside of frustum
        for i in 0..6 {
            let mut out = 0;
            if self.frustrum[i].dot(Vec4::new(aabb.min().x, aabb.min().y, aabb.min().z, 1.0)) < 0.0 {
                out += 1
            };
            if self.frustrum[i].dot(Vec4::new(aabb.max().x, aabb.min().y, aabb.min().z, 1.0)) < 0.0 {
                out += 1
            };
            if self.frustrum[i].dot(Vec4::new(aabb.min().x, aabb.max().y, aabb.min().z, 1.0)) < 0.0 {
                out += 1
            };
            if self.frustrum[i].dot(Vec4::new(aabb.max().x, aabb.max().y, aabb.min().z, 1.0)) < 0.0 {
                out += 1
            };
            if self.frustrum[i].dot(Vec4::new(aabb.min().x, aabb.min().y, aabb.max().z, 1.0)) < 0.0 {
                out += 1
            };
            if self.frustrum[i].dot(Vec4::new(aabb.max().x, aabb.min().y, aabb.max().z, 1.0)) < 0.0 {
                out += 1
            };
            if self.frustrum[i].dot(Vec4::new(aabb.min().x, aabb.max().y, aabb.max().z, 1.0)) < 0.0 {
                out += 1
            };
            if self.frustrum[i].dot(Vec4::new(aabb.max().x, aabb.max().y, aabb.max().z, 1.0)) < 0.0 {
                out += 1
            };
            if out == 8 {
                return false;
            }
        }
        // TODO: Test the frustum corners against the box planes (http://www.iquilezles.org/www/articles/frustumcorrect/frustumcorrect.htm)

        true
    }

    ///
    /// Returns the 3D position at the given pixel coordinate.
    /// The pixel coordinate must be in physical pixels, where `(viewport.x, viewport.y)` indicate the bottom left corner of the viewport
    /// and `(viewport.x + viewport.width, viewport.y + viewport.height)` indicate the top right corner.
    ///
    pub fn position_at_pixel(&self, pixel: (f32, f32)) -> Vec3 {
        match self.projection_type() {
            ProjectionType::Orthographic { .. } => {
                let coords = self.uv_coordinates_at_pixel(pixel);
                self.position_at_uv_coordinates(coords)
            }
            ProjectionType::Perspective { .. } => *self.position(),
        }
    }

    ///
    /// Returns the 3D position at the given uv coordinate of the viewport.
    /// The uv coordinate must be between `(0, 0)` indicating the bottom left corner of the viewport
    /// and `(1, 1)` indicating the top right corner.
    ///
    pub fn position_at_uv_coordinates(&self, coords: (f32, f32)) -> Vec3 {
        match self.projection_type() {
            ProjectionType::Orthographic { height } => {
                let width = height * self.viewport.aspect();
                *self.position() + Vec3::new((coords.0 - 0.5) * width, (coords.1 - 0.5) * height, 0.0)
            }
            ProjectionType::Perspective { .. } => *self.position(),
        }
    }

    ///
    /// Returns the 3D view direction at the given pixel coordinate.
    /// The pixel coordinate must be in physical pixels, where `(viewport.x, viewport.y)` indicate the bottom left corner of the viewport
    /// and `(viewport.x + viewport.width, viewport.y + viewport.height)` indicate the top right corner.
    ///
    pub fn view_direction_at_pixel(&self, pixel: (f32, f32)) -> Vec3 {
        match self.projection_type() {
            ProjectionType::Orthographic { .. } => self.view_direction(),
            ProjectionType::Perspective { .. } => {
                let coords = self.uv_coordinates_at_pixel(pixel);
                self.view_direction_at_uv_coordinates(coords)
            }
        }
    }

    ///
    /// Returns the 3D view direction at the given uv coordinate of the viewport.
    /// The uv coordinate must be between `(0, 0)` indicating the bottom left corner of the viewport
    /// and `(1, 1)` indicating the top right corner.
    ///
    pub fn view_direction_at_uv_coordinates(&self, coords: (f32, f32)) -> Vec3 {
        match self.projection_type() {
            ProjectionType::Orthographic { .. } => self.view_direction(),
            ProjectionType::Perspective { .. } => {
                let screen_pos = Vec4::new(2. * coords.0 - 1., 2. * coords.1 - 1.0, 0., 1.);
                (self.screen2ray * screen_pos).truncate().normalize()
            }
        }
    }

    ///
    /// Returns the uv coordinate for the given pixel coordinate.
    /// The pixel coordinate must be in physical pixels, where `(viewport.x, viewport.y)` indicate the bottom left corner of the viewport
    /// and `(viewport.x + viewport.width, viewport.y + viewport.height)` indicate the top right corner.
    /// The returned uv coordinate is between 0 and 1 where `(0,0)` indicate the bottom left corner of the viewport and `(1,1)` indicate the top right corner.
    ///
    pub fn uv_coordinates_at_pixel(&self, pixel: (f32, f32)) -> (f32, f32) {
        (
            (pixel.0 - self.viewport.x as f32) / self.viewport.width as f32,
            (pixel.1 - self.viewport.y as f32) / self.viewport.height as f32,
        )
    }

    ///
    /// Returns the uv coordinate for the given world position.
    /// The returned uv coordinate are between 0 and 1 where `(0,0)` indicate a position that maps to the bottom left corner of the viewport
    /// and (1,1) indicate a position that maps to the top right corner.
    ///
    pub fn uv_coordinates_at_position(&self, position: Vec3) -> (f32, f32) {
        let proj = *self.projection() * *self.view() * position.extend(1.0);
        (
            0.5 * (proj.x / proj.w.abs() + 1.0),
            0.5 * (proj.y / proj.w.abs() + 1.0),
        )
    }

    ///
    /// Returns the pixel coordinate for the given uv coordinate.
    /// The uv coordinate must be between 0 and 1 where `(0,0)` indicate the bottom left corner of the viewport
    /// and (1,1) indicate the top right corner.
    /// The returned pixel coordinate is in physical pixels, where `(viewport.x, viewport.y)` indicate the bottom left corner of the viewport
    /// and `(viewport.x + viewport.width, viewport.y + viewport.height)` indicate the top right corner.
    ///
    pub fn pixel_at_uv_coordinates(&self, coords: (f32, f32)) -> (f32, f32) {
        (
            coords.0 * self.viewport.width as f32 + self.viewport.x as f32,
            coords.1 * self.viewport.height as f32 + self.viewport.y as f32,
        )
    }

    ///
    /// Returns the pixel coordinate for the given world position.
    /// The returned pixel coordinate is in physical pixels, where `(viewport.x, viewport.y)` indicate the bottom left corner of the viewport
    /// and `(viewport.x + viewport.width, viewport.y + viewport.height)` indicate the top right corner.
    ///
    pub fn pixel_at_position(&self, position: Vec3) -> (f32, f32) {
        self.pixel_at_uv_coordinates(self.uv_coordinates_at_position(position))
    }

    ///
    /// Returns the type of projection (orthographic or perspective) including parameters.
    ///
    pub fn projection_type(&self) -> &ProjectionType {
        &self.projection_type
    }

    ///
    /// Returns the view matrix, ie. the matrix that transforms objects from world space (as placed in the world) to view space (as seen from this camera).
    ///
    pub fn view(&self) -> &Mat4 {
        &self.view
    }

    ///
    /// Returns the projection matrix, ie. the matrix that projects objects in view space onto this cameras image plane.
    ///
    pub fn projection(&self) -> &Mat4 {
        &self.projection
    }

    ///
    /// Returns the viewport.
    ///
    pub fn viewport(&self) -> Viewport {
        self.viewport
    }

    ///
    /// Returns the distance to the near plane of the camera frustum.
    ///
    pub fn z_near(&self) -> f32 {
        self.z_near
    }

    ///
    /// Returns the distance to the far plane of the camera frustum.
    ///
    pub fn z_far(&self) -> f32 {
        self.z_far
    }

    ///
    /// Returns the position of this camera.
    ///
    pub fn position(&self) -> &Vec3 {
        &self.position
    }

    ///
    /// Returns the target of this camera, ie the point that this camera looks towards.
    ///
    pub fn target(&self) -> &Vec3 {
        &self.target
    }

    ///
    /// Returns the up direction of this camera (might not be orthogonal to the view direction).
    ///
    pub fn up(&self) -> &Vec3 {
        &self.up
    }

    ///
    /// Returns the view direction of this camera, ie. the direction the camera is looking.
    ///
    pub fn view_direction(&self) -> Vec3 {
        (self.target - self.position).normalize()
    }

    ///
    /// Returns the right direction of this camera.
    ///
    pub fn right_direction(&self) -> Vec3 {
        self.view_direction().cross(self.up)
    }

    fn new(viewport: Viewport) -> Camera {
        Camera {
            viewport,
            projection_type: ProjectionType::Orthographic { height: 1.0 },
            z_near: 0.0,
            z_far: 0.0,
            frustrum: [Vec4::new(0.0, 0.0, 0.0, 0.0); 6],
            position: Vec3::new(0.0, 0.0, 5.0),
            target: Vec3::new(0.0, 0.0, 0.0),
            up: Vec3::new(0.0, 1.0, 0.0),
            view: Mat4::IDENTITY,
            projection: Mat4::IDENTITY,
            screen2ray: Mat4::IDENTITY,
        }
    }

    fn update_screen2ray(&mut self) {
        let mut w_column = self.view.col_mut(3);
        *w_column = Vec4::new(0.0, 0.0, 0.0, 1.0);
        self.screen2ray = (self.projection * self.view).inverse();
    }

    fn update_frustrum(&mut self) {
        let m = self.projection * self.view;
        self.frustrum = [
            Vec4::new(m.x_axis.w + m.x_axis.x, m.y_axis.w + m.y_axis.x, m.z_axis.w + m.z_axis.x, m.w_axis.w + m.w_axis.x),
            Vec4::new(m.x_axis.w - m.x_axis.x, m.y_axis.w - m.y_axis.x, m.z_axis.w - m.z_axis.x, m.w_axis.w - m.w_axis.x),
            Vec4::new(m.x_axis.w + m.x_axis.y, m.y_axis.w + m.y_axis.y, m.z_axis.w + m.z_axis.y, m.w_axis.w + m.w_axis.y),
            Vec4::new(m.x_axis.w - m.x_axis.y, m.y_axis.w - m.y_axis.y, m.z_axis.w - m.z_axis.y, m.w_axis.w - m.w_axis.y),
            Vec4::new(m.x_axis.w + m.x_axis.z, m.y_axis.w + m.y_axis.z, m.z_axis.w + m.z_axis.z, m.w_axis.w + m.w_axis.z),
            Vec4::new(m.x_axis.w - m.x_axis.z, m.y_axis.w - m.y_axis.z, m.z_axis.w - m.z_axis.z, m.w_axis.w - m.w_axis.z),
        ];
    }

    ///
    /// Translate the camera by the given change while keeping the same view and up directions.
    ///
    pub fn translate(&mut self, change: &Vec3) {
        todo!()
    }

    ///
    /// Rotates the camera by the angle delta around the 'right' direction.
    ///
    pub fn pitch(&mut self, delta_radians: f32) {
        todo!()
    }

    ///
    /// Rotates the camera by the angle delta around the 'up' direction.
    ///
    pub fn yaw(&mut self, delta_radians: f32) {
        todo!()
    }

    ///
    /// Rotates the camera by the angle delta around the 'view' direction.
    ///
    pub fn roll(&mut self, delta_radians: f32) {
        todo!()
    }

    ///
    /// Rotate the camera around the given point while keeping the same distance to the point.
    /// The input `x` specifies the amount of rotation in the left direction and `y` specifies the amount of rotation in the up direction.
    /// If you want the camera up direction to stay fixed, use the [rotate_around_with_fixed_up](Camera::rotate_around_with_fixed_up) function instead.
    ///
    pub fn rotate_around(&mut self, point: &Vec3, x: f32, y: f32) {
        todo!()
    }

    ///
    /// Rotate the camera around the given point while keeping the same distance to the point and the same up direction.
    /// The input `x` specifies the amount of rotation in the left direction and `y` specifies the amount of rotation in the up direction.
    ///
    pub fn rotate_around_with_fixed_up(&mut self, point: &Vec3, x: f32, y: f32) {
        todo!()
    }

    ///
    /// Moves the camera towards the given point by the amount delta while keeping the given minimum and maximum distance to the point.
    ///
    pub fn zoom_towards(
        &mut self,
        point: &Vec3,
        delta: f32,
        minimum_distance: f32,
        maximum_distance: f32,
    ) {
        let minimum_distance = minimum_distance.max(0.0);
        assert!(
            minimum_distance < maximum_distance,
            "minimum_distance larger than maximum_distance"
        );

        let position = *self.position();
        let distance = point.distance(position);
        let direction = (*point - position).normalize();
        let target = *self.target();
        let up = *self.up();
        let new_distance = (distance - delta).clamp(minimum_distance, maximum_distance);
        let new_position = *point - direction * new_distance;
        self.set_view(new_position, new_position + (target - position), up);
        if let ProjectionType::Orthographic { height } = self.projection_type() {
            let h = new_distance * height / distance;
            let z_near = self.z_near();
            let z_far = self.z_far();
            self.set_orthographic_projection(h, z_near, z_far);
        }
    }
}
