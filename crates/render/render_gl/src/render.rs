use bevy_ecs::system::{NonSendMut, Res, ResMut};

use render_api::{
    base::CpuTexture2D,
    components::{Camera, RenderLayers, RenderTarget as CameraRenderTarget},
    resources::RenderFrame,
};

use crate::{
    asset_mapping::AssetMapping,
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
    textures: ResMut<AssetMapping<CpuTexture2D, GpuTexture2D>>,
    depth_textures: ResMut<AssetMapping<CpuTexture2D, GpuDepthTexture2D>>,
) {
    let mut layer_to_order: Vec<Vec<usize>> = Vec::with_capacity(RenderLayers::TOTAL_LAYERS);
    layer_to_order.resize(RenderLayers::TOTAL_LAYERS, Vec::new());
    let mut camera_work: Vec<Option<RenderPass>> = Vec::with_capacity(Camera::MAX_CAMERAS);
    for _ in 0..Camera::MAX_CAMERAS {
        camera_work.push(None);
    }

    let frame_contents = render_frame.take_contents();

    // Aggregate Cameras
    for (render_layer, camera, transform, projection) in frame_contents.cameras.iter() {
        let camera_order = camera.order();
        if camera_work[camera_order].is_some() {
            panic!("Each Camera must have a unique `order` value!");
        }

        camera_work[camera_order] = Some(RenderPass::from_camera(camera, transform, projection));
        layer_to_order[*render_layer].push(camera_order);
    }

    // Aggregate Point Lights
    for (render_layer, point_light) in frame_contents.point_lights.iter() {
        for camera_index in layer_to_order[*render_layer].iter().map(|x| *x) {
            if camera_work[camera_index].is_none() {
                panic!("Found PointLight with RenderLayer not associated with any Camera!");
            }

            camera_work[camera_index]
                .as_mut()
                .unwrap()
                .lights
                .push(point_light);
        }
    }

    // Aggregate Directional Lights
    for (render_layer, dir_light) in frame_contents.directional_lights.iter() {
        for camera_index in layer_to_order[*render_layer].iter().map(|x| *x) {
            if camera_work[camera_index].is_none() {
                panic!("Found DirectionalLight with RenderLayer not associated with any Camera!");
            }

            camera_work[camera_index]
                .as_mut()
                .unwrap()
                .lights
                .push(dir_light);
        }
    }

    // Aggregate Ambient Lights
    for (render_layer, ambient_light) in frame_contents.ambient_lights.iter() {
        for camera_index in layer_to_order[*render_layer].iter().map(|x| *x) {
            if camera_work[camera_index].is_none() {
                panic!("Found AmbientLight with RenderLayer not associated with any Camera!");
            }

            camera_work[camera_index]
                .as_mut()
                .unwrap()
                .lights
                .push(ambient_light);
        }
    }

    // Aggregate Meshes
    for (render_layer, mesh_handle, mat_handle, transform) in frame_contents.meshes.iter() {
        for camera_index in layer_to_order[*render_layer].iter().map(|x| *x) {
            if camera_work[camera_index].is_none() {
                panic!("Found render object with RenderLayer not associated with any Camera!");
            }

            camera_work[camera_index].as_mut().unwrap().add_mesh(
                mesh_handle,
                mat_handle,
                transform,
            );
        }
    }

    // Draw
    for work in camera_work {
        if work.is_none() {
            continue;
        }
        let render_pass = work.unwrap();

        let render_target = {
            match &render_pass.camera.camera.target {
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
        render_target.clear((&render_pass.camera.camera.clear_operation).into());

        render_target.render(
            &gpu_mesh_manager,
            &gpu_material_manager,
            &gpu_skin_manager,
            render_pass,
        );
    }
}