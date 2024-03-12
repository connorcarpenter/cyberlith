use bevy_ecs::system::{NonSendMut, Res, ResMut};

use render_api::{
    base::CpuTexture2D,
    components::{RenderTarget as CameraRenderTarget},
    resources::{RenderFrame, RenderFrameContents},
};
use storage::SideStorage;

use crate::{
    core::{GpuDepthTexture2D, GpuTexture2D, RenderTarget},
    renderer::{RenderPass, RenderTargetExt},
    window::FrameInput,
    GpuMaterialManager, GpuMeshManager, GpuSkinManager,
};

pub fn render(
    mut render_frame: ResMut<RenderFrame>,
    frame_input: NonSendMut<FrameInput<()>>,
    // Resources
    gpu_mesh_manager: Res<GpuMeshManager>,
    gpu_material_manager: Res<GpuMaterialManager>,
    gpu_skin_manager: Res<GpuSkinManager>,
    textures: ResMut<SideStorage<CpuTexture2D, GpuTexture2D>>,
    depth_textures: ResMut<SideStorage<CpuTexture2D, GpuDepthTexture2D>>,
) {
    let frame_contents = render_frame.take_contents();
    let camera_work = contents_to_frame(frame_contents);

    // Draw
    for work in camera_work {
        if work.is_none() {
            continue;
        }
        let render_pass = work.unwrap();

        let render_target = {
            match &render_pass.camera.target {
                CameraRenderTarget::Screen => frame_input.screen(),
                CameraRenderTarget::Image(texture_handle) => {
                    // Render to Image
                    let texture = textures.get(texture_handle).unwrap();
                    let depth_texture = depth_textures.get(texture_handle).unwrap();
                    RenderTarget::new(texture.as_color_target(), depth_texture.as_depth_target())
                }
            }
        };

        // Clear the color and depth of the screen render target using the camera's clear color
        render_target.clear((&render_pass.camera.clear_operation).into());

        render_target.render(
            &gpu_mesh_manager,
            &gpu_material_manager,
            &gpu_skin_manager,
            render_pass,
        );
    }
}

fn contents_to_frame(render_frame_contents: Vec<Option<RenderFrameContents>>) -> Vec<Option<RenderPass>> {
    let mut output = Vec::new();
    for contents_opt in render_frame_contents {
        let result = match contents_opt {
            Some(contents) => {
                Some(RenderPass::from_contents(contents))
            },
            None => None,
        };
        output.push(result);
    }
    output
}