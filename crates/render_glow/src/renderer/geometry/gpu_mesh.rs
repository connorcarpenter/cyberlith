use render_api::{base::*, components::Camera};

use crate::{core::*, renderer::*};

pub struct GpuMesh {
    positions: VertexBuffer,
    normals: VertexBuffer,
    pub(crate) aabb: AxisAlignedBoundingBox,
}

impl GpuMesh {
    pub fn new(cpu_mesh: &CpuMesh) -> Self {
        #[cfg(debug_assertions)]
        cpu_mesh.validate().expect("invalid cpu mesh");

        let aabb = cpu_mesh.compute_aabb();

        Self {
            aabb,
            positions: VertexBuffer::new_with_data(&cpu_mesh.to_vertices()),
            normals: VertexBuffer::new_with_data(&cpu_mesh.compute_normals()),
        }
    }

    pub fn draw(
        &self,
        program: &Program,
        render_states: RenderStates,
        camera: &Camera,
        attributes: FragmentAttributes,
    ) {
        self.use_attributes(program, attributes);
        program.draw_arrays(
            render_states,
            camera.viewport_or_default(),
            self.positions.vertex_count(),
        );
    }

    pub fn draw_instanced(
        &self,
        program: &Program,
        render_states: RenderStates,
        camera: &Camera,
        attributes: FragmentAttributes,
        instance_count: u32,
    ) {
        self.use_attributes(program, attributes);

        program.draw_arrays_instanced(
            render_states,
            camera.viewport_or_default(),
            self.positions.vertex_count(),
            instance_count,
        );
    }

    fn use_attributes(&self, program: &Program, attributes: FragmentAttributes) {
        program.use_vertex_attribute("position", &self.positions);

        if attributes.normal {
            program.use_vertex_attribute_if_required(
                "normal",
                &self.normals
            );
        }
    }
}
