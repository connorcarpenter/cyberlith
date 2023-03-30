use bevy_ecs::{
    entity::Entity,
    system::{NonSendMut, Query, Res, ResMut},
};

use render_api::{Assets, Camera, Handle, Material, Mesh, RenderLayer, RenderLayers, Transform};

use crate::{renderer::Object, window::FrameInput};

#[derive(Clone)]
struct CameraWork {
    pub camera: Entity,
    pub lights: Vec<Entity>,
    pub objects: Vec<Entity>,
}

pub fn draw(
    meshes: Res<Assets<Mesh>>,
    materials: Res<Assets<Material>>,
    frame_input: NonSendMut<FrameInput<()>>,
    cameras_q: Query<(Entity, &Camera, Option<&RenderLayer>)>,
    objects_q: Query<(
        Entity,
        &Handle<Mesh>,
        &Handle<Material>,
        &Transform,
        Option<&RenderLayer>,
    )>,
) {
    let mut layer_to_order: Vec<Option<usize>> = vec![None; RenderLayers::TOTAL_LAYERS];
    let mut camera_work: Vec<Option<CameraWork>> = vec![None; Camera::MAX_CAMERAS];

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
            lights: Vec::new(),
            objects: Vec::new(),
        });

        layer_to_order[render_layer] = Some(camera_order);
    }

    for (entity, _, _, _, render_layer_wrapper) in objects_q.iter() {
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
            .push(entity);
    }

    for work in camera_work {
        if work.is_none() {
            continue;
        }
        let work = work.unwrap();
        let camera_entity = work.camera;
        let object_entities = work.objects;

        // TODO: set render target based on camera value ...
        let render_target = frame_input.screen();

        let Ok((_, camera, _)) = cameras_q.get(camera_entity) else {
            break;
        };

        // Clear the color and depth of the screen render target using the camera's clear color
        render_target.clear((&camera.clear_operation).into());

        let mut objects: Vec<&dyn Object> = Vec::new();

        // Loop through and add refs to a list
        for object_entity in object_entities {
            let Ok((_, mesh_handle, mat_handle, transform, _)) = objects_q.get(object_entity) else {
                break;
            };

            // get mesh
            let mesh = meshes.get(mesh_handle).unwrap();
            let material = materials.get(mat_handle).unwrap();

            // add object ref to list of objects to be rendered
            //objects.push(&render_ref);
        }

        //render_target.render()
    }
}
