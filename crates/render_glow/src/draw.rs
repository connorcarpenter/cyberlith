use bevy_ecs::{
    entity::Entity,
    system::{NonSendMut, Query, Res},
};

use render_api::{
    base::{PbrMaterial, TriMesh},
    AmbientLight, CameraComponent, Handle, PointLight, RenderLayer, RenderLayers, Transform,
};

use crate::{
    asset_impls::AssetImpls,
    renderer::{
        AmbientLightImpl, BaseMesh, DirectionalLightImpl, Light, Material, RenderObject, RenderPass,
    },
    window::FrameInput,
};

struct CameraWork<'a> {
    pub camera: Entity,
    pub objects: Vec<RenderObject<'a>>,
    pub lights: Vec<&'a dyn Light>,
}

pub fn draw(
    frame_input: NonSendMut<FrameInput<()>>,
    // Resources
    meshes: Res<AssetImpls<TriMesh, BaseMesh>>,
    materials: Res<AssetImpls<PbrMaterial, Box<dyn Material>>>,
    // Cameras
    cameras_q: Query<(Entity, &CameraComponent, Option<&RenderLayer>)>,
    // Objects
    objects_q: Query<(
        &Handle<TriMesh>,
        &Handle<PbrMaterial>,
        &Transform,
        Option<&RenderLayer>,
    )>,
    // Lights
    ambient_light: Res<AmbientLight>,
    ambient_light_impl: Res<AmbientLightImpl>,
    point_lights_q: Query<(&PointLight, Option<&RenderLayer>)>,
    directional_lights_q: Query<(&DirectionalLightImpl, Option<&RenderLayer>)>,
) {
    let mut layer_to_order: Vec<Option<usize>> = Vec::with_capacity(RenderLayers::TOTAL_LAYERS);
    layer_to_order.resize(RenderLayers::TOTAL_LAYERS, None);
    let mut camera_work: Vec<Option<CameraWork>> = Vec::with_capacity(CameraComponent::MAX_CAMERAS);
    for _ in 0..CameraComponent::MAX_CAMERAS {
        camera_work.push(None);
    }

    // Aggregate Cameras
    for (entity, camera, render_layer_wrapper) in cameras_q.iter() {
        let camera_order = camera.order();
        if camera_work[camera_order].is_some() {
            panic!("Each Camera must have a unique `order` value!");
        }

        let render_layer = convert_wrapper(render_layer_wrapper);
        if layer_to_order.get(render_layer).unwrap().is_some() {
            panic!("Each Camera must have a unique RenderLayer component!");
        }

        camera_work[camera_order] = Some(CameraWork {
            camera: entity,
            objects: Vec::new(),
            lights: Vec::new(),
        });

        layer_to_order[render_layer] = Some(camera_order);
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
            .push(point_light);
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
            .push(directional_light_impl);
    }

    // Draw
    for work in camera_work {
        if work.is_none() {
            continue;
        }
        let CameraWork {
            camera,
            objects,
            mut lights,
        } = work.unwrap();

        // add ambient light to lights
        let ambient_light_tuple = (&*ambient_light, &*ambient_light_impl);
        lights.push(&ambient_light_tuple);

        // TODO: set render target based on camera value ...
        let render_target = frame_input.screen();

        let Ok((_, camera_component, _)) = cameras_q.get(camera) else {
            break;
        };

        // Clear the color and depth of the screen render target using the camera's clear color
        render_target.clear((&camera_component.clear_operation).into());

        let render_pass = RenderPass::new(&camera_component.camera, &objects, &lights);
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
