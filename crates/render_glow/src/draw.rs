use bevy_ecs::system::{NonSendMut, Query, Res, ResMut};

use render_api::{
    base::{CpuMaterial, CpuMesh, CpuTexture2D},
    components::{
        AmbientLight, Camera, PointLight, Projection, RenderLayer, RenderLayers,
        RenderTarget as CameraRenderTarget, Transform,
    },
    Handle,
};

use crate::{
    asset_mapping::AssetMapping,
    core::{GpuDepthTexture2D, GpuTexture2D, RenderTarget},
    renderer::{
        AmbientLightImpl, DirectionalLightImpl, GpuMesh, Material, RenderLight, RenderObject, RenderPass,
        RenderTargetExt,
    },
    window::FrameInput,
};

pub fn draw(
    frame_input: NonSendMut<FrameInput<()>>,
    // Resources
    meshes: Res<AssetMapping<CpuMesh, GpuMesh>>,
    materials: Res<AssetMapping<CpuMaterial, Box<dyn Material>>>,
    mut textures: ResMut<AssetMapping<CpuTexture2D, GpuTexture2D>>,
    mut depth_textures: ResMut<AssetMapping<CpuTexture2D, GpuDepthTexture2D>>,
    // Cameras
    cameras_q: Query<(&Camera, &Transform, &Projection, Option<&RenderLayer>)>,
    // Objects
    objects_q: Query<(
        &Handle<CpuMesh>,
        &Handle<CpuMaterial>,
        &Transform,
        Option<&RenderLayer>,
    )>,
    // Lights
    ambient_lights_q: Query<(&AmbientLight, &AmbientLightImpl, Option<&RenderLayer>)>,
    point_lights_q: Query<(&PointLight, Option<&RenderLayer>)>,
    directional_lights_q: Query<(&DirectionalLightImpl, Option<&RenderLayer>)>,
) {
    let mut layer_to_order: Vec<Vec<usize>> = Vec::with_capacity(RenderLayers::TOTAL_LAYERS);
    layer_to_order.resize(RenderLayers::TOTAL_LAYERS, Vec::new());
    let mut camera_work: Vec<Option<RenderPass>> = Vec::with_capacity(Camera::MAX_CAMERAS);
    for _ in 0..Camera::MAX_CAMERAS {
        camera_work.push(None);
    }

    // Aggregate Cameras
    for (camera, transform, projection, render_layer_wrapper) in cameras_q.iter() {
        if !camera.is_active {
            continue;
        }
        let camera_order = camera.order();
        if camera_work[camera_order].is_some() {
            panic!("Each Camera must have a unique `order` value!");
        }

        let render_layer = convert_wrapper(render_layer_wrapper);

        camera_work[camera_order] = Some(RenderPass::from_camera(camera, transform, projection));

        layer_to_order[render_layer].push(camera_order);
    }

    // Aggregate Point Lights
    for (point_light, render_layer_wrapper) in point_lights_q.iter() {
        let render_layer = convert_wrapper(render_layer_wrapper);
        for camera_index in layer_to_order[render_layer].iter().map(|x| *x) {
            if camera_work[camera_index].is_none() {
                panic!("Found PointLight with RenderLayer not associated with any Camera!");
            }

            camera_work[camera_index]
                .as_mut()
                .unwrap()
                .lights
                .push(RenderLight::wrapped(point_light));
        }
    }

    // Aggregate Directional Lights
    for (directional_light_impl, render_layer_wrapper) in directional_lights_q.iter() {
        let render_layer = convert_wrapper(render_layer_wrapper);
        for camera_index in layer_to_order[render_layer].iter().map(|x| *x) {
            if camera_work[camera_index].is_none() {
                panic!("Found DirectionalLight with RenderLayer not associated with any Camera!");
            }

            camera_work[camera_index]
                .as_mut()
                .unwrap()
                .lights
                .push(RenderLight::wrapped(directional_light_impl));
        }
    }

    // Aggregate Ambient Lights
    for (ambient_light, ambient_light_impl, render_layer_wrapper) in ambient_lights_q.iter() {
        let render_layer = convert_wrapper(render_layer_wrapper);
        for camera_index in layer_to_order[render_layer].iter().map(|x| *x) {
            if camera_work[camera_index].is_none() {
                panic!("Found DirectionalLight with RenderLayer not associated with any Camera!");
            }

            camera_work[camera_index]
                .as_mut()
                .unwrap()
                .lights
                .push(RenderLight::ambient(ambient_light, ambient_light_impl));
        }
    }

    // Aggregate RenderObjects
    for (mesh_handle, mat_handle, transform, render_layer_wrapper) in objects_q.iter() {
        let render_layer = convert_wrapper(render_layer_wrapper);
        for camera_index in layer_to_order[render_layer].iter().map(|x| *x) {
            if camera_work[camera_index].is_none() {
                panic!("Found render object with RenderLayer not associated with any Camera!");
            }

            let mesh = meshes.get(mesh_handle).unwrap();
            let mat = materials.get(mat_handle).unwrap();

            let mut render_object = RenderObject::new(mesh, mat.as_ref());
            render_object.add_transform(transform);

            camera_work[camera_index]
                .as_mut()
                .unwrap()
                .objects
                .push(render_object);
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
                    let texture = textures.get_mut(texture_handle).unwrap();
                    let depth_texture = depth_textures.get_mut(texture_handle).unwrap();
                    RenderTarget::new(texture.as_color_target(), depth_texture.as_depth_target())
                }
            }
        };

        // Clear the color and depth of the screen render target using the camera's clear color
        render_target.clear((&render_pass.camera.camera.clear_operation).into());

        render_target.render(render_pass);
    }
}

fn convert_wrapper(w: Option<&RenderLayer>) -> usize {
    if let Some(r) = w {
        r.0
    } else {
        RenderLayers::DEFAULT
    }
}
