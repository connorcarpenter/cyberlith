use bevy_ecs::entity::Entity;
use bevy_ecs::system::{NonSendMut, Query, Res, ResMut};

use three_d::{FrameInput, Object, Gm, Geometry, Material, Light, PostMaterial, ColorTexture, DepthTexture, AxisAlignedBoundingBox, MaterialType};

use crate::{Camera, Handle, Mesh, StandardMaterial, Transform, RenderLayer, RenderLayers, Assets};

#[derive(Clone)]
struct CameraWork {
    pub camera: Entity,
    pub objects: Vec<Entity>,
}

pub fn draw(
    meshes: Res<Assets<Mesh>>,
    materials: Res<Assets<StandardMaterial>>,
    frame_input: NonSendMut<FrameInput<()>>,
    cameras_q: Query<(Entity, &Camera, &RenderLayer)>,
    objects_q: Query<(Entity, &Handle<Mesh>, &Handle<StandardMaterial>, &Transform, &RenderLayer)>
) {
    let mut layer_to_order: Vec<Option<usize>> = vec![None; RenderLayers::TOTAL_LAYERS];
    let mut camera_work: Vec<Option<CameraWork>> = vec![None; Camera::MAX_CAMERAS];

    for (entity, camera, render_layer_wrapper) in cameras_q.iter() {
        let camera_order = camera.order();
        if camera_work.get(camera_order).is_some() {
            panic!("Each Camera must have a unique `order` value!");
        }

        let render_layer = render_layer_wrapper.0;
        if layer_to_order.get(render_layer).is_some() {
            panic!("Each Camera must have a unique RenderLayer component!");
        }

        camera_work[camera_order] = Some(CameraWork {
            camera: entity,
            objects: Vec::new(),
        });

        layer_to_order[render_layer] = Some(camera_order);
    }

    for (entity, _, _, _, render_layer_wrapper) in objects_q.iter() {
        let render_layer = render_layer_wrapper.0;
        if layer_to_order.get(render_layer).is_none() {
            panic!("Found render object with RenderLayer not associated with any Camera!");
        }
        let camera_index = layer_to_order[render_layer].unwrap();
        if camera_work.get(camera_index).is_none() {
            panic!("Found render object with RenderLayer not associated with any Camera!");
        }

        camera_work[camera_index].as_mut().unwrap().objects.push(entity);
    }

    for work in camera_work {
        if work.is_none() {continue;}
        let work = work.unwrap();
        let camera_entity = work.camera;
        let object_entities = work.objects;

        // TODO: set render target based on camera value ...
        let render_target = frame_input.screen();

        let Ok((_, camera, _)) = cameras_q.get(camera_entity) else {
            break;
        };

        // Clear the color and depth of the screen render target using the camera's clear color
        render_target.clear(camera.clear_state);

        let mut objects: Vec<&dyn Object> = Vec::new();

        // Loop through and add refs to a list
        for object_entity in object_entities {
            let Ok((_, mesh_handle, mat_handle, transform, _)) = objects_q.get(object_entity) else {
                break;
            };

            // get mesh
            let mesh = meshes.get(mesh_handle).unwrap();
            let material = materials.get(mat_handle).unwrap();
            let render_ref = RenderRef::new(mesh, material);

            todo!();

            // add object ref to list of objects to be rendered
            //objects.push(&render_ref);
        }
    }
}

pub struct RenderRef<'a, 'b> {
    mesh: &'a Mesh,
    material: &'b StandardMaterial,
}

impl<'a, 'b> RenderRef<'a, 'b> {
    pub fn new(mesh: &'a Mesh, material: &'b StandardMaterial) -> Self {
        Self {
            mesh,
            material,
        }
    }
}

impl<'a, 'b> Geometry for RenderRef<'a, 'b> {
    fn render_with_material(&self, material: &dyn Material, camera: &three_d::Camera, lights: &[&dyn Light]) {
        todo!()
    }

    fn render_with_post_material(&self, material: &dyn PostMaterial, camera: &three_d::Camera, lights: &[&dyn Light], color_texture: Option<ColorTexture>, depth_texture: Option<DepthTexture>) {
        todo!()
    }

    fn aabb(&self) -> AxisAlignedBoundingBox {
        todo!()
    }
}

impl<'a, 'b> Object for RenderRef<'a, 'b> {
    fn render(&self, camera: &three_d::Camera, lights: &[&dyn Light]) {
        todo!()
    }

    fn material_type(&self) -> MaterialType {
        todo!()
    }
}