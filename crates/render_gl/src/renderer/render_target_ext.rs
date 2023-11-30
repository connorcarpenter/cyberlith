use std::collections::HashMap;
use render_api::base::CpuMaterial;

use render_api::components::Viewport;

use crate::{AssetMapping, GpuMaterialManager, GpuMeshManager};
use crate::renderer::{cmp_render_order, Material, RenderObjectInstanced, RenderPass};

pub trait RenderTargetExt {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn write(&self, render: impl FnOnce()) -> &Self;

    fn render(&self, gpu_mesh_manager: &GpuMeshManager, gpu_material_manager: &GpuMaterialManager, render_pass: RenderPass) -> &Self {
        let (camera, lights, mut object) = render_pass.take();

        self.write(|| {
            RenderObjectInstanced::render(gpu_mesh_manager, gpu_material_manager, &camera, &lights, object);
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
