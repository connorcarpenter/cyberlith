use render_api::{base::*, components::Camera};

use crate::core::*;

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

    pub fn draw(&self, program: &Program, render_states: RenderStates, camera: &Camera) {
        self.use_attributes(program);
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
        instance_count: u32,
    ) {
        self.use_attributes(program);

        program.draw_arrays_instanced(
            render_states,
            camera.viewport_or_default(),
            self.positions.vertex_count(),
            instance_count,
        );
    }

    fn use_attributes(&self, program: &Program) {
        program.use_vertex_attribute("vertex_world_position", &self.positions);
        program.use_vertex_attribute_if_required("vertex_world_normal", &self.normals);

        // TODO: will need to pass "face_id" in here ... ?
    }
}
