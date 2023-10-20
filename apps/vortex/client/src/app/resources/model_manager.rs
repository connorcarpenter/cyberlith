use std::collections::{HashMap, HashSet};

use bevy_ecs::{
    entity::Entity,
    system::{Commands, Query, ResMut, Resource, SystemState},
    world::{Mut, World},
};

use naia_bevy_client::{Client, CommandsExt, ReplicationConfig};

use math::{quat_from_spin_direction, SerdeQuat, Vec3};

use render_api::{
    base::{Color, CpuMaterial, CpuMesh},
    components::Visibility,
    Assets, Handle,
};

use vortex_proto::components::{
    EdgeAngle, FileExtension, ModelTransform, ModelTransformEntityType, ShapeName, Vertex3d,
};

use crate::app::{
    components::ModelTransformControl,
    resources::{
        action::model::ModelAction, camera_manager::CameraManager, canvas::Canvas,
        edge_manager::EdgeManager, face_manager::FaceManager, input::InputManager,
        tab_manager::TabManager, vertex_manager::VertexManager,
    },
    ui::{widgets::create_networked_dependency, BindingState, UiState},
};

pub struct ModelTransformData {
    edge_2d_entity: Entity,
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
        translation_entity_2d: Entity,
        translation_entity_3d: Entity,
        rotation_entity_2d: Entity,
        rotation_entity_3d: Entity,
        scale_entity_2d: Entity,
        scale_entity_3d: Entity,
    ) -> Self {
        Self {
            edge_2d_entity,
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
    model_transforms: HashMap<Entity, ModelTransformData>,
    edge_2d_to_model_transform: HashMap<Entity, Entity>,
    resync: bool,
    // Option<edge_2d_entity>
    binding_edge_opt: Option<Entity>,
}

impl Default for ModelManager {
    fn default() -> Self {
        Self {
            model_transforms: HashMap::new(),
            edge_2d_to_model_transform: HashMap::new(),
            resync: false,
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
        create_networked_dependency(world, current_file_entity, dependency_file_entity);

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
        model_transform_data
    }

    pub fn queue_resync(&mut self) {
        self.resync = true;
    }

    pub fn will_resync(&self) -> bool {
        self.resync
    }

    pub fn sync_transform_controls(
        &mut self,
        vertex_3d_q: &mut Query<&mut Vertex3d>,
        model_transform_q: &Query<&ModelTransform>,
    ) {
        self.resync = false;

        let unit_length = 10.0;

        for (model_transform_entity, data) in self.model_transforms.iter() {
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

    // pub fn sync_compass_vertices(&self, world: &mut World) {
    //     let mut system_state: SystemState<Query<(&Vertex3d, &mut Transform)>> =
    //         SystemState::new(world);
    //     let mut vertex_3d_q = system_state.get_mut(world);
    //
    //     for vertex_entity in self.compass_vertices_3d.iter() {
    //         let (vertex_3d, mut transform) = vertex_3d_q.get_mut(*vertex_entity).unwrap();
    //         transform.translation = vertex_3d.as_vec3();
    //     }
    // }
}
