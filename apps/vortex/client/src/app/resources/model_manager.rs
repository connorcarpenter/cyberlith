use std::collections::{HashMap, HashSet};

use bevy_ecs::{
    entity::Entity,
    query::With,
    system::Res,
    system::{Commands, Query, ResMut, Resource, SystemState},
    world::{Mut, World},
};
use bevy_log::warn;

use naia_bevy_client::{Client, CommandsExt, ReplicationConfig};

use math::{convert_3d_to_2d, quat_from_spin_direction, Mat4, Quat, SerdeQuat, Vec3};

use render_api::{
    base::{Color, CpuMaterial, CpuMesh},
    components::{Camera, CameraProjection, Projection, RenderLayer, Transform, Visibility},
    resources::RenderFrame,
    shapes,
    shapes::set_2d_line_transform,
    Assets, Handle,
};

use vortex_proto::components::{
    Edge3d, EdgeAngle, Face3d, FileExtension, FileType, ModelTransform, ModelTransformEntityType,
    ShapeName, Vertex3d,
};

use crate::app::{
    components::{
        Edge2dLocal, Edge3dLocal, ModelTransformControl, ModelTransformLocal, OwnedByFileLocal,
    },
    resources::{
        action::model::ModelAction, camera_manager::CameraManager, canvas::Canvas,
        compass::Compass, edge_manager::edge_is_enabled, edge_manager::EdgeManager,
        face_manager::FaceManager, file_manager::FileManager, grid::Grid, input::InputManager,
        shape_data::CanvasShape, shape_manager::ShapeManager, tab_manager::TabManager,
        vertex_manager::VertexManager,
    },
    ui::{widgets::create_networked_dependency, BindingState, UiState},
};

pub struct ModelTransformData {
    edge_2d_entity: Entity,
    owning_file_entity: Entity,
    translation_entity_2d: Entity,
    translation_entity_3d: Entity,
    rotation_entity_2d: Entity,
    rotation_entity_3d: Entity,
    scale_entity_3d: Entity,
    scale_entity_2d: Entity,
}

impl ModelTransformData {
    pub fn new(
        edge_2d_entity: Entity,
        owning_file_entity: Entity,
        translation_entity_2d: Entity,
        translation_entity_3d: Entity,
        rotation_entity_2d: Entity,
        rotation_entity_3d: Entity,
        scale_entity_2d: Entity,
        scale_entity_3d: Entity,
    ) -> Self {
        Self {
            edge_2d_entity,
            owning_file_entity,
            translation_entity_3d,
            rotation_entity_3d,
            scale_entity_3d,
            translation_entity_2d,
            rotation_entity_2d,
            scale_entity_2d,
        }
    }
}

#[derive(Resource)]
pub struct ModelManager {
    file_to_model_transforms: HashMap<Entity, HashSet<Entity>>,
    model_transforms: HashMap<Entity, ModelTransformData>,
    edge_2d_to_model_transform: HashMap<Entity, Entity>,
    // Option<edge_2d_entity>
    binding_edge_opt: Option<Entity>,
}

impl Default for ModelManager {
    fn default() -> Self {
        Self {
            file_to_model_transforms: HashMap::new(),
            model_transforms: HashMap::new(),
            edge_2d_to_model_transform: HashMap::new(),
            binding_edge_opt: None,
        }
    }
}

impl ModelManager {
    pub fn edge_is_binding(&self) -> bool {
        self.binding_edge_opt.is_some()
    }

    pub fn edge_init_assign_skin_or_scene(
        &mut self,
        ui_state: &mut UiState,
        edge_2d_entity: &Entity,
    ) {
        self.binding_edge_opt = Some(*edge_2d_entity);
        let mut file_exts = HashSet::new();
        file_exts.insert(FileExtension::Skin);
        file_exts.insert(FileExtension::Scene);
        ui_state.binding_file = BindingState::Binding(file_exts);
    }

    pub fn take_binding_edge(&mut self) -> Entity {
        self.binding_edge_opt.take().unwrap()
    }

    pub fn process_render_bind_button_result(
        world: &mut World,
        current_file_entity: &Entity,
        dependency_file_ext: &FileExtension,
        dependency_file_entity: &Entity,
        edge_2d_entity: &Entity,
    ) {
        let file_manager = world.get_resource::<FileManager>().unwrap();
        if !file_manager.file_has_dependency(current_file_entity, dependency_file_entity) {
            create_networked_dependency(world, current_file_entity, dependency_file_entity);
        }

        world.resource_scope(|world, mut input_manager: Mut<InputManager>| {
            world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                tab_manager.current_tab_execute_model_action(
                    world,
                    &mut input_manager,
                    ModelAction::CreateModelTransform(
                        *edge_2d_entity,
                        *dependency_file_ext,
                        *dependency_file_entity,
                    ),
                );
            });
        });
    }

    pub fn create_networked_model_transform(
        &mut self,
        world: &mut World,
        input_manager: &mut InputManager,
        edge_2d_entity: &Entity,
        current_file_entity: &Entity,
        dependency_file_ext: &FileExtension,
        dependency_file_entity: &Entity,
    ) -> Entity {
        let mut system_state: SystemState<(
            Commands,
            Client,
            ResMut<Canvas>,
            ResMut<CameraManager>,
            ResMut<VertexManager>,
            ResMut<EdgeManager>,
            ResMut<FaceManager>,
            ResMut<Assets<CpuMesh>>,
            ResMut<Assets<CpuMaterial>>,
            Query<&Vertex3d>,
            Query<&EdgeAngle>,
            Query<&ShapeName>,
        )> = SystemState::new(world);
        let (
            mut commands,
            mut client,
            mut canvas,
            mut camera_manager,
            mut vertex_manager,
            mut edge_manager,
            mut face_manager,
            mut meshes,
            mut materials,
            vertex_3d_q,
            edge_angle_q,
            shape_name_q,
        ) = system_state.get_mut(world);

        input_manager.deselect_shape(&mut canvas);

        let edge_3d_entity = edge_manager.edge_entity_2d_to_3d(&edge_2d_entity).unwrap();

        // get vertex from edge, in order to get name
        let (parent_vertex_3d_entity, vertex_3d_entity) =
            edge_manager.edge_get_endpoints(&edge_3d_entity);
        let shape_name = shape_name_q.get(vertex_3d_entity).unwrap();
        let vertex_name = (*shape_name.value).clone();

        // get translation for model transform (midpoint of edge)
        let parent_original_3d_position =
            vertex_3d_q.get(parent_vertex_3d_entity).unwrap().as_vec3();
        let original_3d_position = vertex_3d_q.get(vertex_3d_entity).unwrap().as_vec3();
        let translation = (parent_original_3d_position + original_3d_position) * 0.5;

        // get rotation
        let target_direction = (original_3d_position - parent_original_3d_position).normalize();
        let edge_old_angle = edge_angle_q.get(edge_3d_entity).unwrap();
        let edge_old_angle: f32 = edge_old_angle.get_radians();
        let rotation = quat_from_spin_direction(edge_old_angle, Vec3::Z, target_direction);

        // get scale
        // TODO: this is naive .. find scale by comparing edge length to skin/scene size
        let scale = Vec3::splat(1.0);

        let mut component = ModelTransform::new(
            vertex_name,
            SerdeQuat::from(rotation),
            translation.x,
            translation.y,
            translation.z,
            scale.x,
            scale.y,
            scale.z,
        );
        let dependency_file_type = match dependency_file_ext {
            FileExtension::Skin => ModelTransformEntityType::Skin,
            FileExtension::Scene => ModelTransformEntityType::Scene,
            _ => {
                panic!("not possible");
            }
        };
        component.set_owner(&client, current_file_entity);
        component.set_entity(&client, *dependency_file_entity, dependency_file_type);
        let new_model_transform_entity = commands
            .spawn_empty()
            .enable_replication(&mut client)
            .configure_replication(ReplicationConfig::Delegated)
            .insert(component)
            .id();

        // postprocess
        self.model_transform_postprocess(
            &mut commands,
            &mut camera_manager,
            &mut vertex_manager,
            &mut edge_manager,
            &mut face_manager,
            &mut meshes,
            &mut materials,
            new_model_transform_entity,
            *edge_2d_entity,
            current_file_entity,
            translation,
        );

        system_state.apply(world);

        new_model_transform_entity
    }

    pub fn model_transform_postprocess(
        &mut self,
        commands: &mut Commands,
        camera_manager: &mut CameraManager,
        vertex_manager: &mut VertexManager,
        edge_manager: &mut EdgeManager,
        face_manager: &mut FaceManager,
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
        new_model_transform_entity: Entity,
        edge_2d_entity: Entity,
        owning_file_entity: &Entity,
        translation: Vec3,
    ) {
        // translation control
        let (translation_entity_2d, translation_entity_3d) = Self::new_model_transform_control(
            commands,
            camera_manager,
            vertex_manager,
            edge_manager,
            face_manager,
            meshes,
            materials,
            translation,
            None,
            Color::LIGHT_BLUE,
        );

        // rotation control
        let (rotation_entity_2d, rotation_entity_3d) = Self::new_model_transform_control(
            commands,
            camera_manager,
            vertex_manager,
            edge_manager,
            face_manager,
            meshes,
            materials,
            translation,
            Some(translation_entity_2d),
            Color::RED,
        );

        // scale control
        let (scale_entity_2d, scale_entity_3d) = Self::new_model_transform_control(
            commands,
            camera_manager,
            vertex_manager,
            edge_manager,
            face_manager,
            meshes,
            materials,
            translation,
            Some(translation_entity_2d),
            Color::WHITE,
        );

        self.register_model_transform_controls(
            new_model_transform_entity,
            edge_2d_entity,
            owning_file_entity,
            translation_entity_2d,
            translation_entity_3d,
            rotation_entity_2d,
            rotation_entity_3d,
            scale_entity_2d,
            scale_entity_3d,
        );
    }

    fn new_model_transform_control(
        commands: &mut Commands,
        camera_manager: &mut CameraManager,
        vertex_manager: &mut VertexManager,
        edge_manager: &mut EdgeManager,
        face_manager: &mut FaceManager,
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
        translation: Vec3,
        translation_entity_2d_opt: Option<Entity>,
        color: Color,
    ) -> (Entity, Entity) {
        let (rotation_entity_2d, rotation_entity_3d, edge_2d_entity_opt, edge_3d_entity_opt) =
            vertex_manager.new_local_vertex(
                commands,
                camera_manager,
                edge_manager,
                face_manager,
                meshes,
                materials,
                translation_entity_2d_opt,
                translation,
                color,
            );

        commands
            .entity(rotation_entity_2d)
            .insert(ModelTransformControl);
        commands
            .entity(rotation_entity_3d)
            .insert(ModelTransformControl)
            .remove::<Handle<CpuMesh>>()
            .remove::<Handle<CpuMaterial>>()
            .remove::<Visibility>();

        if let Some(edge_2d_entity) = edge_2d_entity_opt {
            commands
                .entity(edge_2d_entity)
                .insert(ModelTransformControl);
        }
        if let Some(edge_3d_entity) = edge_3d_entity_opt {
            commands
                .entity(edge_3d_entity)
                .insert(ModelTransformControl)
                .remove::<Handle<CpuMesh>>()
                .remove::<Handle<CpuMaterial>>()
                .remove::<Visibility>();
        }

        (rotation_entity_2d, rotation_entity_3d)
    }

    pub fn register_model_transform_controls(
        &mut self,
        model_transform_entity: Entity,
        edge_2d_entity: Entity,
        owning_file_entity: &Entity,
        translation_entity_2d: Entity,
        translation_entity_3d: Entity,
        rotation_entity_2d: Entity,
        rotation_entity_3d: Entity,
        scale_entity_2d: Entity,
        scale_entity_3d: Entity,
    ) {
        self.model_transforms.insert(
            model_transform_entity,
            ModelTransformData::new(
                edge_2d_entity,
                *owning_file_entity,
                translation_entity_2d,
                translation_entity_3d,
                rotation_entity_2d,
                rotation_entity_3d,
                scale_entity_2d,
                scale_entity_3d,
            ),
        );
        self.edge_2d_to_model_transform
            .insert(edge_2d_entity, model_transform_entity);

        if !self
            .file_to_model_transforms
            .contains_key(owning_file_entity)
        {
            self.file_to_model_transforms
                .insert(*owning_file_entity, HashSet::new());
        }
        let model_transforms = self
            .file_to_model_transforms
            .get_mut(owning_file_entity)
            .unwrap();
        model_transforms.insert(model_transform_entity);
    }

    pub(crate) fn edge_2d_has_model_transform(&self, edge_2d_entity: &Entity) -> bool {
        self.edge_2d_to_model_transform.contains_key(edge_2d_entity)
    }

    pub(crate) fn model_transform_from_edge_2d(&self, edge_2d_entity: &Entity) -> Option<Entity> {
        self.edge_2d_to_model_transform.get(edge_2d_entity).cloned()
    }

    pub(crate) fn on_despawn_model_transform(
        &mut self,
        commands: &mut Commands,
        model_transform_entity: &Entity,
    ) {
        let model_transform_data = self.deregister_model_transform_controls(model_transform_entity);
        commands
            .entity(model_transform_data.translation_entity_2d)
            .despawn();
        commands
            .entity(model_transform_data.translation_entity_3d)
            .despawn();
        commands
            .entity(model_transform_data.rotation_entity_2d)
            .despawn();
        commands
            .entity(model_transform_data.rotation_entity_3d)
            .despawn();
        commands
            .entity(model_transform_data.scale_entity_2d)
            .despawn();
        commands
            .entity(model_transform_data.scale_entity_3d)
            .despawn();
    }

    pub(crate) fn deregister_model_transform_controls(
        &mut self,
        model_transform_entity: &Entity,
    ) -> ModelTransformData {
        let model_transform_data = self
            .model_transforms
            .remove(model_transform_entity)
            .unwrap();
        self.edge_2d_to_model_transform
            .remove(&model_transform_data.edge_2d_entity);

        let model_transforms = self
            .file_to_model_transforms
            .get_mut(&model_transform_data.owning_file_entity)
            .unwrap();
        model_transforms.remove(model_transform_entity);
        if model_transforms.is_empty() {
            self.file_to_model_transforms
                .remove(&model_transform_data.owning_file_entity);
        }

        model_transform_data
    }

    fn sync_transform_controls(
        &self,
        file_entity: &Entity,
        vertex_3d_q: &mut Query<&mut Vertex3d>,
        model_transform_q: &Query<&ModelTransform>,
    ) {
        let Some(model_transform_entities) = self.file_to_model_transforms.get(file_entity) else {
            return;
        };

        let unit_length = 10.0;

        for model_transform_entity in model_transform_entities.iter() {
            let data = self.model_transforms.get(model_transform_entity).unwrap();
            let model_transform = model_transform_q.get(*model_transform_entity).unwrap();

            // translation
            let translation = model_transform.translation_vec3();
            let translation_control_entity = data.translation_entity_3d;
            let mut translation_control_3d =
                vertex_3d_q.get_mut(translation_control_entity).unwrap();
            translation_control_3d.set_vec3(&translation);

            // rotation
            let mut rotation_vector = Vec3::new(0.0, 0.0, unit_length);
            let rotation = model_transform.rotation();
            rotation_vector = rotation * rotation_vector;
            let rotation_with_offset = rotation_vector + translation;
            let rotation_control_entity = data.rotation_entity_3d;
            let mut rotation_control_3d = vertex_3d_q.get_mut(rotation_control_entity).unwrap();
            rotation_control_3d.set_vec3(&rotation_with_offset);

            // scale
            let scale = model_transform.scale_vec3();
            let scale_with_offset = (scale * unit_length) + translation;
            let scale_control_entity = data.scale_entity_3d;
            let mut scale_control_3d = vertex_3d_q.get_mut(scale_control_entity).unwrap();
            scale_control_3d.set_vec3(&scale_with_offset);
        }
    }

    fn model_transform_3d_vertices(&self, file_entity: &Entity) -> Vec<Entity> {
        let mut vertices = Vec::new();
        if let Some(model_transform_entities) = self.file_to_model_transforms.get(file_entity) {
            for model_transform_entity in model_transform_entities.iter() {
                let data = self.model_transforms.get(model_transform_entity).unwrap();
                vertices.push(data.translation_entity_3d);
                vertices.push(data.rotation_entity_3d);
                vertices.push(data.scale_entity_3d);
            }
        }
        vertices
    }

    pub fn sync_vertices(
        &self,
        world: &mut World,
        vertex_manager: &VertexManager,
        file_entity: &Entity,
        camera_3d_entity: &Entity,
        camera_is_2d: bool,
    ) {
        // only triggers when canvas is redrawn

        // ModelTransformControls
        // (setting Vertex3d)
        let mut system_state: SystemState<(Query<&mut Vertex3d>, Query<&ModelTransform>)> =
            SystemState::new(world);
        let (mut vertex_3d_q, model_transform_q) = system_state.get_mut(world);
        self.sync_transform_controls(file_entity, &mut vertex_3d_q, &model_transform_q);

        // gather 3D entities for Skel Vertices, Compass/Grid/ModelTransformControls Vertices
        let mut vertex_3d_entities: HashSet<Entity> = HashSet::new();
        let compass_3d_entities = world.get_resource::<Compass>().unwrap().vertices();
        let grid_3d_entities = world.get_resource::<Grid>().unwrap().vertices();
        let mtc_3d_entites = self.model_transform_3d_vertices(file_entity);
        vertex_3d_entities.extend(compass_3d_entities);
        vertex_3d_entities.extend(grid_3d_entities);
        vertex_3d_entities.extend(mtc_3d_entites);

        let mut system_state: SystemState<(
            Res<FileManager>,
            Query<(Entity, &FileType, &OwnedByFileLocal), With<Vertex3d>>,
        )> = SystemState::new(world);
        let (file_manager, vert_q) = system_state.get_mut(world);
        for (entity, file_type, owned_by_file_local) in vert_q.iter() {
            if *file_type.value == FileExtension::Skel {
                if ShapeManager::is_owned_by_file(
                    &file_manager,
                    file_entity,
                    Some(&owned_by_file_local.file_entity),
                ) {
                    vertex_3d_entities.insert(entity);
                }
            }
        }

        // from 3D vertex entities, get list of 3D edge entities
        let mut edge_3d_entities: HashSet<Entity> = HashSet::new();

        for vertex_3d_entity in vertex_3d_entities.iter() {
            let Some(vertex_data) = vertex_manager.get_vertex_3d_data(vertex_3d_entity) else {
                warn!("vertex_3d_entity {:?} has no vertex_data", vertex_3d_entity);
                continue;
            };

            for edge_3d_entity in vertex_data.edges_3d.iter() {
                edge_3d_entities.insert(*edge_3d_entity);
            }
        }

        // for ALL gathered 3D vertex entities, convert Vertex3D -> 3d Transform
        let mut system_state: SystemState<Query<(&Vertex3d, &mut Transform)>> =
            SystemState::new(world);
        let mut vertex_3d_q = system_state.get_mut(world);

        for vertex_3d_entity in vertex_3d_entities.iter() {
            let Ok((vertex_3d, mut transform)) = vertex_3d_q.get_mut(*vertex_3d_entity) else {
                continue;
            };
            transform.translation = vertex_3d.as_vec3();
        }

        // for ALL gathered 3D edge entities, sync with 3d vertex transforms
        let mut system_state: SystemState<(
            Query<(&Edge3dLocal, Option<&EdgeAngle>)>,
            Query<&mut Transform>,
        )> = SystemState::new(world);
        let (edge_3d_q, mut transform_q) = system_state.get_mut(world);

        for edge_3d_entity in edge_3d_entities.iter() {
            let Ok((edge_3d_local, edge_angle_opt)) = edge_3d_q.get(*edge_3d_entity) else {
                continue;
            };
            EdgeManager::sync_3d_edge(
                &mut transform_q,
                edge_3d_entity,
                edge_3d_local,
                edge_angle_opt,
            );
        }

        if !camera_is_2d {
            return;
        }

        let mut system_state: SystemState<(
            Res<EdgeManager>,
            Query<(&Camera, &Projection)>,
            Query<&mut Transform>,
            Query<&Edge2dLocal>,
        )> = SystemState::new(world);
        let (edge_manager, camera_q, mut transform_q, edge_2d_local_q) =
            system_state.get_mut(world);

        let Ok((camera, camera_projection)) = camera_q.get(*camera_3d_entity) else {
            return;
        };
        let Ok(camera_transform) = transform_q.get(*camera_3d_entity) else {
            return;
        };
        let camera_viewport = camera.viewport.unwrap();
        let view_matrix = camera_transform.view_matrix();
        let projection_matrix = camera_projection.projection_matrix(&camera_viewport);

        // for ALL gathered 2D vertex entities, derive 2d transform from 3d transform
        for vertex_3d_entity in vertex_3d_entities.iter() {
            let Some(vertex_data) = vertex_manager.get_vertex_3d_data(vertex_3d_entity) else {
                warn!("vertex_3d_entity {:?} has no vertex_data", vertex_3d_entity);
                continue;
            };
            let vertex_2d_entity = vertex_data.entity_2d;

            // get 3d transform
            let Ok(vertex_3d_transform) = transform_q.get(*vertex_3d_entity) else {
                warn!("Vertex3d entity {:?} has no Transform", vertex_3d_entity);
                continue;
            };
            // derive 2d transform from 3d transform
            let (coords, depth) = convert_3d_to_2d(
                &view_matrix,
                &projection_matrix,
                &camera_viewport.size_vec2(),
                &vertex_3d_transform.translation,
            );

            // get 2d transform
            let Ok(mut vertex_2d_transform) = transform_q.get_mut(vertex_2d_entity) else {
                panic!("Vertex2d entity {:?} has no Transform", vertex_2d_entity);
            };
            vertex_2d_transform.translation.x = coords.x;
            vertex_2d_transform.translation.y = coords.y;
            vertex_2d_transform.translation.z = depth;
        }

        // for ALL gathered 2D edge entities, derive 2d transform from 2d vertex data
        for edge_3d_entity in edge_3d_entities.iter() {
            let Some(edge_2d_entity) = edge_manager.edge_entity_3d_to_2d(edge_3d_entity) else {
                panic!("edge_3d_entity {:?} has no edge_2d_entity", edge_3d_entity);
            };

            // derive 2d transform from 2d vertex data
            let Ok(edge_endpoints) = edge_2d_local_q.get(edge_2d_entity) else {
                warn!("Edge2d entity {:?} has no Edge2dLocal", edge_2d_entity);
                continue;
            };
            EdgeManager::sync_2d_edge(&mut transform_q, &edge_2d_entity, edge_endpoints);
        }
    }

    pub fn draw(&self, world: &mut World, current_file_entity: &Entity) {
        let Some(current_tab_state) = world.get_resource::<TabManager>().unwrap().current_tab_state() else {
            return;
        };
        let camera_state = &current_tab_state.camera_state;
        let camera_3d_scale = camera_state.camera_3d_scale();
        let camera_is_2d = camera_state.is_2d();
        if camera_is_2d {
            self.draw_2d(world, current_file_entity, camera_3d_scale);
        } else {
            self.draw_3d(world, current_file_entity);
        }
    }

    fn draw_2d(&self, world: &mut World, current_file_entity: &Entity, camera_3d_scale: f32) {
        {
            let mut vertex_3d_entities: HashSet<Entity> = HashSet::new();
            let compass_3d_entities = world.get_resource::<Compass>().unwrap().vertices();
            let grid_3d_entities = world.get_resource::<Grid>().unwrap().vertices();
            let mtc_3d_entites = self.model_transform_3d_vertices(current_file_entity);
            vertex_3d_entities.extend(compass_3d_entities);
            vertex_3d_entities.extend(grid_3d_entities);
            vertex_3d_entities.extend(mtc_3d_entites);

            let mut edge_2d_entities = HashSet::new();

            let mut system_state: SystemState<(
                ResMut<RenderFrame>,
                Res<FileManager>,
                Res<InputManager>,
                Res<VertexManager>,
                Res<EdgeManager>,
                Query<(
                    &Handle<CpuMesh>,
                    &Handle<CpuMaterial>,
                    &Transform,
                    Option<&RenderLayer>,
                )>,
                Query<(Entity, &OwnedByFileLocal, &FileType), With<Edge3d>>,
                Query<Option<&ShapeName>>,
            )> = SystemState::new(world);
            let (
                mut render_frame,
                file_manager,
                input_manager,
                vertex_manager,
                edge_manager,
                objects_q,
                edge_q,
                shape_name_q,
            ) = system_state.get_mut(world);

            // draw vertices (compass, grid, model transform controls)
            for vertex_3d_entity in vertex_3d_entities.iter() {
                // draw vertex 2d
                let Some(data) = vertex_manager.get_vertex_3d_data(&vertex_3d_entity) else {
                    continue;
                };

                let (mesh_handle, mat_handle, transform, render_layer_opt) =
                    objects_q.get(data.entity_2d).unwrap();
                render_frame.draw_object(render_layer_opt, mesh_handle, mat_handle, transform);

                for edge_3d_entity in data.edges_3d.iter() {
                    let edge_2d_entity = edge_manager.edge_entity_3d_to_2d(edge_3d_entity).unwrap();
                    edge_2d_entities.insert(edge_2d_entity);
                }
            }

            // draw edges (compass, grid, model transform controls)
            for edge_2d_entity in edge_2d_entities.iter() {
                let (mesh_handle, mat_handle, transform, render_layer_opt) =
                    objects_q.get(*edge_2d_entity).unwrap();
                render_frame.draw_object(render_layer_opt, mesh_handle, mat_handle, transform);
            }

            let normal_edge_2d_scale = Edge2dLocal::NORMAL_THICKNESS * camera_3d_scale;
            let hover_edge_2d_scale = Edge2dLocal::HOVER_THICKNESS * camera_3d_scale;

            // draw skel bones
            for (edge_3d_entity, owned_by_file, file_type) in edge_q.iter() {
                if *file_type.value != FileExtension::Skel {
                    continue;
                }
                if !ShapeManager::is_owned_by_file(
                    &file_manager,
                    current_file_entity,
                    Some(&owned_by_file.file_entity),
                ) {
                    continue;
                }
                let Some(edge_2d_entity) = edge_manager.edge_entity_3d_to_2d(&edge_3d_entity) else {
                    continue;
                };
                if self.edge_2d_has_model_transform(&edge_2d_entity) {
                    continue;
                }

                let (_, end_vertex_3d_entity) = edge_manager.edge_get_endpoints(&edge_3d_entity);
                let shape_name_opt = shape_name_q.get(end_vertex_3d_entity).unwrap();
                let edge_is_enabled = edge_is_enabled(shape_name_opt);
                let mat_handle = get_shape_color(&vertex_manager, edge_is_enabled);

                let (mesh_handle, _, transform, render_layer_opt) =
                    objects_q.get(edge_2d_entity).unwrap();
                let mut new_transform = transform.clone();
                new_transform.scale.y = normal_edge_2d_scale;
                if let Some((hover_entity, CanvasShape::Edge)) = input_manager.hovered_entity {
                    if hover_entity == edge_2d_entity {
                        new_transform.scale.y = hover_edge_2d_scale;
                    }
                }
                render_frame.draw_object(
                    render_layer_opt,
                    mesh_handle,
                    &mat_handle,
                    &new_transform,
                );
            }

            match input_manager.selected_shape_2d() {
                Some((_, CanvasShape::Edge)) => {
                    // draw select line
                    if let Some(select_line_entity) = input_manager.select_line_entity {
                        let (mesh_handle, mat_handle, transform, render_layer_opt) =
                            objects_q.get(select_line_entity).unwrap();
                        render_frame.draw_object(
                            render_layer_opt,
                            mesh_handle,
                            &mat_handle,
                            transform,
                        );
                    }
                }
                Some((_, CanvasShape::Vertex)) => {
                    // draw select circle
                    if let Some(select_circle_entity) = input_manager.select_circle_entity {
                        let (mesh_handle, mat_handle, transform, render_layer_opt) =
                            objects_q.get(select_circle_entity).unwrap();
                        render_frame.draw_object(
                            render_layer_opt,
                            mesh_handle,
                            &mat_handle,
                            transform,
                        );
                    }
                }
                _ => {}
            }
        }

        {
            // draw models in correct positions
            let Some(model_transform_entities) = self.file_to_model_transforms.get(current_file_entity) else {
                return;
            };

            let mut system_state: SystemState<(
                ResMut<RenderFrame>,
                Client,
                Res<FileManager>,
                Res<CameraManager>,
                Res<VertexManager>,
                ResMut<Assets<CpuMesh>>,
                Query<(&Camera, &Projection, &Transform)>,
                Query<&Vertex3d>,
                Query<(&OwnedByFileLocal, &Edge3dLocal), With<Edge3d>>,
                Query<&ModelTransform>,
            )> = SystemState::new(world);
            let (
                mut render_frame,
                client,
                file_manager,
                camera_manager,
                vertex_manager,
                mut meshes,
                camera_q,
                vertex_q,
                edge_q,
                model_transform_q,
            ) = system_state.get_mut(world);
            let camera_3d_entity = camera_manager.camera_3d_entity().unwrap();
            let Ok((camera, camera_projection, camera_transform)) = camera_q.get(camera_3d_entity) else {
                return;
            };
            let camera_viewport = camera.viewport.unwrap();
            let view_matrix = camera_transform.view_matrix();
            let projection_matrix = camera_projection.projection_matrix(&camera_viewport);

            let render_layer = camera_manager.layer_2d;
            let line_mesh = meshes.add(shapes::Line);
            let line_mat = vertex_manager.mat_disabled_vertex;
            let corrective_rot = Quat::from_rotation_x(f32::to_radians(90.0));

            for model_transform_entity in model_transform_entities {
                let Ok(model_transform) = model_transform_q.get(*model_transform_entity) else {
                    continue;
                };
                let ModelTransformEntityType::Skin = *model_transform.entity_type else {
                    panic!("not possible ... yet");
                };
                let skin_entity = model_transform.skin_or_scene_entity.get(&client).unwrap();
                let mut model_transform = ModelTransformLocal::to_transform(model_transform);
                model_transform.rotation = model_transform.rotation * corrective_rot;
                let model_transform = model_transform.compute_matrix();

                for (owned_by_file, edge_3d_local) in edge_q.iter() {
                    if !ShapeManager::is_owned_by_file(
                        &file_manager,
                        &skin_entity,
                        Some(&owned_by_file.file_entity),
                    ) {
                        continue;
                    }

                    let mut vertices = [Vec3::ZERO, Vec3::ZERO];
                    for (index, vertex_3d_entity) in
                        [edge_3d_local.start, edge_3d_local.end].iter().enumerate()
                    {
                        let Ok(vertex_3d) = vertex_q.get(*vertex_3d_entity) else {
                            continue;
                        };
                        let point = vertex_3d.as_vec3();

                        // transform by model_transform
                        let point = transform_point(&model_transform, &point);

                        // transform to 2D
                        let (coords, depth) = convert_3d_to_2d(
                            &view_matrix,
                            &projection_matrix,
                            &camera_viewport.size_vec2(),
                            &point,
                        );

                        let point = Vec3::new(coords.x, coords.y, depth);

                        // load into output
                        vertices[index] = point;
                    }

                    // get edge transform between both 2d vertex transforms
                    let mut line_transform = Transform::default();
                    let start = vertices[0].truncate();
                    let end = vertices[1].truncate();
                    let depth = (vertices[0].z + vertices[1].z) / 2.0;
                    set_2d_line_transform(&mut line_transform, start, end, depth);

                    // draw edge
                    render_frame.draw_object(
                        Some(&render_layer),
                        &line_mesh,
                        &line_mat,
                        &line_transform,
                    );
                }
            }
        }
    }

    fn draw_3d(&self, world: &mut World, current_file_entity: &Entity) {
        {
            let mut vertex_3d_entities: HashSet<Entity> = HashSet::new();
            let compass_3d_entities = world.get_resource::<Compass>().unwrap().vertices();
            let grid_3d_entities = world.get_resource::<Grid>().unwrap().vertices();
            vertex_3d_entities.extend(compass_3d_entities);
            vertex_3d_entities.extend(grid_3d_entities);

            let mut edge_3d_entities = HashSet::new();

            let mut system_state: SystemState<(
                ResMut<RenderFrame>,
                Res<FileManager>,
                Res<VertexManager>,
                Res<EdgeManager>,
                Query<(
                    &Handle<CpuMesh>,
                    &Handle<CpuMaterial>,
                    &Transform,
                    Option<&RenderLayer>,
                )>,
                Query<(Entity, &OwnedByFileLocal, &FileType), With<Edge3d>>,
                Query<Option<&ShapeName>>,
            )> = SystemState::new(world);
            let (
                mut render_frame,
                file_manager,
                vertex_manager,
                edge_manager,
                objects_q,
                edge_q,
                shape_name_q,
            ) = system_state.get_mut(world);

            // draw vertices (compass, grid)
            for vertex_3d_entity in vertex_3d_entities.iter() {
                // draw vertex 2d
                let Some(data) = vertex_manager.get_vertex_3d_data(&vertex_3d_entity) else { continue };

                let (mesh_handle, mat_handle, transform, render_layer_opt) =
                    objects_q.get(*vertex_3d_entity).unwrap();
                render_frame.draw_object(render_layer_opt, mesh_handle, mat_handle, transform);

                for edge_3d_entity in data.edges_3d.iter() {
                    edge_3d_entities.insert(*edge_3d_entity);
                }
            }

            // draw edges (compass, grid)
            for edge_3d_entity in edge_3d_entities.iter() {
                let (mesh_handle, mat_handle, transform, render_layer_opt) =
                    objects_q.get(*edge_3d_entity).unwrap();
                render_frame.draw_object(render_layer_opt, mesh_handle, mat_handle, transform);
            }

            // skel bone edges
            for (edge_3d_entity, owned_by_file, file_type) in edge_q.iter() {
                if *file_type.value != FileExtension::Skel {
                    continue;
                }
                if !ShapeManager::is_owned_by_file(
                    &file_manager,
                    current_file_entity,
                    Some(&owned_by_file.file_entity),
                ) {
                    continue;
                }
                let Some(edge_2d_entity) = edge_manager.edge_entity_3d_to_2d(&edge_3d_entity) else {
                    continue;
                };
                if self.edge_2d_has_model_transform(&edge_2d_entity) {
                    continue;
                }

                let (_, end_vertex_3d_entity) = edge_manager.edge_get_endpoints(&edge_3d_entity);
                let shape_name_opt = shape_name_q.get(end_vertex_3d_entity).unwrap();
                let edge_is_enabled = edge_is_enabled(shape_name_opt);
                let mat_handle = get_shape_color(&vertex_manager, edge_is_enabled);

                let (mesh_handle, _, transform, render_layer_opt) =
                    objects_q.get(edge_3d_entity).unwrap();
                render_frame.draw_object(render_layer_opt, mesh_handle, &mat_handle, transform);
            }
        }

        {
            // draw skins in correct positions
            let Some(model_transform_entities) = self.file_to_model_transforms.get(current_file_entity) else {
                return;
            };

            let mut system_state: SystemState<(
                ResMut<RenderFrame>,
                Client,
                Res<FileManager>,
                Res<CameraManager>,
                Res<VertexManager>,
                Query<(Entity, &OwnedByFileLocal), With<Face3d>>,
                Query<&ModelTransform>,
                Query<(&Handle<CpuMesh>, &Transform)>,
            )> = SystemState::new(world);
            let (
                mut render_frame,
                client,
                file_manager,
                camera_manager,
                vertex_manager,
                face_q,
                model_transform_q,
                object_q,
            ) = system_state.get_mut(world);

            let render_layer = camera_manager.layer_3d;
            let temp_mat = vertex_manager.mat_disabled_vertex;
            let corrective_rot = Quat::from_rotation_x(f32::to_radians(90.0));

            for model_transform_entity in model_transform_entities {
                let Ok(model_transform) = model_transform_q.get(*model_transform_entity) else {
                    continue;
                };
                let ModelTransformEntityType::Skin = *model_transform.entity_type else {
                    panic!("not possible ... yet");
                };
                let skin_entity = model_transform.skin_or_scene_entity.get(&client).unwrap();
                let mut model_transform = ModelTransformLocal::to_transform(model_transform);
                model_transform.rotation = model_transform.rotation * corrective_rot;

                for (face_3d_entity, owned_by_file) in face_q.iter() {
                    if !ShapeManager::is_owned_by_file(
                        &file_manager,
                        &skin_entity,
                        Some(&owned_by_file.file_entity),
                    ) {
                        continue;
                    }

                    let (mesh_handle, transform) = object_q.get(face_3d_entity).unwrap();

                    let transform = *transform;
                    let transform = model_transform * transform;

                    // draw face
                    render_frame.draw_object(
                        Some(&render_layer),
                        mesh_handle,
                        &temp_mat,
                        &transform,
                    );
                }
            }
        }
    }
}

fn get_shape_color(
    vertex_manager: &Res<VertexManager>,
    edge_is_enabled: bool,
) -> Handle<CpuMaterial> {
    if edge_is_enabled {
        vertex_manager.mat_enabled_vertex
    } else {
        vertex_manager.mat_disabled_vertex
    }
}

fn transform_point(transform_mat: &Mat4, point: &Vec3) -> Vec3 {
    // Convert the point to a 4D vector (homogeneous coordinates)
    let mut point4 = point.extend(1.0);

    // Apply the transformation
    point4 = *transform_mat * point4;

    // Convert the result back to a 3D vector
    point4.truncate()
}
