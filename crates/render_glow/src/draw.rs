use bevy_ecs::{
    entity::Entity,
    system::{NonSendMut, Query, Res, ResMut},
};

use render_api::base::Camera;
use render_api::{
    base::{PbrMaterial, Texture2D, TriMesh},
    AmbientLight, Handle, PointLight, RenderLayer, RenderLayers, RenderOperation,
    RenderTarget as CameraRenderTarget, Transform,
};

use crate::renderer::RenderAmbientLight;
use crate::{
    asset_impls::AssetImpls,
    core::{DepthTexture2D, RenderTarget, Texture2DImpl},
    renderer::{
        AmbientLightImpl, BaseMesh, DirectionalLightImpl, Light, Material, RenderCamera,
        RenderLight, RenderObject, RenderPass,
    },
    window::FrameInput,
};

pub fn draw(
    frame_input: NonSendMut<FrameInput<()>>,
    // Resources
    meshes: Res<AssetImpls<TriMesh, BaseMesh>>,
    materials: Res<AssetImpls<PbrMaterial, Box<dyn Material>>>,
    mut textures: ResMut<AssetImpls<Texture2D, Texture2DImpl>>,
    mut depth_textures: ResMut<AssetImpls<Texture2D, DepthTexture2D>>,
    // Cameras
    cameras_q: Query<(&Camera, &Transform, &RenderOperation, Option<&RenderLayer>)>,
    // Objects
    objects_q: Query<(
        &Handle<TriMesh>,
        &Handle<PbrMaterial>,
        &Transform,
        Option<&RenderLayer>,
    )>,
    // Lights
    ambient_lights_q: Query<(&AmbientLight, &AmbientLightImpl, Option<&RenderLayer>)>,
    point_lights_q: Query<(&PointLight, Option<&RenderLayer>)>,
    directional_lights_q: Query<(&DirectionalLightImpl, Option<&RenderLayer>)>,
) {
    let mut layer_to_order: Vec<Option<usize>> = Vec::with_capacity(RenderLayers::TOTAL_LAYERS);
    layer_to_order.resize(RenderLayers::TOTAL_LAYERS, None);
    let mut camera_work: Vec<Option<RenderPass>> = Vec::with_capacity(RenderOperation::MAX_CAMERAS);
    for _ in 0..RenderOperation::MAX_CAMERAS {
        camera_work.push(None);
    }

    // Aggregate Cameras
    for (camera, transform, operation, render_layer_wrapper) in cameras_q.iter() {
        let camera_order = operation.order();
        if camera_work[camera_order].is_some() {
            panic!("Each Camera must have a unique `order` value!");
        }

        let render_layer = convert_wrapper(render_layer_wrapper);
        if layer_to_order.get(render_layer).unwrap().is_some() {
            panic!("Each Camera must have a unique RenderLayer component!");
        }

        camera_work[camera_order] = Some(RenderPass::from_camera(camera, transform, operation));

        layer_to_order[render_layer] = Some(camera_order);
    }

    // Aggregate Point Lights
    for (point_light, render_layer_wrapper) in point_lights_q.iter() {
        let render_layer = convert_wrapper(render_layer_wrapper);
        if layer_to_order.get(render_layer).is_none() {
            panic!("Found PointLight with RenderLayer not associated with any Camera!");
        }
        let camera_index = layer_to_order[render_layer].unwrap();
        if camera_work.get(camera_index).is_none() {
            panic!("Found PointLight with RenderLayer not associated with any Camera!");
        }

        camera_work[camera_index]
            .as_mut()
            .unwrap()
            .lights
            .push(RenderLight::wrapped(point_light));
    }

    // Aggregate Directional Lights
    for (directional_light_impl, render_layer_wrapper) in directional_lights_q.iter() {
        let render_layer = convert_wrapper(render_layer_wrapper);
        if layer_to_order.get(render_layer).is_none() {
            panic!("Found DirectionalLight with RenderLayer not associated with any Camera!");
        }
        let camera_index = layer_to_order[render_layer].unwrap();
        if camera_work.get(camera_index).is_none() {
            panic!("Found DirectionalLight with RenderLayer not associated with any Camera!");
        }

        camera_work[camera_index]
            .as_mut()
            .unwrap()
            .lights
            .push(RenderLight::wrapped(directional_light_impl));
    }

    // Aggregate Ambient Lights
    for (ambient_light, ambient_light_impl, render_layer_wrapper) in ambient_lights_q.iter() {
        let render_layer = convert_wrapper(render_layer_wrapper);
        if layer_to_order.get(render_layer).is_none() {
            panic!("Found DirectionalLight with RenderLayer not associated with any Camera!");
        }
        let camera_index = layer_to_order[render_layer].unwrap();
        if camera_work.get(camera_index).is_none() {
            panic!("Found DirectionalLight with RenderLayer not associated with any Camera!");
        }

        camera_work[camera_index]
            .as_mut()
            .unwrap()
            .lights
            .push(RenderLight::ambient(ambient_light, ambient_light_impl));
    }

    // Aggregate RenderObjects
    for (mesh_handle, mat_handle, transform, render_layer_wrapper) in objects_q.iter() {
        let render_layer = convert_wrapper(render_layer_wrapper);
        if layer_to_order.get(render_layer).is_none() {
            panic!("Found render object with RenderLayer not associated with any Camera!");
        }
        let camera_index: usize = layer_to_order[render_layer].unwrap();
        if camera_work.get(camera_index).is_none() {
            panic!("Found render object with RenderLayer not associated with any Camera!");
        }

        let mesh = meshes.get(mesh_handle).unwrap();
        let mat = materials.get(mat_handle).unwrap();

        camera_work[camera_index]
            .as_mut()
            .unwrap()
            .objects
            .push(RenderObject::new(mesh, mat.as_ref(), transform))
    }

    // Draw
    for work in camera_work {
        if work.is_none() {
            continue;
        }
        let render_pass = work.unwrap();

        let render_target = {
            match &render_pass.camera.operation.target {
                CameraRenderTarget::Screen => frame_input.screen(),
                CameraRenderTarget::Image(texture_handle) => {
                    // Render to Image
                    let mut texture = textures.get_mut(texture_handle).unwrap();
                    let mut depth_texture = depth_textures.get_mut(texture_handle).unwrap();
                    RenderTarget::new(
                        texture.as_color_target(None),
                        depth_texture.as_depth_target(),
                    )
                }
            }
        };

        // Clear the color and depth of the screen render target using the camera's clear color
        render_target.clear((&render_pass.camera.operation.clear_operation).into());

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
