use math::Mat4;
use render_api::{
    base::AxisAlignedBoundingBox,
    components::{CameraProjection, Transform},
};

use crate::{
    core::{Context, Program, RenderStates},
    renderer::{FragmentAttributes, FragmentShader, GpuMesh, Light, Material, RenderCamera},
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

    pub fn render(&self, render_camera: &'a RenderCamera<'a>, lights: &[&dyn Light]) {
        if self.will_instance {
            // instancing!
            todo!();
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
                panic!("must call 'finalize()' on an instanced render object before calling 'aabb()'!");
            }
            self.instanced_aabb.unwrap()
        } else {
            let mut aabb = self.mesh.aabb;
            aabb.transform(&self.transforms[0]);
            aabb
        }
    }

    pub fn finalize(&mut self) {
        //todo!()
    }
}

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
        let vertex_shader = Self::vertex_shader_source(mesh, fragment_shader.attributes);

        Context::get()
            .program(vertex_shader, fragment_shader.source, |program| {
                material.use_uniforms(program, render_camera, lights);
                Self::draw(
                    program,
                    material.render_states(),
                    render_camera,
                    fragment_shader.attributes,
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
        attributes: FragmentAttributes,
        mesh: &'a GpuMesh,
        transform: Mat4,
    ) {
        let camera = render_camera.camera;

        if attributes.normal {
            let inverse = transform.inverse();
            program.use_uniform_if_required("normalMatrix", inverse.transpose());
        }

        program.use_uniform(
            "viewProjection",
            render_camera
                .projection
                .projection_matrix(&camera.viewport_or_default())
                * render_camera.transform.view_matrix(),
        );
        program.use_uniform("modelMatrix", transform);

        mesh.draw(program, render_states, camera, attributes);
    }

    fn vertex_shader_source(mesh: &GpuMesh, required_attributes: FragmentAttributes) -> String {
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
            if mesh.colors.is_some() {
                "#define USE_VERTEX_COLORS\n"
            } else {
                ""
            },
            include_str!("../core/shared.frag"),
            include_str!("geometry/shaders/mesh.vert"),
        )
    }
}