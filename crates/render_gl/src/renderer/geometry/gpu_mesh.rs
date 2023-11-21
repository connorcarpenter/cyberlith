use render_api::{base::*, components::Camera};

use crate::core::*;

pub struct GpuMesh {
    positions: VertexBuffer,
    normals: VertexBuffer,
    aabb: AxisAlignedBoundingBox,
}

impl GpuMesh {
    pub fn new(cpu_mesh: &CpuMesh) -> Self {

        Self {
            aabb: cpu_mesh.compute_aabb(),
            positions: VertexBuffer::new_with_data(&cpu_mesh.to_vertices()),
            normals: VertexBuffer::new_with_data(&cpu_mesh.compute_normals()),
        }
    }

    pub fn aabb(&self) -> AxisAlignedBoundingBox {
        self.aabb
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
