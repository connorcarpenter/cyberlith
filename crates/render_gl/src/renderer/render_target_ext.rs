use render_api::components::Viewport;

use crate::{renderer::{RenderPass, RenderMeshes}, GpuMaterialManager, GpuMeshManager, GpuSkinManager};

pub trait RenderTargetExt {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn write(&self, render: impl FnOnce()) -> &Self;

    fn render(
        &self,
        gpu_mesh_manager: &GpuMeshManager,
        gpu_material_manager: &GpuMaterialManager,
        gpu_skin_manager: &GpuSkinManager,
        render_pass: RenderPass,
    ) -> &Self {
        let (camera, lights, object) = render_pass.take();

        self.write(|| {
            RenderMeshes::render(
                gpu_mesh_manager,
                gpu_material_manager,
                gpu_skin_manager,
                &camera,
                &lights,
                object,
            );
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
