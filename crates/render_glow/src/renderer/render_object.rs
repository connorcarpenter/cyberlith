use std::collections::HashMap;

use math::Mat4;
use render_api::{
    base::AxisAlignedBoundingBox,
    components::{CameraProjection, Transform},
};

use crate::{
    core::{Context, InstanceBuffer, Program, RenderStates},
    renderer::{GpuMesh, Instances, Light, Material, RenderCamera},
};

// Render Object
#[derive(Clone)]
pub struct RenderObject<'a> {
    mesh: &'a GpuMesh,
    material: &'a dyn Material,
    transforms: Vec<Mat4>,
    will_instance: bool,
    instanced_aabb: Option<AxisAlignedBoundingBox>,
}

impl<'a> RenderObject<'a> {
    pub fn new(mesh: &'a GpuMesh, material: &'a dyn Material) -> Self {
        Self {
            mesh,
            material,
            transforms: Vec::new(),
            will_instance: false,
            instanced_aabb: None,
        }
    }

    pub fn add_transform(&mut self, transform: &'a Transform) {
        self.transforms.push(transform.compute_matrix());
        if self.transforms.len() > 1 {
            self.will_instance = true;
        }
    }

    pub fn render(self, render_camera: &'a RenderCamera<'a>, lights: &[&dyn Light]) {
        if self.will_instance {
            RenderObjectInstanced::render(
                render_camera,
                lights,
                self.mesh,
                self.material,
                self.transforms,
            );
        } else {
            RenderObjectSingle::render(
                render_camera,
                lights,
                self.mesh,
                self.material,
                self.transforms[0],
            );
        }
    }

    pub fn aabb(&self) -> AxisAlignedBoundingBox {
        if self.will_instance {
            if self.instanced_aabb.is_none() {
                panic!(
                    "must call 'finalize()' on an instanced render object before calling 'aabb()'!"
                );
            }
            self.instanced_aabb.unwrap()
        } else {
            let mut aabb = self.mesh.aabb;
            aabb.transform(&self.transforms[0]);
            aabb
        }
    }

    pub fn finalize(&mut self) {
        if !self.will_instance {
            // non-instanced render objects don't need to do anything at the moment
            return;
        }

        if self.instanced_aabb.is_some() {
            panic!("shouldn't call finalize twice on an instanced mesh...");
        }

        self.instanced_aabb = {
            let mut aabb = AxisAlignedBoundingBox::EMPTY;
            for i in 0..self.transforms.len() as usize {
                let mut aabb2 = self.mesh.aabb;
                aabb2.transform(&(self.transforms[i]));
                aabb.expand_with_aabb(&aabb2);
            }
            Some(aabb)
        };
    }
}

// Non-instanced rendering
struct RenderObjectSingle;

impl RenderObjectSingle {
    fn render<'a>(
        render_camera: &'a RenderCamera<'a>,
        lights: &[&dyn Light],
        mesh: &'a GpuMesh,
        material: &'a dyn Material,
        transform: Mat4,
    ) {
        let fragment_shader = material.fragment_shader(lights);
        let vertex_shader = Self::vertex_shader_source();

        Context::get()
            .program(vertex_shader, fragment_shader.source, |program| {
                material.use_uniforms(program, render_camera, lights);
                Self::draw(
                    program,
                    material.render_states(),
                    render_camera,
                    mesh,
                    transform,
                );
            })
            .expect("Failed compiling shader");
    }

    fn draw<'a>(
        program: &Program,
        render_states: RenderStates,
        render_camera: &'a RenderCamera<'a>,
        mesh: &'a GpuMesh,
        transform: Mat4,
    ) {
        let camera = render_camera.camera;

        let inverse = transform.inverse();
        program.use_uniform_if_required("normalMatrix", inverse.transpose());

        program.use_uniform(
            "viewProjection",
            render_camera
                .projection
                .projection_matrix(&camera.viewport_or_default())
                * render_camera.transform.view_matrix(),
        );
        program.use_uniform("modelMatrix", transform);

        mesh.draw(program, render_states, camera);
    }

    fn vertex_shader_source() -> String {
        format!(
            "{}{}",
            include_str!("../core/shared.frag"),
            include_str!("geometry/shaders/mesh.vert"),
        )
    }
}

// Instanced rendering
struct RenderObjectInstanced;

impl RenderObjectInstanced {
    fn render<'a>(
        render_camera: &'a RenderCamera<'a>,
        lights: &[&dyn Light],
        mesh: &'a GpuMesh,
        material: &dyn Material,
        transforms: Vec<Mat4>,
    ) {
        let instances = Instances::new(transforms);
        #[cfg(debug_assertions)]
        instances.validate().expect("invalid instances");

        let instance_buffers = Self::create_instance_buffers(&instances);

        let fragment_shader = material.fragment_shader(lights);
        let vertex_shader_source =
            Self::vertex_shader_source();
        Context::get()
            .program(vertex_shader_source, fragment_shader.source, |program| {
                material.use_uniforms(program, render_camera, lights);
                Self::draw(
                    program,
                    material.render_states(),
                    render_camera,
                    mesh,
                    &instance_buffers,
                    instances.count(),
                );
            })
            .expect("Failed compiling shader");
    }

    fn draw<'a>(
        program: &Program,
        render_states: RenderStates,
        render_camera: &'a RenderCamera<'a>,
        mesh: &'a GpuMesh,
        instance_buffers: &HashMap<String, InstanceBuffer>,
        instance_count: u32,
    ) {
        let camera = render_camera.camera;
        let transform = Mat4::IDENTITY;

        program.use_uniform(
            "viewProjection",
            render_camera
                .projection
                .projection_matrix(&camera.viewport_or_default())
                * render_camera.transform.view_matrix(),
        );
        program.use_uniform("modelMatrix", transform);

        for attribute_name in [
            "transform_row1",
            "transform_row2",
            "transform_row3",
        ] {
            if program.requires_attribute(attribute_name) {
                program.use_instance_attribute(
                    attribute_name,
                    instance_buffers
                        .get(attribute_name).unwrap_or_else(|| panic!("the render call requires the {} instance buffer which is missing on the given geometry", attribute_name)),
                );
            }
        }
        mesh.draw_instanced(program, render_states, camera, instance_count);
    }

    fn vertex_shader_source() -> String {
        format!(
            "{}{}{}",
            "#define USE_INSTANCE_TRANSFORMS\n",
            include_str!("../core/shared.frag"),
            include_str!("geometry/shaders/mesh.vert"),
        )
    }

    ///
    /// This function creates the instance buffers, ordering them by distance to the camera
    ///
    fn create_instance_buffers(instances: &Instances) -> HashMap<String, InstanceBuffer> {
        let indices = {
            // No need to order, just return the indices as is.
            (0..instances.transformations.len()).collect::<Vec<usize>>()
        };

        // Next, we can compute the instance buffers with that ordering.
        let mut instance_buffers: HashMap<String, InstanceBuffer> = Default::default();

        {
            let mut transform_row1 = Vec::new();
            let mut transform_row2 = Vec::new();
            let mut transform_row3 = Vec::new();
            for transformation in indices.iter().map(|i| instances.transformations[*i]) {
                transform_row1.push(transformation.row(0));
                transform_row2.push(transformation.row(1));
                transform_row3.push(transformation.row(2));
            }

            instance_buffers.insert("transform_row1".to_string(), InstanceBuffer::new_with_data(&transform_row1));
            instance_buffers.insert("transform_row2".to_string(), InstanceBuffer::new_with_data(&transform_row2));
            instance_buffers.insert("transform_row3".to_string(), InstanceBuffer::new_with_data(&transform_row3));
        }

        instance_buffers
    }
}
