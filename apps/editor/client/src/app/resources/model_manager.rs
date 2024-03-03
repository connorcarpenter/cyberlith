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

use math::{convert_3d_to_2d, Affine3A, Mat4, Quat, SerdeQuat, Vec3};
use render_api::{
    base::{Color, CpuMaterial, CpuMesh},
    components::{
        Camera, CameraProjection, Projection, RenderLayer, Transform, Viewport, Visibility,
    },
    resources::RenderFrame,
    shapes,
    shapes::set_2d_line_transform,
};
use storage::{Handle, Storage};

use editor_proto::components::{
    Edge3d, EdgeAngle, Face3d, FileExtension, FileType, NetTransform, NetTransformEntityType,
    OwnedByFile, ShapeName, SkinOrSceneEntity, Vertex3d,
};

use crate::app::{
    components::{
        Edge2dLocal, Edge3dLocal, EdgeAngleLocal, LocalShape, NetTransformControl,
        NetTransformControlType, NetTransformLocal, OwnedByFileLocal, ScaleAxis, Vertex2d,
    },
    plugin::Main,
    resources::{
        action::model::ModelAction, camera_manager::CameraManager, canvas::Canvas,
        compass::Compass, edge_manager::edge_is_enabled, edge_manager::EdgeManager,
        file_manager::FileManager, grid::Grid, input::InputManager, shape_data::CanvasShape,
        tab_manager::TabManager, vertex_manager::VertexManager,
    },
    transform_from_endpoints_and_spin,
    ui::{widgets::create_networked_dependency, BindingState, UiState},
};

pub struct NetTransformData {
    model_file_entity: Entity,
    skel_edge_name: Option<String>, // todo: can we get this from component when necessary? (ehh... probably not)
    // Option<(edge 2d entity, edge 3d entity, vertex 3d entity start, vertex 3d entity end)>
    skel_entities: Option<(Entity, Entity, Entity, Entity)>,
    translation_entity_2d: Entity,
    translation_entity_3d: Entity,
    rotation_entity_vert_2d: Entity,
    rotation_entity_edge_2d: Entity,
    rotation_entity_vert_3d: Entity,
    rotation_entity_edge_3d: Entity,
    scale_x_entity_3d: Entity,
    scale_x_entity_2d: Entity,
    scale_y_entity_3d: Entity,
    scale_y_entity_2d: Entity,
    scale_z_entity_3d: Entity,
    scale_z_entity_2d: Entity,
    scale_x_entity_edge_3d: Entity,
    scale_y_entity_edge_3d: Entity,
    scale_z_entity_edge_3d: Entity,
}

impl NetTransformData {
    pub fn new(
        model_file_entity: Entity,
        skel_edge_name: Option<String>,
        translation_entity_2d: Entity,
        translation_entity_3d: Entity,
        rotation_entity_vert_2d: Entity,
        rotation_entity_edge_2d: Entity,
        rotation_entity_vert_3d: Entity,
        rotation_entity_edge_3d: Entity,
        scale_x_entity_2d: Entity,
        scale_x_entity_3d: Entity,
        scale_y_entity_2d: Entity,
        scale_y_entity_3d: Entity,
        scale_z_entity_2d: Entity,
        scale_z_entity_3d: Entity,
        scale_x_entity_edge_3d: Entity,
        scale_y_entity_edge_3d: Entity,
        scale_z_entity_edge_3d: Entity,
    ) -> Self {
        Self {
            model_file_entity,
            skel_edge_name,
            skel_entities: None,
            translation_entity_3d,
            rotation_entity_vert_3d,
            rotation_entity_edge_3d,
            scale_x_entity_3d,
            scale_y_entity_3d,
            scale_z_entity_3d,
            translation_entity_2d,
            rotation_entity_vert_2d,
            rotation_entity_edge_2d,
            scale_x_entity_2d,
            scale_y_entity_2d,
            scale_z_entity_2d,
            scale_x_entity_edge_3d,
            scale_y_entity_edge_3d,
            scale_z_entity_edge_3d,
        }
    }

    fn get_skel_entities(&self) -> Option<(Entity, Entity, Entity)> {
        if let Some((_, edge_3d_entity, vertex_3d_entity_start, vertex_3d_entity_end)) =
            self.skel_entities
        {
            return Some((edge_3d_entity, vertex_3d_entity_start, vertex_3d_entity_end));
        }
        return None;
    }

    fn get_or_update_skel_entities(
        &mut self,
        file_manager: &FileManager,
        edge_manager: &EdgeManager,
        edge_3d_q: &Query<(Entity, &OwnedByFileLocal), With<Edge3d>>,
        shape_name_q: &Query<&ShapeName>,
    ) -> Option<(Entity, Entity, Entity)> {
        if let Some((_, edge_3d_entity, vertex_3d_entity_start, vertex_3d_entity_end)) =
            self.skel_entities
        {
            return Some((edge_3d_entity, vertex_3d_entity_start, vertex_3d_entity_end));
        }

        let skel_file_entity = file_manager
            .file_get_dependency(&self.model_file_entity, FileExtension::Skel)
            .unwrap();

        // go fetch entities ..
        let bone_name: String = self.skel_edge_name.as_ref().unwrap().clone();

        for (edge_3d_entity, owned_by) in edge_3d_q.iter() {
            if owned_by.file_entity != skel_file_entity {
                continue;
            }
            let (vertex_3d_entity_start, vertex_3d_entity_end) =
                edge_manager.edge_get_endpoints(&edge_3d_entity);
            let Ok(shape_name) = shape_name_q.get(vertex_3d_entity_end) else {
                continue;
            };
            if *shape_name.value != bone_name {
                continue;
            }
            let edge_2d_entity = edge_manager.edge_entity_3d_to_2d(&edge_3d_entity).unwrap();
            self.skel_entities = Some((
                edge_2d_entity,
                edge_3d_entity,
                vertex_3d_entity_start,
                vertex_3d_entity_end,
            ));
            return Some((edge_3d_entity, vertex_3d_entity_start, vertex_3d_entity_end));
        }

        return None;
    }

    pub(crate) fn get_bone_transform(
        &self,
        vertex_3d_q: &Query<&Vertex3d>,
        edge_angle_q: &Query<&EdgeAngle>,
    ) -> Option<Transform> {
        let (edge_3d_entity, vertex_3d_entity_start, vertex_3d_entity_end) =
            self.get_skel_entities()?;

        self.get_bone_transform_from_entities(
            vertex_3d_q,
            edge_angle_q,
            &edge_3d_entity,
            &vertex_3d_entity_start,
            &vertex_3d_entity_end,
        )
    }

    pub(crate) fn get_or_update_bone_transform(
        &mut self,
        file_manager: &FileManager,
        edge_manager: &EdgeManager,
        vertex_3d_q: &Query<&Vertex3d>,
        edge_3d_q: &Query<(Entity, &OwnedByFileLocal), With<Edge3d>>,
        edge_angle_q: &Query<&EdgeAngle>,
        shape_name_q: &Query<&ShapeName>,
    ) -> Option<Transform> {
        let (edge_3d_entity, vertex_3d_entity_start, vertex_3d_entity_end) =
            self.get_or_update_skel_entities(file_manager, edge_manager, edge_3d_q, shape_name_q)?;

        self.get_bone_transform_from_entities(
            vertex_3d_q,
            edge_angle_q,
            &edge_3d_entity,
            &vertex_3d_entity_start,
            &vertex_3d_entity_end,
        )
    }

    fn get_bone_transform_from_entities(
        &self,
        vertex_3d_q: &Query<&Vertex3d>,
        edge_angle_q: &Query<&EdgeAngle>,
        edge_3d_entity: &Entity,
        vertex_3d_entity_start: &Entity,
        vertex_3d_entity_end: &Entity,
    ) -> Option<Transform> {
        let Ok(vertex_3d) = vertex_3d_q.get(*vertex_3d_entity_start) else {
            return None;
        };
        let start_pos = vertex_3d.as_vec3();
        let Ok(vertex_3d) = vertex_3d_q.get(*vertex_3d_entity_end) else {
            return None;
        };
        let end_pos = vertex_3d.as_vec3();

        let Ok(edge_angle) = edge_angle_q.get(*edge_3d_entity) else {
            return None;
        };
        let spin = edge_angle.get_radians();

        Some(transform_from_endpoints_and_spin(start_pos, end_pos, spin))
    }

    pub(crate) fn cleanup_deleted_transform(
        self,
        commands: &mut Commands,
        canvas: &mut Canvas,
        input_manager: &mut InputManager,
        vertex_manager: &mut VertexManager,
        edge_manager: &mut EdgeManager,
    ) {
        let mut vertex_3d_entities = Vec::new();

        vertex_3d_entities.push(self.translation_entity_3d);
        vertex_3d_entities.push(self.rotation_entity_vert_3d);
        vertex_3d_entities.push(self.scale_x_entity_3d);
        vertex_3d_entities.push(self.scale_y_entity_3d);
        vertex_3d_entities.push(self.scale_z_entity_3d);

        let mut edges = Vec::new();

        edges.push(self.rotation_entity_edge_3d);
        edges.push(self.scale_x_entity_edge_3d);
        edges.push(self.scale_y_entity_edge_3d);
        edges.push(self.scale_z_entity_edge_3d);

        for vertex_3d_entity in vertex_3d_entities {
            vertex_manager.cleanup_deleted_vertex(
                commands,
                canvas,
                input_manager,
                &vertex_3d_entity,
            );
        }

        for edge_3d_entity in edges {
            edge_manager.cleanup_deleted_edge(
                commands,
                canvas,
                input_manager,
                vertex_manager,
                None,
                &edge_3d_entity,
            );
        }
    }
}

#[derive(Resource)]
pub struct ModelManager {
    file_to_transform_entities: HashMap<Entity, HashSet<Entity>>,
    transform_entities: HashMap<Entity, NetTransformData>,
    // (.model file entity, edge name) -> net transform entity
    name_to_transform_entity: HashMap<(Entity, String), Entity>,
    // Option<Option<edge_2d_entity>>
    transform_binding_opt: Option<Option<Entity>>,

    drags: Vec<(Entity, Transform, Transform)>,
    dragging_entity: Option<Entity>,
    dragging_start: Option<Transform>,
    dragging_end: Option<Transform>,
}

impl Default for ModelManager {
    fn default() -> Self {
        Self {
            file_to_transform_entities: HashMap::new(),
            transform_entities: HashMap::new(),
            name_to_transform_entity: HashMap::new(),
            transform_binding_opt: None,

            drags: Vec::new(),
            dragging_entity: None,
            dragging_start: None,
            dragging_end: None,
        }
    }
}

impl ModelManager {
    pub fn transform_is_binding(&self) -> bool {
        self.transform_binding_opt.is_some()
    }

    pub fn init_assign_skin_or_scene(
        &mut self,
        ui_state: &mut UiState,
        edge_2d_entity_opt: Option<&Entity>,
    ) {
        let edge_2d_entity_opt = edge_2d_entity_opt.map(|edge_2d_entity| *edge_2d_entity);
        self.transform_binding_opt = Some(edge_2d_entity_opt);
        let mut file_exts = HashSet::new();
        file_exts.insert(FileExtension::Skin);
        file_exts.insert(FileExtension::Scene);
        ui_state.binding_file = BindingState::Binding(file_exts);
    }

    pub fn take_binding_result(&mut self) -> Option<Entity> {
        self.transform_binding_opt.take().unwrap()
    }

    pub fn process_render_bind_button_result(
        world: &mut World,
        current_file_entity: &Entity,
        dependency_file_ext: &FileExtension,
        dependency_file_entity: &Entity,
        edge_2d_entity_opt: Option<&Entity>,
    ) {
        let file_manager = world.get_resource::<FileManager>().unwrap();
        if !file_manager.file_has_dependency(current_file_entity, dependency_file_entity) {
            create_networked_dependency(world, current_file_entity, dependency_file_entity);
        }

        let edge_2d_entity_opt = edge_2d_entity_opt.map(|edge_2d_entity| *edge_2d_entity);

        world.resource_scope(|world, mut input_manager: Mut<InputManager>| {
            world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                tab_manager.current_tab_execute_model_action(
                    world,
                    &mut input_manager,
                    ModelAction::CreateTransform(
                        edge_2d_entity_opt,
                        *dependency_file_ext,
                        *dependency_file_entity,
                    ),
                );
            });
        });
    }

    pub fn create_networked_transform(
        &mut self,
        world: &mut World,
        input_manager: &mut InputManager,
        current_file_entity: &Entity,
        dependency_file_ext: &FileExtension,
        dependency_file_entity: &Entity,
        skel_bone_name_opt: Option<String>,
    ) -> Entity {
        let mut system_state: SystemState<(
            Commands,
            Client<Main>,
            ResMut<Canvas>,
            ResMut<CameraManager>,
            ResMut<VertexManager>,
            ResMut<EdgeManager>,
            ResMut<Storage<CpuMesh>>,
            ResMut<Storage<CpuMaterial>>,
        )> = SystemState::new(world);
        let (
            mut commands,
            mut client,
            mut canvas,
            mut camera_manager,
            mut vertex_manager,
            mut edge_manager,
            mut meshes,
            mut materials,
        ) = system_state.get_mut(world);

        input_manager.deselect_shape(&mut canvas);

        let file_type = if skel_bone_name_opt.is_some() {
            FileExtension::Model
        } else {
            FileExtension::Scene
        };

        let mut owned_by_component = OwnedByFile::new();
        owned_by_component
            .file_entity
            .set(&client, current_file_entity);

        let dependency_file_type = match dependency_file_ext {
            FileExtension::Skin => NetTransformEntityType::Skin,
            FileExtension::Scene => NetTransformEntityType::Scene,
            _ => {
                panic!("not possible");
            }
        };
        let mut skin_or_scene_component = SkinOrSceneEntity::new(dependency_file_type);
        skin_or_scene_component
            .value
            .set(&client, dependency_file_entity);

        let new_net_transform_entity = commands
            .spawn_empty()
            .enable_replication(&mut client)
            .configure_replication::<Main>(ReplicationConfig::Delegated)
            .insert(NetTransform::new(
                SerdeQuat::from(Quat::IDENTITY),
                0.0,
                0.0,
                0.0,
                1.0,
                1.0,
                1.0,
            ))
            .insert(FileType::new(file_type))
            .insert(owned_by_component)
            .insert(skin_or_scene_component)
            .id();

        if let Some(edge_name) = &skel_bone_name_opt {
            commands
                .entity(new_net_transform_entity)
                .insert(ShapeName::new(edge_name.clone()));
        }

        // postprocess
        self.net_transform_postprocess(
            &mut commands,
            &mut camera_manager,
            &mut vertex_manager,
            &mut edge_manager,
            &mut meshes,
            &mut materials,
            current_file_entity,
            skel_bone_name_opt,
            new_net_transform_entity,
        );

        system_state.apply(world);

        //

        let mut system_state: SystemState<(Commands, Client<Main>)> = SystemState::new(world);
        let (mut commands, mut client) = system_state.get_mut(world);

        commands
            .entity(new_net_transform_entity)
            .release_authority(&mut client);

        new_net_transform_entity
    }

    pub fn net_transform_postprocess(
        &mut self,
        commands: &mut Commands,
        camera_manager: &mut CameraManager,
        vertex_manager: &mut VertexManager,
        edge_manager: &mut EdgeManager,
        meshes: &mut Storage<CpuMesh>,
        materials: &mut Storage<CpuMaterial>,
        owning_file_entity: &Entity,
        skel_bone_name_opt: Option<String>,
        net_transform_entity: Entity,
    ) {
        // translation control
        let mat_handle = materials.add(Color::LIGHT_BLUE);
        let (translation_entity_2d, translation_entity_3d, _) = Self::new_net_transform_control(
            commands,
            camera_manager,
            vertex_manager,
            edge_manager,
            meshes,
            materials,
            &mat_handle,
            net_transform_entity,
            None,
            NetTransformControlType::Translation,
        );

        // rotation control
        let mat_handle = materials.add(Color::RED);
        let (
            rotation_entity_vert_2d,
            rotation_entity_vert_3d,
            Some((rotation_entity_edge_3d, rotation_entity_edge_2d)),
        ) = Self::new_net_transform_control(
            commands,
            camera_manager,
            vertex_manager,
            edge_manager,
            meshes,
            materials,
            &mat_handle,
            net_transform_entity,
            Some(translation_entity_2d),
            NetTransformControlType::RotationVertex,
        )
        else {
            panic!("should def have an edge here");
        };

        // scale x control
        let mat_handle = materials.add(Color::WHITE);
        let (scale_x_entity_2d, scale_x_entity_3d, Some((scale_x_entity_edge_3d, _))) =
            Self::new_net_transform_control(
                commands,
                camera_manager,
                vertex_manager,
                edge_manager,
                meshes,
                materials,
                &mat_handle,
                net_transform_entity,
                Some(translation_entity_2d),
                NetTransformControlType::Scale(ScaleAxis::X),
            )
        else {
            panic!("should def have an edge here");
        };

        // scale y control
        let (scale_y_entity_2d, scale_y_entity_3d, Some((scale_y_entity_edge_3d, _))) =
            Self::new_net_transform_control(
                commands,
                camera_manager,
                vertex_manager,
                edge_manager,
                meshes,
                materials,
                &mat_handle,
                net_transform_entity,
                Some(translation_entity_2d),
                NetTransformControlType::Scale(ScaleAxis::Y),
            )
        else {
            panic!("should def have an edge here");
        };

        // scale z control
        let (scale_z_entity_2d, scale_z_entity_3d, Some((scale_z_entity_edge_3d, _))) =
            Self::new_net_transform_control(
                commands,
                camera_manager,
                vertex_manager,
                edge_manager,
                meshes,
                materials,
                &mat_handle,
                net_transform_entity,
                Some(translation_entity_2d),
                NetTransformControlType::Scale(ScaleAxis::Z),
            )
        else {
            panic!("should def have an edge here");
        };

        self.register_net_transform_controls(
            owning_file_entity,
            net_transform_entity,
            skel_bone_name_opt,
            translation_entity_2d,
            translation_entity_3d,
            rotation_entity_vert_2d,
            rotation_entity_edge_2d,
            rotation_entity_vert_3d,
            rotation_entity_edge_3d,
            scale_x_entity_2d,
            scale_x_entity_3d,
            scale_y_entity_2d,
            scale_y_entity_3d,
            scale_z_entity_2d,
            scale_z_entity_3d,
            scale_x_entity_edge_3d,
            scale_y_entity_edge_3d,
            scale_z_entity_edge_3d,
        );
    }

    fn new_net_transform_control(
        commands: &mut Commands,
        camera_manager: &mut CameraManager,
        vertex_manager: &mut VertexManager,
        edge_manager: &mut EdgeManager,
        meshes: &mut Storage<CpuMesh>,
        materials: &mut Storage<CpuMaterial>,
        material: &Handle<CpuMaterial>,
        transform_entity: Entity,
        translation_entity_2d_opt: Option<Entity>,
        control_type: NetTransformControlType,
    ) -> (Entity, Entity, Option<(Entity, Entity)>) {
        let edge_angle_opt = match control_type {
            NetTransformControlType::RotationVertex => Some(0.0),
            _ => None,
        };

        let (vertex_entity_2d, vertex_entity_3d, edge_2d_entity_opt, edge_3d_entity_opt) =
            vertex_manager.new_local_vertex(
                commands,
                camera_manager,
                edge_manager,
                meshes,
                materials,
                material,
                translation_entity_2d_opt,
                Vec3::ZERO,
                edge_angle_opt,
            );

        commands
            .entity(vertex_entity_2d)
            .insert(NetTransformControl::new(transform_entity, control_type));
        commands
            .entity(vertex_entity_3d)
            .insert(NetTransformControl::new(
                transform_entity,
                NetTransformControlType::NA,
            ))
            .remove::<Handle<CpuMesh>>()
            .remove::<Handle<CpuMaterial>>()
            .remove::<Visibility>();

        if let Some(edge_2d_entity) = edge_2d_entity_opt {
            let control_type = if edge_angle_opt.is_some() {
                NetTransformControlType::RotationEdge
            } else {
                NetTransformControlType::NA
            };
            commands
                .entity(edge_2d_entity)
                .insert(NetTransformControl::new(transform_entity, control_type));
        }
        if let Some(edge_3d_entity) = edge_3d_entity_opt {
            commands
                .entity(edge_3d_entity)
                .insert(NetTransformControl::new(
                    transform_entity,
                    NetTransformControlType::NA,
                ))
                .remove::<Handle<CpuMesh>>()
                .remove::<Handle<CpuMaterial>>()
                .remove::<Visibility>();
        }

        let mut edge_entity_opt = None;
        if let Some(edge_3d_entity) = edge_3d_entity_opt {
            let Some(edge_2d_entity) = edge_2d_entity_opt else {
                panic!("both should exist or neither");
            };
            edge_entity_opt = Some((edge_3d_entity, edge_2d_entity));
        }

        (vertex_entity_2d, vertex_entity_3d, edge_entity_opt)
    }

    pub fn register_net_transform_controls(
        &mut self,
        model_file_entity: &Entity,
        net_transform_entity: Entity,
        skel_bone_name_opt: Option<String>,
        translation_entity_2d: Entity,
        translation_entity_3d: Entity,
        rotation_entity_vert_2d: Entity,
        rotation_entity_edge_2d: Entity,
        rotation_entity_vert_3d: Entity,
        rotation_entity_edge_3d: Entity,
        scale_x_entity_2d: Entity,
        scale_x_entity_3d: Entity,
        scale_y_entity_2d: Entity,
        scale_y_entity_3d: Entity,
        scale_z_entity_2d: Entity,
        scale_z_entity_3d: Entity,
        scale_x_entity_edge_3d: Entity,
        scale_y_entity_edge_3d: Entity,
        scale_z_entity_edge_3d: Entity,
    ) {
        self.transform_entities.insert(
            net_transform_entity,
            NetTransformData::new(
                *model_file_entity,
                skel_bone_name_opt.clone(),
                translation_entity_2d,
                translation_entity_3d,
                rotation_entity_vert_2d,
                rotation_entity_edge_2d,
                rotation_entity_vert_3d,
                rotation_entity_edge_3d,
                scale_x_entity_2d,
                scale_x_entity_3d,
                scale_y_entity_2d,
                scale_y_entity_3d,
                scale_z_entity_2d,
                scale_z_entity_3d,
                scale_x_entity_edge_3d,
                scale_y_entity_edge_3d,
                scale_z_entity_edge_3d,
            ),
        );
        if let Some(skel_bone_name) = skel_bone_name_opt {
            let key: (Entity, String) = (*model_file_entity, skel_bone_name);
            self.name_to_transform_entity
                .insert(key, net_transform_entity);
        }

        if !self
            .file_to_transform_entities
            .contains_key(model_file_entity)
        {
            self.file_to_transform_entities
                .insert(*model_file_entity, HashSet::new());
        }
        let net_transforms = self
            .file_to_transform_entities
            .get_mut(model_file_entity)
            .unwrap();
        net_transforms.insert(net_transform_entity);
    }

    pub(crate) fn file_transform_entities(
        &self,
        model_file_entity: &Entity,
    ) -> Option<Vec<Entity>> {
        self.file_to_transform_entities
            .get(model_file_entity)
            .map(|set| set.iter().cloned().collect())
    }

    pub(crate) fn net_transform_exists(
        &self,
        model_file_entity: &Entity,
        skel_bone_name: &str,
    ) -> bool {
        let key: (Entity, String) = (*model_file_entity, skel_bone_name.to_string());
        self.name_to_transform_entity.contains_key(&key)
    }

    pub(crate) fn on_despawn_net_transform(
        &mut self,
        commands: &mut Commands,
        canvas: &mut Canvas,
        input_manager: &mut InputManager,
        vertex_manager: &mut VertexManager,
        edge_manager: &mut EdgeManager,
        net_transform_entity: &Entity,
    ) {
        let net_transform_data = self.deregister_transform_controls(net_transform_entity);
        net_transform_data.cleanup_deleted_transform(
            commands,
            canvas,
            input_manager,
            vertex_manager,
            edge_manager,
        );
    }

    pub(crate) fn deregister_transform_controls(
        &mut self,
        net_transform_entity: &Entity,
    ) -> NetTransformData {
        let net_transform_data = self
            .transform_entities
            .remove(net_transform_entity)
            .unwrap();
        if let Some(skel_edge_name) = &net_transform_data.skel_edge_name {
            let key: (Entity, String) =
                (net_transform_data.model_file_entity, skel_edge_name.clone());
            self.name_to_transform_entity.remove(&key);
        }

        let net_transforms = self
            .file_to_transform_entities
            .get_mut(&net_transform_data.model_file_entity)
            .unwrap();
        net_transforms.remove(net_transform_entity);
        if net_transforms.is_empty() {
            self.file_to_transform_entities
                .remove(&net_transform_data.model_file_entity);
        }

        net_transform_data
    }

    fn sync_transform_controls(
        &mut self,
        world: &mut World,
        file_ext: &FileExtension,
        file_entity: &Entity,
    ) {
        let Some(net_transform_entities) = self.file_to_transform_entities.get(file_entity) else {
            return;
        };

        let mut system_state: SystemState<(
            Res<FileManager>,
            Res<EdgeManager>,
            Query<&Vertex3d>,
            Query<(Entity, &OwnedByFileLocal), With<Edge3d>>,
            Query<&EdgeAngle>,
            Query<&ShapeName>,
            Query<&NetTransform>,
        )> = SystemState::new(world);
        let (
            file_manager,
            edge_manager,
            vertex_3d_q,
            edge_3d_q,
            edge_angle_q,
            shape_name_q,
            net_transform_q,
        ) = system_state.get_mut(world);

        let mut vertex_3d_mutations = Vec::new();

        for net_transform_entity in net_transform_entities.iter() {
            let net_transform_data = self
                .transform_entities
                .get_mut(net_transform_entity)
                .unwrap();
            let bone_transform_opt = match file_ext {
                FileExtension::Model => {
                    let Some(bone_transform) = net_transform_data.get_or_update_bone_transform(
                        &file_manager,
                        &edge_manager,
                        &vertex_3d_q,
                        &edge_3d_q,
                        &edge_angle_q,
                        &shape_name_q,
                    ) else {
                        continue;
                    };
                    Some(bone_transform)
                }
                FileExtension::Scene => None,
                _ => panic!("not possible"),
            };

            let net_transform = net_transform_q.get(*net_transform_entity).unwrap();
            let mut net_transform = NetTransformLocal::to_transform(net_transform);

            if let Some(bone_transform) = bone_transform_opt {
                // apply bone transform to net_transform
                net_transform = net_transform.multiply(&bone_transform);
            }

            // translation
            let translation = net_transform.translation;
            let translation_control_entity = net_transform_data.translation_entity_3d;
            vertex_3d_mutations.push((translation_control_entity, translation));

            // rotation
            let mut rotation_vector = Vec3::new(0.0, 0.0, NetTransformControl::EDGE_LENGTH);
            let rotation = net_transform.rotation;
            rotation_vector = rotation * rotation_vector;
            let rotation_with_offset = rotation_vector + translation;
            let rotation_control_entity = net_transform_data.rotation_entity_vert_3d;
            vertex_3d_mutations.push((rotation_control_entity, rotation_with_offset));

            // scale
            let scale = net_transform.scale;

            {
                // scale x
                let scale_x = Vec3::new(scale.x, 0.0, 0.0);
                let scale_x_with_offset =
                    (scale_x * NetTransformControl::SCALE_EDGE_LENGTH) + translation;
                let scale_x_control_entity = net_transform_data.scale_x_entity_3d;
                vertex_3d_mutations.push((scale_x_control_entity, scale_x_with_offset));
            }

            {
                // scale y
                let scale_y = Vec3::new(0.0, scale.y, 0.0);
                let scale_y_with_offset =
                    (scale_y * NetTransformControl::SCALE_EDGE_LENGTH) + translation;
                let scale_y_control_entity = net_transform_data.scale_y_entity_3d;
                vertex_3d_mutations.push((scale_y_control_entity, scale_y_with_offset));
            }

            {
                // scale z
                let scale_z = Vec3::new(0.0, 0.0, scale.z);
                let scale_z_with_offset =
                    (scale_z * NetTransformControl::SCALE_EDGE_LENGTH) + translation;
                let scale_z_control_entity = net_transform_data.scale_z_entity_3d;
                vertex_3d_mutations.push((scale_z_control_entity, scale_z_with_offset));
            }
        }

        let mut system_state: SystemState<Query<&mut Vertex3d>> = SystemState::new(world);
        let mut vertex_3d_q = system_state.get_mut(world);

        for (vertex_3d_entity, new_translation) in vertex_3d_mutations {
            let mut vertex_3d = vertex_3d_q.get_mut(vertex_3d_entity).unwrap();
            vertex_3d.set_vec3(&new_translation);
        }
    }

    pub fn sync_shapes(
        &mut self,
        world: &mut World,
        vertex_manager: &VertexManager,
        file_ext: &FileExtension,
        file_entity: &Entity,
        camera_3d_entity: &Entity,
        camera_is_2d: bool,
        camera_3d_scale: f32,
    ) {
        let (
            vertex_3d_entities,
            local_vertex_3d_entities,
            edge_3d_entities,
            local_edge_3d_entities,
        ) = match self.sync_3d_shapes(
            world,
            vertex_manager,
            file_ext,
            &file_entity,
            camera_3d_scale,
        ) {
            Some(value) => value,
            None => return,
        };

        if !camera_is_2d {
            return;
        }

        Self::sync_2d_shapes(
            world,
            vertex_manager,
            camera_3d_entity,
            camera_3d_scale,
            vertex_3d_entities,
            local_vertex_3d_entities,
            edge_3d_entities,
            local_edge_3d_entities,
        );
    }

    fn sync_3d_shapes(
        &mut self,
        world: &mut World,
        vertex_manager: &VertexManager,
        file_ext: &FileExtension,
        file_entity: &Entity,
        camera_3d_scale: f32,
    ) -> Option<(
        HashSet<Entity>,
        HashSet<Entity>,
        HashSet<Entity>,
        HashSet<Entity>,
    )> {
        // only triggers when canvas is redrawn
        let local_vertex_3d_scale = LocalShape::VERTEX_RADIUS / camera_3d_scale;
        let local_vertex_3d_scale = Vec3::splat(local_vertex_3d_scale);
        let local_edge_3d_scale = LocalShape::EDGE_THICKNESS / camera_3d_scale;

        let skel_file_entity_opt = match file_ext {
            FileExtension::Model => {
                let file_manager = world.get_resource::<FileManager>().unwrap();
                file_manager.file_get_dependency(file_entity, FileExtension::Skel)
            }
            FileExtension::Scene => None,
            _ => panic!("invalid"),
        };

        // TransformControls
        // (setting Vertex3d)
        self.sync_transform_controls(world, file_ext, file_entity);

        // gather 3D entities for Compass/Grid/NetTransformControls Vertices
        let mut vertex_3d_entities: HashSet<Entity> = HashSet::new();
        let mut local_vertex_3d_entities: HashSet<Entity> = HashSet::new();

        let compass_3d_entities = world.get_resource::<Compass>().unwrap().vertices();
        let grid_3d_entities = world.get_resource::<Grid>().unwrap().vertices();
        vertex_3d_entities.extend(compass_3d_entities);
        vertex_3d_entities.extend(grid_3d_entities);
        local_vertex_3d_entities.extend(compass_3d_entities);
        local_vertex_3d_entities.extend(grid_3d_entities);

        let ntc_3d_entites = self.net_transform_3d_vertices(file_entity);
        vertex_3d_entities.extend(ntc_3d_entites);

        let mut system_state: SystemState<Query<(Entity, &OwnedByFileLocal), With<Vertex3d>>> =
            SystemState::new(world);
        let vert_q = system_state.get_mut(world);

        if let Some(skel_file_entity) = skel_file_entity_opt {
            // gather 3d entities for Skel vertices
            for (entity, owned_by_file_local) in vert_q.iter() {
                if skel_file_entity == owned_by_file_local.file_entity {
                    vertex_3d_entities.insert(entity);
                }
            }
        }

        // from 3D vertex entities, get list of 3D edge entities
        let mut edge_3d_entities: HashSet<Entity> = HashSet::new();
        let mut local_edge_3d_entities = HashSet::new();

        for vertex_3d_entity in vertex_3d_entities.iter() {
            let Some(vertex_data) = vertex_manager.get_vertex_3d_data(vertex_3d_entity) else {
                warn!("vertex_3d_entity {:?} has no vertex_data", vertex_3d_entity);
                continue;
            };

            for edge_3d_entity in vertex_data.edges_3d.iter() {
                edge_3d_entities.insert(*edge_3d_entity);

                if local_vertex_3d_entities.contains(vertex_3d_entity) {
                    local_edge_3d_entities.insert(*edge_3d_entity);
                }
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

            if local_vertex_3d_entities.contains(vertex_3d_entity) {
                transform.scale = local_vertex_3d_scale;
            }
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
            if local_edge_3d_entities.contains(edge_3d_entity) {
                let mut transform = transform_q.get_mut(*edge_3d_entity).unwrap();
                transform.scale.y = local_edge_3d_scale;
                transform.scale.z = local_edge_3d_scale;
            }
        }
        Some((
            vertex_3d_entities,
            local_vertex_3d_entities,
            edge_3d_entities,
            local_edge_3d_entities,
        ))
    }

    fn sync_2d_shapes(
        world: &mut World,
        vertex_manager: &VertexManager,
        camera_3d_entity: &Entity,
        camera_3d_scale: f32,
        vertex_3d_entities: HashSet<Entity>,
        local_vertex_3d_entities: HashSet<Entity>,
        edge_3d_entities: HashSet<Entity>,
        local_edge_3d_entities: HashSet<Entity>,
    ) {
        // let vertex_2d_scale = Vec3::splat(LocalShape::VERTEX_RADIUS * camera_3d_scale);
        // let edge_2d_scale = LocalShape::EDGE_THICKNESS * camera_3d_scale;
        let local_vertex_2d_scale = LocalShape::VERTEX_RADIUS;
        let normal_vertex_2d_scale = Vertex2d::RADIUS * camera_3d_scale;
        let hover_vertex_2d_scale = Vertex2d::HOVER_RADIUS * camera_3d_scale;

        let local_edge_2d_scale = LocalShape::EDGE_THICKNESS;
        let normal_edge_2d_scale = Edge2dLocal::NORMAL_THICKNESS * camera_3d_scale;
        let hover_edge_2d_scale = Edge2dLocal::HOVER_THICKNESS * camera_3d_scale;

        let mut system_state: SystemState<(
            Res<InputManager>,
            Res<EdgeManager>,
            Query<(&Camera, &Projection)>,
            Query<&mut Transform>,
            Query<&Edge2dLocal>,
            Query<&EdgeAngleLocal, With<NetTransformControl>>,
        )> = SystemState::new(world);
        let (input_manager, edge_manager, camera_q, mut transform_q, edge_2d_local_q, edge_angle_q) =
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

            if local_vertex_3d_entities.contains(vertex_3d_entity) {
                vertex_2d_transform.scale.x = local_vertex_2d_scale;
                vertex_2d_transform.scale.y = local_vertex_2d_scale;
            } else {
                vertex_2d_transform.scale.x = normal_vertex_2d_scale;
                vertex_2d_transform.scale.y = normal_vertex_2d_scale;
                if let Some((hover_entity, CanvasShape::Vertex)) = input_manager.hovered_entity {
                    if hover_entity == vertex_2d_entity {
                        vertex_2d_transform.scale.x = hover_vertex_2d_scale;
                        vertex_2d_transform.scale.y = hover_vertex_2d_scale;
                    }
                }
            }
        }

        // edge angle attributes
        let edge_angle_base_circle_scale =
            Edge2dLocal::EDGE_ANGLE_BASE_CIRCLE_RADIUS * camera_3d_scale;
        let edge_angle_end_circle_scale =
            Edge2dLocal::EDGE_ANGLE_END_CIRCLE_RADIUS * camera_3d_scale;
        let edge_angle_length = Edge2dLocal::EDGE_ANGLE_LENGTH * camera_3d_scale;
        let edge_angle_thickness = Edge2dLocal::EDGE_ANGLE_THICKNESS * camera_3d_scale;

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
            let mut transform = transform_q.get_mut(edge_2d_entity).unwrap();
            if local_edge_3d_entities.contains(edge_3d_entity) {
                transform.scale.y = local_edge_2d_scale;
            } else {
                transform.scale.y = normal_edge_2d_scale;
                if let Some((hover_entity, CanvasShape::Edge)) = input_manager.hovered_entity {
                    if hover_entity == edge_2d_entity {
                        transform.scale.y = hover_edge_2d_scale;
                    }
                }
            }

            // sync edge angle if it exists (only for ModelControlTransform "rotation" edges)
            if let Ok(edge_angle) = edge_angle_q.get(*edge_3d_entity) {
                let Some((base_circle_entity, angle_edge_entity, end_circle_entity)) =
                    edge_manager.edge_angle_entities(edge_3d_entity)
                else {
                    panic!(
                        "edge_3d_entity {:?} has no edge_angle_entities",
                        edge_3d_entity
                    );
                };
                EdgeManager::sync_edge_angle(
                    edge_angle_base_circle_scale,
                    edge_angle_end_circle_scale,
                    edge_angle_length,
                    edge_angle_thickness,
                    base_circle_entity,
                    angle_edge_entity,
                    end_circle_entity,
                    &mut transform_q,
                    edge_2d_entity,
                    edge_angle.get_radians(),
                );
            }
        }
    }

    pub fn draw(&self, world: &mut World, file_ext: &FileExtension, current_file_entity: &Entity) {
        let Some(current_tab_state) = world
            .get_resource::<TabManager>()
            .unwrap()
            .current_tab_state()
        else {
            return;
        };
        let camera_state = &current_tab_state.camera_state;
        let camera_is_2d = camera_state.is_2d();
        if camera_is_2d {
            self.draw_2d(world, file_ext, current_file_entity);
        } else {
            self.draw_3d(world, file_ext, current_file_entity);
        }
    }

    fn draw_2d(&self, world: &mut World, file_ext: &FileExtension, current_file_entity: &Entity) {
        {
            let mut vertex_3d_entities: HashSet<Entity> = HashSet::new();
            let compass_3d_entities = world.get_resource::<Compass>().unwrap().vertices();
            let grid_3d_entities = world.get_resource::<Grid>().unwrap().vertices();
            let ntc_3d_entites = self.net_transform_3d_vertices(current_file_entity);
            vertex_3d_entities.extend(compass_3d_entities);
            vertex_3d_entities.extend(grid_3d_entities);
            vertex_3d_entities.extend(ntc_3d_entites);

            let mut edge_2d_entities = HashSet::new();

            let ntc_rotation_edge_3d_entities =
                self.net_transform_rotation_edge_3d_entities(current_file_entity);

            let mut system_state: SystemState<(
                ResMut<RenderFrame>,
                Res<FileManager>,
                Res<InputManager>,
                Res<VertexManager>,
                Res<EdgeManager>,
                Query<(&Handle<CpuMesh>, &Transform, Option<&RenderLayer>)>,
                Query<&Handle<CpuMaterial>>,
                Query<(Entity, &OwnedByFileLocal), With<Edge3d>>,
                Query<Option<&ShapeName>>,
            )> = SystemState::new(world);
            let (
                mut render_frame,
                file_manager,
                input_manager,
                vertex_manager,
                edge_manager,
                objects_q,
                materials_q,
                edge_q,
                shape_name_q,
            ) = system_state.get_mut(world);

            let skel_file_entity_opt = match file_ext {
                FileExtension::Model => {
                    file_manager.file_get_dependency(current_file_entity, FileExtension::Skel)
                }
                FileExtension::Scene => None,
                _ => panic!("invalid"),
            };

            // draw vertices (compass, grid, net transform controls)
            for vertex_3d_entity in vertex_3d_entities.iter() {
                // draw vertex 2d
                let Some(data) = vertex_manager.get_vertex_3d_data(&vertex_3d_entity) else {
                    continue;
                };

                let (mesh_handle, transform, render_layer_opt) =
                    objects_q.get(data.entity_2d).unwrap();
                let mat_handle = materials_q.get(data.entity_2d).unwrap();
                render_frame.draw_mesh(render_layer_opt, mesh_handle, mat_handle, transform);

                for edge_3d_entity in data.edges_3d.iter() {
                    let edge_2d_entity = edge_manager.edge_entity_3d_to_2d(edge_3d_entity).unwrap();
                    edge_2d_entities.insert(edge_2d_entity);
                }
            }

            // draw edges (compass, grid, net transform controls)
            for edge_2d_entity in edge_2d_entities.iter() {
                let (mesh_handle, transform, render_layer_opt) =
                    objects_q.get(*edge_2d_entity).unwrap();
                let mat_handle = materials_q.get(*edge_2d_entity).unwrap();
                render_frame.draw_mesh(render_layer_opt, mesh_handle, mat_handle, transform);
            }

            // draw edge angles (net transform controls (rotation only))
            for edge_3d_entity in ntc_rotation_edge_3d_entities.iter() {
                edge_manager.draw_edge_angles(
                    &edge_3d_entity,
                    &mut render_frame,
                    &objects_q,
                    &materials_q,
                );
            }

            if let Some(skel_file_entity) = skel_file_entity_opt {
                // draw skel bones
                for (edge_3d_entity, owned_by_file) in edge_q.iter() {
                    if owned_by_file.file_entity != skel_file_entity {
                        continue;
                    }
                    let Some(edge_2d_entity) = edge_manager.edge_entity_3d_to_2d(&edge_3d_entity)
                    else {
                        continue;
                    };

                    let (_, end_vertex_3d_entity) =
                        edge_manager.edge_get_endpoints(&edge_3d_entity);
                    let shape_name_opt = shape_name_q.get(end_vertex_3d_entity).unwrap();

                    if let Some(shape_name) = shape_name_opt {
                        let shape_name: &str = &(*shape_name.value);
                        if self.net_transform_exists(current_file_entity, shape_name) {
                            continue;
                        }
                    }

                    let edge_is_enabled = edge_is_enabled(shape_name_opt);
                    let mat_handle = get_shape_color(&vertex_manager, edge_is_enabled);

                    let (mesh_handle, transform, render_layer_opt) =
                        objects_q.get(edge_2d_entity).unwrap();
                    render_frame.draw_mesh(render_layer_opt, mesh_handle, &mat_handle, transform);
                }
            }

            // draw select line & circle
            match input_manager.selected_shape_2d() {
                Some((_, CanvasShape::Edge)) => {
                    // draw select line
                    if let Some(select_line_entity) = input_manager.select_line_entity {
                        let (mesh_handle, transform, render_layer_opt) =
                            objects_q.get(select_line_entity).unwrap();
                        let mat_handle = materials_q.get(select_line_entity).unwrap();
                        render_frame.draw_mesh(
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
                        let (mesh_handle, transform, render_layer_opt) =
                            objects_q.get(select_circle_entity).unwrap();
                        let mat_handle = materials_q.get(select_circle_entity).unwrap();
                        render_frame.draw_mesh(
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
            let mut system_state: SystemState<(
                ResMut<RenderFrame>,
                Client<Main>,
                Res<FileManager>,
                Res<CameraManager>,
                Res<VertexManager>,
                ResMut<Storage<CpuMesh>>,
                Query<(&Camera, &Projection, &Transform)>,
                Query<&Vertex3d>,
                Query<(&OwnedByFileLocal, &Edge3dLocal), With<Edge3d>>,
                Query<&EdgeAngle>,
                Query<(&NetTransform, &SkinOrSceneEntity)>,
            )> = SystemState::new(world);
            let (
                mut render_frame,
                client,
                file_manager,
                camera_manager,
                vertex_manager,
                mut meshes,
                camera_q,
                vertex_3d_q,
                edge_q,
                edge_angle_q,
                net_transform_q,
            ) = system_state.get_mut(world);

            let camera_3d_entity = camera_manager.camera_3d_entity().unwrap();
            let Ok((camera, camera_projection, camera_transform)) = camera_q.get(camera_3d_entity)
            else {
                return;
            };
            let camera_viewport = camera.viewport.unwrap();
            let view_matrix = camera_transform.view_matrix();
            let projection_matrix = camera_projection.projection_matrix(&camera_viewport);

            let render_layer = camera_manager.layer_2d;
            let line_mesh = meshes.add(shapes::Line);
            let line_mat = vertex_manager.mat_disabled_vertex;
            //let corrective_rot = Quat::from_rotation_x(f32::to_radians(90.0));
            let file_is_model = *file_ext == FileExtension::Model;

            self.render_2d_skins_recursive(
                current_file_entity,
                file_is_model,
                &mut render_frame,
                &client,
                &file_manager,
                &vertex_3d_q,
                &edge_q,
                &edge_angle_q,
                &net_transform_q,
                &camera_viewport,
                &view_matrix,
                &projection_matrix,
                &render_layer,
                &line_mesh,
                &line_mat,
                None,
            );
        }
    }

    fn render_2d_skins_recursive(
        &self,
        file_entity: &Entity,
        file_is_model: bool,
        render_frame: &mut RenderFrame,
        client: &Client<Main>,
        file_manager: &FileManager,
        vertex_3d_q: &Query<&Vertex3d>,
        edge_q: &Query<(&OwnedByFileLocal, &Edge3dLocal), With<Edge3d>>,
        edge_angle_q: &Query<&EdgeAngle>,
        net_transform_q: &Query<(&NetTransform, &SkinOrSceneEntity)>,
        camera_viewport: &Viewport,
        view_matrix: &Mat4,
        projection_matrix: &Mat4,
        render_layer: &RenderLayer,
        line_mesh: &Handle<CpuMesh>,
        line_mat: &Handle<CpuMaterial>,
        parent_affine_opt: Option<&Affine3A>,
    ) {
        let Some(net_transform_entities) = self.file_to_transform_entities.get(file_entity) else {
            //warn!("current_file_entity {:?} has no net_transform_entities", current_file_entity);
            return;
        };

        for net_transform_entity in net_transform_entities {
            let new_parent_affine_opt: Option<Affine3A> = if file_is_model {
                if parent_affine_opt.is_some() {
                    panic!("not possible");
                }
                let net_transform_data = self.transform_entities.get(net_transform_entity).unwrap();

                // get bone transform
                if let Some(bone_transform) =
                    net_transform_data.get_bone_transform(&vertex_3d_q, &edge_angle_q)
                {
                    Some(bone_transform.compute_affine())
                } else {
                    None
                }
            } else {
                None
            };

            let new_parent_affine_opt_ref = if file_is_model {
                new_parent_affine_opt.as_ref()
            } else {
                parent_affine_opt
            };

            self.render_2d_skins_super_recursive(
                render_frame,
                client,
                file_manager,
                vertex_3d_q,
                edge_q,
                edge_angle_q,
                net_transform_q,
                &camera_viewport,
                &view_matrix,
                &projection_matrix,
                &render_layer,
                &line_mesh,
                &line_mat,
                new_parent_affine_opt_ref,
                net_transform_entity,
            );
        }
    }

    fn render_2d_skins_super_recursive(
        &self,
        render_frame: &mut RenderFrame,
        client: &Client<Main>,
        file_manager: &FileManager,
        vertex_3d_q: &Query<&Vertex3d>,
        edge_q: &Query<(&OwnedByFileLocal, &Edge3dLocal), With<Edge3d>>,
        edge_angle_q: &Query<&EdgeAngle>,
        net_transform_q: &Query<(&NetTransform, &SkinOrSceneEntity)>,
        camera_viewport: &Viewport,
        view_matrix: &Mat4,
        projection_matrix: &Mat4,
        render_layer: &RenderLayer,
        line_mesh: &Handle<CpuMesh>,
        line_mat: &Handle<CpuMaterial>,
        parent_affine_opt: Option<&Affine3A>,
        net_transform_entity: &Entity,
    ) {
        let Ok((net_transform, skin_or_scene_entity)) = net_transform_q.get(*net_transform_entity)
        else {
            warn!(
                "net_transform_entity {:?} has no NetTransform",
                net_transform_entity
            );
            return;
        };

        let local_transform = NetTransformLocal::to_transform(net_transform);

        let current_affine = if let Some(parent_affine) = parent_affine_opt {
            *parent_affine * local_transform.compute_affine()
        } else {
            local_transform.compute_affine()
        };

        match *skin_or_scene_entity.value_type {
            NetTransformEntityType::Skin => {
                let skin_file_entity = skin_or_scene_entity.value.get(client).unwrap();
                let Some(mesh_file_entity) =
                    file_manager.file_get_dependency(&skin_file_entity, FileExtension::Mesh)
                else {
                    warn!(
                        "skin_file_entity {:?} has no mesh_file_entity",
                        skin_file_entity
                    );
                    return;
                };

                for (owned_by_file, edge_3d_local) in edge_q.iter() {
                    if owned_by_file.file_entity != mesh_file_entity {
                        continue;
                    }

                    let mut vertices = [Vec3::ZERO, Vec3::ZERO];
                    for (index, vertex_3d_entity) in
                        [edge_3d_local.start, edge_3d_local.end].iter().enumerate()
                    {
                        let Ok(vertex_3d) = vertex_3d_q.get(*vertex_3d_entity) else {
                            warn!("Vertex3d entity {:?} has no Transform", vertex_3d_entity);
                            continue;
                        };
                        let point = vertex_3d.as_vec3();

                        // transform by net_transform
                        let point = current_affine.transform_point3(point);

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
                    render_frame.draw_mesh(
                        Some(&render_layer),
                        &line_mesh,
                        &line_mat,
                        &line_transform,
                    );
                }
            }
            NetTransformEntityType::Scene => {
                let scene_file_entity = skin_or_scene_entity.value.get(client).unwrap();

                self.render_2d_skins_recursive(
                    &scene_file_entity,
                    false,
                    render_frame,
                    client,
                    file_manager,
                    vertex_3d_q,
                    edge_q,
                    edge_angle_q,
                    net_transform_q,
                    camera_viewport,
                    view_matrix,
                    projection_matrix,
                    render_layer,
                    line_mesh,
                    line_mat,
                    Some(&current_affine),
                );
            }
            _ => {
                panic!("invalid");
            }
        };
    }

    fn draw_3d(&self, world: &mut World, file_ext: &FileExtension, current_file_entity: &Entity) {
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
                Query<(Entity, &OwnedByFileLocal), With<Edge3d>>,
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

            let skel_file_entity_opt = match file_ext {
                FileExtension::Model => {
                    file_manager.file_get_dependency(current_file_entity, FileExtension::Skel)
                }
                FileExtension::Scene => None,
                _ => panic!("invalid"),
            };

            // draw vertices (compass, grid)
            for vertex_3d_entity in vertex_3d_entities.iter() {
                // draw vertex 2d
                let Some(data) = vertex_manager.get_vertex_3d_data(&vertex_3d_entity) else {
                    continue;
                };

                let (mesh_handle, mat_handle, transform, render_layer_opt) =
                    objects_q.get(*vertex_3d_entity).unwrap();
                render_frame.draw_mesh(render_layer_opt, mesh_handle, mat_handle, transform);

                for edge_3d_entity in data.edges_3d.iter() {
                    edge_3d_entities.insert(*edge_3d_entity);
                }
            }

            // draw edges (compass, grid)
            for edge_3d_entity in edge_3d_entities.iter() {
                let (mesh_handle, mat_handle, transform, render_layer_opt) =
                    objects_q.get(*edge_3d_entity).unwrap();
                render_frame.draw_mesh(render_layer_opt, mesh_handle, mat_handle, transform);
            }

            if let Some(skel_file_entity) = skel_file_entity_opt {
                // draw skel bone edges
                for (edge_3d_entity, owned_by_file) in edge_q.iter() {
                    if owned_by_file.file_entity != skel_file_entity {
                        continue;
                    }

                    let (_, end_vertex_3d_entity) =
                        edge_manager.edge_get_endpoints(&edge_3d_entity);
                    let shape_name_opt = shape_name_q.get(end_vertex_3d_entity).unwrap();

                    if let Some(shape_name) = shape_name_opt {
                        let shape_name: &str = &(*shape_name.value);
                        if self.net_transform_exists(current_file_entity, shape_name) {
                            continue;
                        }
                    }

                    let edge_is_enabled = edge_is_enabled(shape_name_opt);
                    let mat_handle = get_shape_color(&vertex_manager, edge_is_enabled);

                    let (mesh_handle, _, transform, render_layer_opt) =
                        objects_q.get(edge_3d_entity).unwrap();
                    render_frame.draw_mesh(render_layer_opt, mesh_handle, &mat_handle, transform);
                }
            }
        }

        {
            // draw skins in correct positions

            let mut system_state: SystemState<(
                ResMut<RenderFrame>,
                Client<Main>,
                Res<FileManager>,
                Res<CameraManager>,
                Query<&Vertex3d>,
                Query<&EdgeAngle>,
                Query<(Entity, &OwnedByFileLocal), With<Face3d>>,
                Query<(&NetTransform, &SkinOrSceneEntity)>,
                Query<(&Handle<CpuMesh>, &Handle<CpuMaterial>, &Transform)>,
            )> = SystemState::new(world);
            let (
                mut render_frame,
                client,
                file_manager,
                camera_manager,
                vertex_3d_q,
                edge_angle_q,
                face_q,
                net_transform_q,
                object_q,
            ) = system_state.get_mut(world);

            let render_layer = camera_manager.layer_3d;
            let file_is_model = *file_ext == FileExtension::Model;

            self.render_3d_faces_recursive(
                current_file_entity,
                file_is_model,
                &mut render_frame,
                &render_layer,
                &client,
                &file_manager,
                &camera_manager,
                &vertex_3d_q,
                &edge_angle_q,
                &face_q,
                &net_transform_q,
                &object_q,
                None,
            );
        }
    }

    fn render_3d_faces_recursive(
        &self,
        file_entity: &Entity,
        file_is_model: bool,
        render_frame: &mut RenderFrame,
        render_layer: &RenderLayer,
        client: &Client<Main>,
        file_manager: &FileManager,
        camera_manager: &CameraManager,
        vertex_3d_q: &Query<&Vertex3d>,
        edge_angle_q: &Query<&EdgeAngle>,
        face_q: &Query<(Entity, &OwnedByFileLocal), With<Face3d>>,
        net_transform_q: &Query<(&NetTransform, &SkinOrSceneEntity)>,
        object_q: &Query<(&Handle<CpuMesh>, &Handle<CpuMaterial>, &Transform)>,
        parent_affine_opt: Option<&Affine3A>,
    ) {
        let Some(net_transform_entities) = self.file_to_transform_entities.get(file_entity) else {
            return;
        };

        for net_transform_entity in net_transform_entities {
            let Ok((net_transform, skin_or_scene_entity)) =
                net_transform_q.get(*net_transform_entity)
            else {
                continue;
            };

            let new_parent_affine_opt: Option<Affine3A> = if file_is_model {
                if parent_affine_opt.is_some() {
                    panic!("not possible");
                }
                let net_transform_data = self.transform_entities.get(net_transform_entity).unwrap();

                // apply bone transform to net_transform
                if let Some(bone_transform) =
                    net_transform_data.get_bone_transform(&vertex_3d_q, &edge_angle_q)
                {
                    Some(bone_transform.compute_affine())
                } else {
                    None
                }
            } else {
                None
            };

            let new_parent_affine_opt_ref = if file_is_model {
                new_parent_affine_opt.as_ref()
            } else {
                parent_affine_opt
            };

            let local_transform = NetTransformLocal::to_transform(net_transform);
            let current_affine = if let Some(parent_affine) = new_parent_affine_opt_ref {
                *parent_affine * local_transform.compute_affine()
            } else {
                local_transform.compute_affine()
            };

            match *skin_or_scene_entity.value_type {
                NetTransformEntityType::Skin => {
                    let skin_file_entity = skin_or_scene_entity.value.get(client).unwrap();
                    let Some(mesh_file_entity) =
                        file_manager.file_get_dependency(&skin_file_entity, FileExtension::Mesh)
                    else {
                        continue;
                    };

                    Self::render_3d_faces(
                        render_frame,
                        face_q,
                        object_q,
                        render_layer,
                        &mesh_file_entity,
                        &current_affine,
                    );
                }
                NetTransformEntityType::Scene => {
                    let scene_file_entity = skin_or_scene_entity.value.get(client).unwrap();

                    self.render_3d_faces_recursive(
                        &scene_file_entity,
                        false,
                        render_frame,
                        render_layer,
                        client,
                        file_manager,
                        camera_manager,
                        vertex_3d_q,
                        edge_angle_q,
                        face_q,
                        net_transform_q,
                        object_q,
                        Some(&current_affine),
                    );
                }
                _ => {
                    panic!("invalid");
                }
            }
        }
    }

    fn render_3d_faces(
        render_frame: &mut RenderFrame,
        face_q: &Query<(Entity, &OwnedByFileLocal), With<Face3d>>,
        object_q: &Query<(&Handle<CpuMesh>, &Handle<CpuMaterial>, &Transform)>,
        render_layer: &RenderLayer,
        mesh_file_entity: &Entity,
        parent_affine: &Affine3A,
    ) {
        for (face_3d_entity, owned_by_file) in face_q.iter() {
            if owned_by_file.file_entity != *mesh_file_entity {
                continue;
            }

            let (mesh_handle, mat_handle, face_transform) = object_q.get(face_3d_entity).unwrap();

            let face_affine = *parent_affine * face_transform.compute_affine();
            let face_transform = Transform::from(face_affine);

            // draw face
            render_frame.draw_mesh(
                Some(&render_layer),
                mesh_handle,
                mat_handle,
                &face_transform,
            );
        }
    }

    pub fn on_drag_transform_end(&mut self, world: &mut World, input_manager: &mut InputManager) {
        // reset last dragged transform
        if let Some(drags) = self.take_drags() {
            world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                for (transform_entity, old_transform, new_transform) in drags {
                    tab_manager.current_tab_execute_model_action(
                        world,
                        input_manager,
                        ModelAction::MoveTransform(
                            transform_entity,
                            old_transform,
                            new_transform,
                            true,
                        ),
                    );
                }
            });
        }
    }

    pub fn reset_last_transform_dragged(&mut self) {
        self.drags = Vec::new();
        self.dragging_entity = None;
        self.dragging_start = None;
        self.dragging_end = None;
    }

    pub fn update_last_transform_dragged(
        &mut self,
        transform_entity: Entity,
        old_transform: Transform,
        new_transform: Transform,
    ) {
        if let Some(old_transform_entity) = self.dragging_entity {
            // already dragging an entity
            if old_transform_entity == transform_entity {
                // dragging same entity
                self.dragging_end = Some(new_transform);
            } else {
                // dragging different entity

                // finish current drag
                self.drags.push((
                    self.dragging_entity.unwrap(),
                    self.dragging_start.unwrap(),
                    self.dragging_end.unwrap(),
                ));
                self.dragging_entity = None;
                self.dragging_start = None;
                self.dragging_end = None;

                // start next drag
                self.dragging_entity = Some(transform_entity);
                self.dragging_start = Some(old_transform);
                self.dragging_end = Some(new_transform);
            }
        } else {
            // not dragging an entity
            self.dragging_entity = Some(transform_entity);
            self.dragging_start = Some(old_transform);
            self.dragging_end = Some(new_transform);
        }
    }

    pub fn take_drags(&mut self) -> Option<Vec<(Entity, Transform, Transform)>> {
        if self.dragging_entity.is_some() {
            // finish current drag
            self.drags.push((
                self.dragging_entity.unwrap(),
                self.dragging_start.unwrap(),
                self.dragging_end.unwrap(),
            ));
            self.dragging_entity = None;
            self.dragging_start = None;
            self.dragging_end = None;
        }

        if self.drags.is_empty() {
            return None;
        } else {
            let drags = std::mem::take(&mut self.drags);
            return Some(drags);
        }
    }

    pub fn get_bone_transform(
        &self,
        vertex_3d_q: &Query<&Vertex3d>,
        edge_angle_q: &Query<&EdgeAngle>,
        ntc_entity: &Entity,
    ) -> Option<Transform> {
        let ntc_data = self.transform_entities.get(ntc_entity)?;
        return ntc_data.get_bone_transform(vertex_3d_q, edge_angle_q);
    }

    pub(crate) fn get_rotation_edge_3d_entity(
        &self,
        net_transform_entity: &Entity,
    ) -> Option<Entity> {
        let net_transform_data = self.transform_entities.get(net_transform_entity)?;
        Some(net_transform_data.rotation_entity_edge_3d)
    }

    pub(crate) fn get_edge_2d_entity(&self, net_transform_entity: &Entity) -> Option<Entity> {
        let net_transform_data = self.transform_entities.get(net_transform_entity)?;
        let (edge_2d_entity, _, _, _) = net_transform_data.skel_entities?;
        Some(edge_2d_entity)
    }

    fn net_transform_3d_vertices(&self, file_entity: &Entity) -> Vec<Entity> {
        let mut vertices = Vec::new();
        if let Some(net_transform_entities) = self.file_to_transform_entities.get(file_entity) {
            for net_transform_entity in net_transform_entities.iter() {
                let data = self.transform_entities.get(net_transform_entity).unwrap();
                vertices.push(data.translation_entity_3d);
                vertices.push(data.rotation_entity_vert_3d);
                vertices.push(data.scale_x_entity_3d);
                vertices.push(data.scale_y_entity_3d);
                vertices.push(data.scale_z_entity_3d);
            }
        }
        vertices
    }

    pub fn net_transform_2d_vertices(&self, file_entity: &Entity) -> Vec<Entity> {
        let mut vertices = Vec::new();
        if let Some(net_transform_entities) = self.file_to_transform_entities.get(file_entity) {
            for net_transform_entity in net_transform_entities.iter() {
                let data = self.transform_entities.get(net_transform_entity).unwrap();
                vertices.push(data.translation_entity_2d);
                vertices.push(data.rotation_entity_vert_2d);
                vertices.push(data.scale_x_entity_2d);
                vertices.push(data.scale_y_entity_2d);
                vertices.push(data.scale_z_entity_2d);
            }
        }
        vertices
    }

    fn net_transform_rotation_edge_3d_entities(&self, file_entity: &Entity) -> Vec<Entity> {
        let mut output = Vec::new();
        if let Some(net_transform_entities) = self.file_to_transform_entities.get(file_entity) {
            for net_transform_entity in net_transform_entities.iter() {
                let data = self.transform_entities.get(net_transform_entity).unwrap();
                output.push(data.rotation_entity_edge_3d);
            }
        }
        output
    }

    pub(crate) fn net_transform_rotation_edge_2d_entities(
        &self,
        file_entity: &Entity,
    ) -> Vec<Entity> {
        let mut output = Vec::new();
        if let Some(net_transform_entities) = self.file_to_transform_entities.get(file_entity) {
            for net_transform_entity in net_transform_entities.iter() {
                let data = self.transform_entities.get(net_transform_entity).unwrap();
                output.push(data.rotation_entity_edge_2d);
            }
        }
        output
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
