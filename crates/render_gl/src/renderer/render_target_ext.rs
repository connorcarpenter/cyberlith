use std::collections::HashMap;
use render_api::base::CpuMaterial;

use render_api::components::Viewport;

use crate::{AssetMapping, GpuMeshManager};
use crate::renderer::{cmp_render_order, Material, RenderObjectInstanced, RenderPass};

pub trait RenderTargetExt {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn write(&self, render: impl FnOnce()) -> &Self;

    fn render(&self, gpu_mesh_manager: &GpuMeshManager, materials: &AssetMapping<CpuMaterial, Box<dyn Material>>, render_pass: RenderPass) -> &Self {
        let (camera, lights, mut objects) = render_pass.take();

        self.write(|| {
            for (mat_handle, object) in objects {
                RenderObjectInstanced::render(gpu_mesh_manager, materials, &camera, &lights, mat_handle, object);
            }
        });
        self
    }

    ///
    /// Returns the viewport that encloses the entire target.
    ///
    fn viewport(&self) -> Viewport {
        Viewport::new_at_origin(self.width(), self.height())
    }
}
