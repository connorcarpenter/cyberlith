use std::collections::HashMap;

use math::Mat4;
use render_api::{
    base::AxisAlignedBoundingBox,
    components::{CameraProjection, Transform},
};

use crate::renderer::lights_shader_source;
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
    instanced_aabb: Option<AxisAlignedBoundingBox>,
}

impl<'a> RenderObject<'a> {
    pub fn new(mesh: &'a GpuMesh, material: &'a dyn Material) -> Self {
        Self {
            mesh,
            material,
            transforms: Vec::new(),
            instanced_aabb: None,
        }
    }

    pub fn add_transform(&mut self, transform: &'a Transform) {
        self.transforms.push(transform.compute_matrix());
    }

    pub fn render(self, render_camera: &'a RenderCamera<'a>, lights: &[&dyn Light]) {
        RenderObjectInstanced::render(
            render_camera,
            lights,
            self.mesh,
            self.material,
            self.transforms,
        );
    }

    pub fn aabb(&self) -> AxisAlignedBoundingBox {
        if self.instanced_aabb.is_none() {
            panic!("must call 'finalize()' on an instanced render object before calling 'aabb()'!");
        }
        self.instanced_aabb.unwrap()
    }

    pub fn finalize(&mut self) {
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

        let fragment_shader = material.fragment_shader();
        let vertex_shader_source = Self::vertex_shader_source(lights);
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

        program.use_uniform(
            "view_projection",
            render_camera
                .projection
                .projection_matrix(&camera.viewport_or_default())
                * render_camera.transform.view_matrix(),
        );

        for attribute_name in ["transform_row1", "transform_row2", "transform_row3"] {
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

    fn vertex_shader_source(lights: &[&dyn Light]) -> String {
        let mut output = lights_shader_source(lights);
        output.push_str(include_str!("../shaders/mesh.vert"));

        output
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

            instance_buffers.insert(
                "transform_row1".to_string(),
                InstanceBuffer::new_with_data(&transform_row1),
            );
            instance_buffers.insert(
                "transform_row2".to_string(),
                InstanceBuffer::new_with_data(&transform_row2),
            );
            instance_buffers.insert(
                "transform_row3".to_string(),
                InstanceBuffer::new_with_data(&transform_row3),
            );
        }

        instance_buffers
    }
}
