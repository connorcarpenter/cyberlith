use std::collections::HashMap;

use bevy_ecs::{
    entity::Entity,
    prelude::{Commands, World},
    system::{Query, SystemState},
};
use bevy_log::info;

use naia_bevy_server::{CommandsExt, ReplicationConfig, Server};

use asset_io::json::{AssetId, ModelFile};

use editor_proto::{
    components::{
        FileExtension, FileType, NetTransform, NetTransformEntityType, OwnedByFile, ShapeName,
        SkinOrSceneEntity,
    },
    resources::FileKey,
};

use crate::{
    files::{
        add_file_dependency, convert_from_quat, convert_from_component_type, convert_into_quat,
        convert_into_component_type, FileWriter,
    },
    resources::{ContentEntityData, Project},
};

// Writer
pub struct ModelWriter;

impl ModelWriter {
    fn world_to_data(
        &self,
        world: &mut World,
        project: &Project,
        content_entities: &HashMap<Entity, ContentEntityData>,
    ) -> ModelFile {
        let working_file_entries = project.working_file_entries();

        let mut skel_dependency_key_opt = None;
        let mut skin_dependencies = Vec::new();
        let mut skin_dependency_to_index = HashMap::new();
        let mut net_transform_entities = Vec::new();

        for (content_entity, content_data) in content_entities {
            match content_data {
                ContentEntityData::Dependency(dependency_key) => {
                    let dependency_entity = project.file_entity(dependency_key).unwrap();
                    let dependency_value = working_file_entries.get(dependency_key).unwrap();
                    let dependency_file_ext = dependency_value.extension().unwrap();
                    match dependency_file_ext {
                        FileExtension::Skel => {
                            skel_dependency_key_opt = Some(dependency_key);
                        }
                        FileExtension::Skin => {
                            let skin_index = skin_dependencies.len() as u16;
                            skin_dependency_to_index.insert(dependency_entity, skin_index);
                            info!(
                                "writing skin index for entity: `{:?}`, skin_index: `{}`",
                                dependency_entity, skin_index
                            );
                            skin_dependencies.push((dependency_key, NetTransformEntityType::Skin));
                        }
                        FileExtension::Scene => {
                            let skin_index = skin_dependencies.len() as u16;
                            skin_dependency_to_index.insert(dependency_entity, skin_index);
                            skin_dependencies.push((dependency_key, NetTransformEntityType::Scene));
                        }
                        _ => {
                            panic!("model file should depend on a single .skel file & potentially many .skin or .scene files");
                        }
                    }
                }
                ContentEntityData::NetTransform => {
                    net_transform_entities.push(*content_entity);
                }
                _ => {
                    panic!("model should not have this content entity type");
                }
            }
        }

        let mut actions = Vec::new();

        // Write Skel Dependency
        if let Some(dependency_key) = skel_dependency_key_opt {
            info!("writing skel dependency: {}", dependency_key.full_path());
            actions.push(ModelAction::SkelFile(
                dependency_key.path().to_string(),
                dependency_key.name().to_string(),
            ));
        }

        // Write Skin Dependencies
        for (dependency_key, dependency_type) in skin_dependencies {
            info!(
                "writing skin/scene dependency: {}",
                dependency_key.full_path()
            );
            actions.push(ModelAction::SkinOrSceneFile(
                dependency_key.path().to_string(),
                dependency_key.name().to_string(),
                convert_into_component_type(dependency_type),
            ));
        }

        // Write NetTransforms
        for net_transform_entity in net_transform_entities {
            let mut system_state: SystemState<(
                Server,
                Query<(&NetTransform, &SkinOrSceneEntity, &ShapeName)>,
            )> = SystemState::new(world);
            let (server, transform_q) = system_state.get_mut(world);
            let Ok((transform, skin_or_scene_entity, shape_name)) = transform_q.get(net_transform_entity) else {
                panic!("Error getting net transform");
            };
            let skin_entity: Entity = skin_or_scene_entity.value.get(&server).unwrap();
            info!(
                "in writing net transform, skin entity is: `{:?}`",
                skin_entity
            );
            let Some(skin_index) = skin_dependency_to_index.get(&skin_entity) else {
                panic!("skin entity not found in skin_dependency_to_index: `{:?}`", skin_entity);
            };

            let bone_name = (*shape_name.value).clone();
            let translation_x = transform.translation_x();
            let translation_y = transform.translation_y();
            let translation_z = transform.translation_z();
            let scale_x = transform.scale_x();
            let scale_y = transform.scale_y();
            let scale_z = transform.scale_z();
            let rotation = transform.get_rotation_serde();

            info!(
                "writing action for net transform for bone: `{}`, skin index is: {}",
                bone_name, skin_index
            );
            actions.push(ModelAction::NetTransform(
                *skin_index,
                bone_name,
                translation_x,
                translation_y,
                translation_z,
                scale_x,
                scale_y,
                scale_z,
                convert_into_quat(rotation),
            ));
        }

        actions
    }
}

impl FileWriter for ModelWriter {
    fn write(
        &self,
        world: &mut World,
        project: &Project,
        content_entities: &HashMap<Entity, ContentEntityData>,
        asset_id: &AssetId,
    ) -> Box<[u8]> {
        let data = self.world_to_data(world, project, content_entities);
        data.write(asset_id)
    }

    fn write_new_default(&self, asset_id: &AssetId,) -> Box<[u8]> {
        let data = ModelFile::new();
        data.write(asset_id)
    }
}

// Reader
pub struct ModelReader;

impl ModelReader {
    fn data_to_world(
        world: &mut World,
        project: &mut Project,
        file_key: &FileKey,
        file_entity: &Entity,
        data: &ModelFile,
    ) -> HashMap<Entity, ContentEntityData> {
        let mut output = HashMap::new();

        let mut system_state: SystemState<(Commands, Server)> = SystemState::new(world);
        let (mut commands, mut server) = system_state.get_mut(world);

        let mut skin_files = Vec::new();

        for action in actions {
            match action {
                ModelAction::SkelFile(path, file_name) => {
                    let (new_dependency_entity, _dependency_file_entity, dependency_file_key) =
                        add_file_dependency(
                            project,
                            file_key,
                            file_entity,
                            &mut commands,
                            &mut server,
                            FileExtension::Skel,
                            &path,
                            &file_name,
                        );
                    output.insert(
                        new_dependency_entity,
                        ContentEntityData::new_dependency(dependency_file_key),
                    );
                }
                ModelAction::SkinOrSceneFile(path, file_name, file_type) => {
                    let dependency_file_ext = match file_type {
                        FileTransformEntityType::Skin => FileExtension::Skin,
                        FileTransformEntityType::Scene => FileExtension::Scene,
                    };
                    let (new_dependency_entity, dependency_file_entity, dependency_file_key) =
                        add_file_dependency(
                            project,
                            file_key,
                            file_entity,
                            &mut commands,
                            &mut server,
                            dependency_file_ext,
                            &path,
                            &file_name,
                        );
                    output.insert(
                        new_dependency_entity,
                        ContentEntityData::new_dependency(dependency_file_key),
                    );

                    info!(
                        "reading new skin file at index: {}, entity: `{:?}`",
                        skin_files.len(),
                        dependency_file_entity
                    );
                    skin_files.push((file_type, dependency_file_entity));
                }
                ModelAction::NetTransform(
                    skin_index,
                    vertex_name,
                    translation_x,
                    translation_y,
                    translation_z,
                    scale_x,
                    scale_y,
                    scale_z,
                    rotation,
                ) => {
                    let Some((skin_or_scene_type, skin_or_scene_entity)) = skin_files.get(skin_index as usize) else {
                        panic!("skin index out of bounds");
                    };
                    let mut skin_or_scene_component =
                        SkinOrSceneEntity::new(convert_from_component_type(*skin_or_scene_type));
                    skin_or_scene_component
                        .value
                        .set(&server, skin_or_scene_entity);
                    info!("reading net transform for bone: `{}`, into world. skin index: {} -> entity: `{:?}`",
                        vertex_name.clone(),
                        skin_index,
                        skin_or_scene_entity);

                    let mut owning_file_component = OwnedByFile::new();
                    owning_file_component
                        .file_entity
                        .set(&mut server, file_entity);

                    let net_transform_entity = commands
                        .spawn_empty()
                        .enable_replication(&mut server)
                        .configure_replication(ReplicationConfig::Delegated)
                        .insert(NetTransform::new(
                            convert_from_quat(rotation),
                            translation_x as f32,
                            translation_y as f32,
                            translation_z as f32,
                            scale_x,
                            scale_y,
                            scale_z,
                        ))
                        .insert(ShapeName::new(vertex_name.clone()))
                        .insert(skin_or_scene_component)
                        .insert(owning_file_component)
                        .insert(FileType::new(FileExtension::Model))
                        .id();

                    output.insert(net_transform_entity, ContentEntityData::new_net_transform());
                }
            }
        }

        system_state.apply(world);

        output
    }

    pub fn read(
        &self,
        world: &mut World,
        project: &mut Project,
        file_key: &FileKey,
        file_entity: &Entity,
        bytes: &Box<[u8]>,
    ) -> HashMap<Entity, ContentEntityData> {

        let Ok((meta, data)) = ModelFile::read(bytes) else {
            panic!("Error reading .model file");
        };

        if meta.schema_version() != ModelFile::CURRENT_SCHEMA_VERSION {
            panic!("Invalid schema version");
        }

        let result = Self::data_to_world(world, project, file_key, file_entity, &data);

        result
    }
}
