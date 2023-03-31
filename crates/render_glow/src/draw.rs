use bevy_ecs::{
    entity::Entity,
    system::{NonSendMut, Query, Res},
};

use render_api::{
    base::{PbrMaterial, TriMesh},
    CameraComponent, Handle, RenderLayer, RenderLayers, Transform,
};

use crate::{
    asset_impls::AssetImpls,
    renderer::{BaseMesh, Material, RenderObject, RenderPass},
    window::FrameInput,
};

#[derive(Clone)]
struct CameraWork {
    pub camera: Entity,
    pub objects: Vec<RenderObject>,
}

pub fn draw(
    meshes: Res<AssetImpls<TriMesh, BaseMesh>>,
    materials: Res<AssetImpls<PbrMaterial, Box<dyn Material>>>,
    frame_input: NonSendMut<FrameInput<()>>,
    cameras_q: Query<(Entity, &CameraComponent, Option<&RenderLayer>)>,
    objects_q: Query<(
        &Handle<TriMesh>,
        &Handle<PbrMaterial>,
        &Transform,
        Option<&RenderLayer>,
    )>,
) {
    let mut layer_to_order: Vec<Option<usize>> = vec![None; RenderLayers::TOTAL_LAYERS];
    let mut camera_work: Vec<Option<CameraWork>> = vec![None; CameraComponent::MAX_CAMERAS];

    for (entity, camera, render_layer_wrapper) in cameras_q.iter() {
        let camera_order = camera.order();
        if camera_work.get(camera_order).unwrap().is_some() {
            panic!("Each Camera must have a unique `order` value!");
        }

        let render_layer = {
            if let Some(r) = render_layer_wrapper {
                r.0
            } else {
                RenderLayers::DEFAULT
            }
        };
        if layer_to_order.get(render_layer).unwrap().is_some() {
            panic!("Each Camera must have a unique RenderLayer component!");
        }

        camera_work[camera_order] = Some(CameraWork {
            camera: entity,
            objects: Vec::new(),
        });

        layer_to_order[render_layer] = Some(camera_order);
    }

    for (mesh_handle, mat_handle, transform, render_layer_wrapper) in objects_q.iter() {
        let render_layer = {
            if let Some(r) = render_layer_wrapper {
                r.0
            } else {
                RenderLayers::DEFAULT
            }
        };
        if layer_to_order.get(render_layer).is_none() {
            panic!("Found render object with RenderLayer not associated with any Camera!");
        }
        let camera_index = layer_to_order[render_layer].unwrap();
        if camera_work.get(camera_index).is_none() {
            panic!("Found render object with RenderLayer not associated with any Camera!");
        }

        camera_work[camera_index]
            .as_mut()
            .unwrap()
            .objects
            .push(RenderObject::new(*mesh_handle, *mat_handle, *transform))
    }

    for work in camera_work {
        if work.is_none() {
            continue;
        }
        let work = work.unwrap();
        let camera_entity = work.camera;
        let objects = work.objects;

        // TODO: set render target based on camera value ...
        let render_target = frame_input.screen();

        let Ok((_, camera, _)) = cameras_q.get(camera_entity) else {
            break;
        };

        // Clear the color and depth of the screen render target using the camera's clear color
        render_target.clear((&camera.clear_operation).into());

        let render_pass = RenderPass::new(&meshes, &materials, &camera.camera, &objects);
        render_target.render(render_pass);
    }
}
