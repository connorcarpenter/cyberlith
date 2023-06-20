use math::Mat4;
use render_api::{base::AxisAlignedBoundingBox, components::{CameraProjection, Transform}};

use crate::{core::{Context, Program, RenderStates}, renderer::{FragmentAttributes, Geometry, GpuMesh, Light, Material, RenderCamera}};

// Render Object
#[derive(Clone, Copy)]
pub struct RenderObject<'a> {
    pub mesh: &'a GpuMesh,
    pub material: &'a dyn Material,
    pub transform: Mat4,
}

impl<'a> RenderObject<'a> {
    pub fn new(mesh: &'a GpuMesh, material: &'a dyn Material, transform: &'a Transform) -> Self {
        Self {
            mesh,
            material,
            transform: transform.compute_matrix(),
        }
    }

    pub fn render(&self, camera: &RenderCamera, lights: &[&dyn Light]) {
        self.render_with_material(self.material, camera, lights);
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
            let inverse = self.transform.inverse();
            program.use_uniform_if_required("normalMatrix", inverse.transpose());
        }

        program.use_uniform(
            "viewProjection",
            render_camera
                .projection
                .projection_matrix(&camera.viewport_or_default())
                * render_camera.transform.view_matrix(),
        );
        program.use_uniform("modelMatrix", self.transform);

        self.mesh
            .draw(program, render_states, camera, attributes);
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
}

impl<'a> Geometry for RenderObject<'a> {
    fn render_with_material(
        &self,
        material: &dyn Material,
        render_camera: &RenderCamera,
        lights: &[&dyn Light],
    ) {
        let fragment_shader = material.fragment_shader(lights);
        let vertex_shader_source = self.vertex_shader_source(fragment_shader.attributes);
        Context::get()
            .program(vertex_shader_source, fragment_shader.source, |program| {
                material.use_uniforms(program, render_camera, lights);
                self.draw(
                    program,
                    material.render_states(),
                    render_camera,
                    fragment_shader.attributes,
                );
            })
            .expect("Failed compiling shader");
    }

    fn aabb(&self) -> AxisAlignedBoundingBox {
        let mut aabb = self.mesh.aabb;
        aabb.transform(&self.transform);
        aabb
    }
}
