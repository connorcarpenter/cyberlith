use math::Mat4;
use render_api::{
    base::AxisAlignedBoundingBox,
    components::{CameraProjection, Transform},
};

use crate::{
    core::{Context, Program, RenderStates},
    renderer::{FragmentAttributes, GpuMesh, Light, Material, RenderCamera},
};

// Render Object
#[derive(Clone)]
pub struct RenderObject<'a> {
    mesh: &'a GpuMesh,
    material: &'a dyn Material,
    transforms: Vec<Mat4>,
}

impl<'a> RenderObject<'a> {
    pub fn new(mesh: &'a GpuMesh, material: &'a dyn Material) -> Self {
        Self {
            mesh,
            material,
            transforms: Vec::new(),
        }
    }

    pub fn add_transform(&mut self, transform: &'a Transform) {
        self.transforms.push(transform.compute_matrix());
    }

    pub fn render(&self, render_camera: &RenderCamera, lights: &[&dyn Light]) {
        let fragment_shader = self.material.fragment_shader(lights);
        let vertex_shader_source = self.vertex_shader_source(fragment_shader.attributes);
        Context::get()
            .program(vertex_shader_source, fragment_shader.source, |program| {
                self.material.use_uniforms(program, render_camera, lights);
                self.draw(
                    program,
                    self.material.render_states(),
                    render_camera,
                    fragment_shader.attributes,
                );
            })
            .expect("Failed compiling shader");
    }

    fn draw(
        &self,
        program: &Program,
        render_states: RenderStates,
        render_camera: &'a RenderCamera<'a>,
        attributes: FragmentAttributes,
    ) {
        let camera = render_camera.camera;

        if attributes.normal {
            let inverse = self.transforms[0].inverse();
            program.use_uniform_if_required("normalMatrix", inverse.transpose());
        }

        program.use_uniform(
            "viewProjection",
            render_camera
                .projection
                .projection_matrix(&camera.viewport_or_default())
                * render_camera.transform.view_matrix(),
        );
        program.use_uniform("modelMatrix", self.transforms[0]);

        self.mesh.draw(program, render_states, camera, attributes);
    }

    fn vertex_shader_source(&self, required_attributes: FragmentAttributes) -> String {
        format!(
            "{}{}{}{}{}",
            if required_attributes.normal {
                "#define USE_NORMALS\n"
            } else {
                ""
            },
            if required_attributes.uv {
                "#define USE_UVS\n"
            } else {
                ""
            },
            if self.mesh.colors.is_some() {
                "#define USE_VERTEX_COLORS\n"
            } else {
                ""
            },
            include_str!("../core/shared.frag"),
            include_str!("geometry/shaders/mesh.vert"),
        )
    }

    pub fn aabb(&self) -> AxisAlignedBoundingBox {
        let mut aabb = self.mesh.aabb;
        aabb.transform(&self.transforms[0]);
        aabb
    }
}
