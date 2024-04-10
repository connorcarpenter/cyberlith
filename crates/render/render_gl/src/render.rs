use bevy_ecs::system::{NonSendMut, Res, ResMut};

use render_api::{
    base::CpuTexture2D, components::RenderTarget as CameraRenderTarget, resources::RenderFrame,
};
use storage::SideStorage;

use crate::{
    core::{GpuDepthTexture2D, GpuTexture2D, RenderTarget},
    renderer::RenderTargetExt,
    window::FrameInput,
    GpuMaterialManager, GpuMeshManager, GpuSkinManager,
};

pub fn render(
    mut render_frame: ResMut<RenderFrame>,
    frame_input: NonSendMut<FrameInput>,
    // Resources
    gpu_mesh_manager: Res<GpuMeshManager>,
    gpu_material_manager: Res<GpuMaterialManager>,
    gpu_skin_manager: Res<GpuSkinManager>,
    textures: ResMut<SideStorage<CpuTexture2D, GpuTexture2D>>,
    depth_textures: ResMut<SideStorage<CpuTexture2D, GpuDepthTexture2D>>,
) {
    let render_passes = render_frame.take_render_passes();

    // Draw
    for render_pass_opt in render_passes {
        if render_pass_opt.is_none() {
            continue;
        }
        let render_pass = render_pass_opt.unwrap();

        // let Some(render_layer) =  render_pass.render_layer else {
        //     panic!("RenderPass has no RenderLayer!")
        // };
        // if render_layer == RenderLayer::UI {
        //     info!("Rendering RenderLayer::UI");
        // } else {
        //     info!("Rendering RenderLayer {:?}", render_layer);
        // }

        let Some(camera) = render_pass.camera_opt.as_ref() else {
            continue;
        };

        let render_target = {
            match &camera.target {
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
        render_target.clear((&camera.clear_operation).into());

        render_target.render(
            &gpu_mesh_manager,
            &gpu_material_manager,
            &gpu_skin_manager,
            render_pass,
        );
    }
}
