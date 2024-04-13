use std::collections::HashMap;

use bevy_ecs::{
    entity::Entity,
    prelude::{Commands, World},
    system::{Query, SystemState},
};
use logging::info;

use naia_bevy_server::{CommandsExt, ReplicationConfig, Server};

use asset_id::AssetId;
use asset_serde::json::{FileComponentType, SceneJson};
use math::Quat;

use editor_proto::{
    components::{
        FileExtension, FileType, NetTransform, NetTransformEntityType, OwnedByFile,
        SkinOrSceneEntity,
    },
    resources::FileKey,
};

use crate::{
    files::{
        add_file_dependency, convert_from_component_type, convert_into_component_type, FileWriter,
    },
    resources::{ContentEntityData, Project},
};

// Writer
pub struct SceneWriter;

impl SceneWriter {
    fn world_to_data(
        &self,
        world: &mut World,
        project: &Project,
        content_entities: &HashMap<Entity, ContentEntityData>,
    ) -> SceneJson {
        let working_file_entries = project.working_file_entries();

        let mut skin_dependencies = Vec::new();
        let mut component_dependency_to_index = HashMap::new();
        let mut net_transform_entities = Vec::new();

        for (content_entity, content_data) in content_entities {
            match content_data {
                ContentEntityData::Dependency(dependency_key) => {
                    let dependency_entity = project.file_entity(dependency_key).unwrap();
                    let dependency_value = working_file_entries.get(dependency_key).unwrap();
                    let dependency_file_ext = dependency_value.extension().unwrap();
                    match dependency_file_ext {
                        FileExtension::Skin => {
                            let skin_index = skin_dependencies.len() as u16;
                            component_dependency_to_index.insert(dependency_entity, skin_index);
                            info!(
                                "writing skin index for entity: `{:?}`, skin_index: `{}`",
                                dependency_entity, skin_index
                            );
                            skin_dependencies.push((dependency_key, NetTransformEntityType::Skin));
                        }
                        FileExtension::Scene => {
                            let skin_index = skin_dependencies.len() as u16;
                            component_dependency_to_index.insert(dependency_entity, skin_index);
                            skin_dependencies.push((dependency_key, NetTransformEntityType::Scene));
                        }
                        _ => {
                            panic!("scene file should depend on potentially many .skin or .scene files");
                        }
                    }
                }
                ContentEntityData::NetTransform => {
                    net_transform_entities.push(*content_entity);
                }
                _ => {
                    panic!("scene should not have this content entity type");
                }
            }
        }

        let mut output = SceneJson::new();

        // Write Component Dependencies
        for (dependency_key, dependency_type) in skin_dependencies {
            info!(
                "writing skin/scene dependency: {}",
                dependency_key.full_path()
            );
            let dependency_asset_id = project.asset_id(dependency_key).unwrap();
            let kind = convert_into_component_type(dependency_type);
            output.add_component(dependency_asset_id, kind);
        }

        // Write NetTransforms
        for net_transform_entity in net_transform_entities {
            let mut system_state: SystemState<(
                Server,
                Query<(&NetTransform, &SkinOrSceneEntity)>,
            )> = SystemState::new(world);
            let (server, transform_q) = system_state.get_mut(world);
            let Ok((transform, component_entity_prop)) = transform_q.get(net_transform_entity)
            else {
                panic!("Error getting net transform");
            };
            let component_entity: Entity = component_entity_prop.value.get(&server).unwrap();
            info!(
                "in writing net transform, component entity is: `{:?}`",
                component_entity
            );
            let Some(component_id) = component_dependency_to_index.get(&component_entity) else {
                panic!(
                    "skin entity not found in component_dependency_to_index: `{:?}`",
                    component_entity
                );
            };

            let translation_x = transform.translation_x();
            let translation_y = transform.translation_y();
            let translation_z = transform.translation_z();
            let scale_x = transform.scale_x();
            let scale_y = transform.scale_y();
            let scale_z = transform.scale_z();
            let rotation = transform.get_rotation_serde();

            info!(
                "writing action for net transform. component index is: {}",
                component_id
            );
            output.add_transform(
                *component_id,
                translation_x,
                translation_y,
                translation_z,
                scale_x,
                scale_y,
                scale_z,
                rotation.0.x,
                rotation.0.y,
                rotation.0.z,
                rotation.0.w,
            );
        }

        output
    }
}

impl FileWriter for SceneWriter {
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

    fn write_new_default(&self, asset_id: &AssetId) -> Box<[u8]> {
        let data = SceneJson::new();
        data.write(asset_id)
    }
}

// Reader
pub struct SceneReader;

impl SceneReader {
    fn data_to_world(
        world: &mut World,
        project: &mut Project,
        file_key: &FileKey,
        file_entity: &Entity,
        data: &SceneJson,
    ) -> HashMap<Entity, ContentEntityData> {
        let mut output = HashMap::new();

        let mut system_state: SystemState<(Commands, Server)> = SystemState::new(world);
        let (mut commands, mut server) = system_state.get_mut(world);

        let mut components = Vec::new();

        //

        for component in data.get_components() {
            let file_type = component.kind();
            let asset_id = component.asset_id();
            let dependency_file_ext = match file_type {
                FileComponentType::Skin => FileExtension::Skin,
                FileComponentType::Scene => FileExtension::Scene,
            };
            let dependency_file_key = project.file_key_from_asset_id(&asset_id).unwrap();
            let (new_dependency_entity, dependency_file_entity) = add_file_dependency(
                project,
                file_key,
                file_entity,
                &mut commands,
                &mut server,
                dependency_file_ext,
                &dependency_file_key,
            );
            output.insert(
                new_dependency_entity,
                ContentEntityData::new_dependency(dependency_file_key),
            );

            info!(
                "reading new component at index: {}, entity: `{:?}`",
                components.len(),
                dependency_file_entity
            );
            components.push((file_type, dependency_file_entity));
        }

        for transform in data.get_transforms() {
            let component_index = transform.component_id();
            let position = transform.position();
            let scale = transform.scale();
            let rotation = transform.rotation();
            let Some((component_type, component_entity)) = components.get(component_index as usize)
            else {
                panic!("skin index out of bounds");
            };
            let mut skin_or_scene_component =
                SkinOrSceneEntity::new(convert_from_component_type(*component_type));
            skin_or_scene_component.value.set(&server, component_entity);
            info!(
                "reading net transform into world. skin index: {} -> entity: `{:?}`",
                component_index, component_entity
            );

            let mut owning_file_component = OwnedByFile::new();
            owning_file_component
                .file_entity
                .set(&mut server, file_entity);

            let net_transform_entity = commands
                .spawn_empty()
                .enable_replication(&mut server)
                .configure_replication(ReplicationConfig::Delegated)
                .insert(NetTransform::new(
                    math::SerdeQuat::from(Quat::from_xyzw(
                        rotation.x(),
                        rotation.y(),
                        rotation.z(),
                        rotation.w(),
                    )),
                    position.x() as f32,
                    position.y() as f32,
                    position.z() as f32,
                    scale.x(),
                    scale.y(),
                    scale.z(),
                ))
                .insert(skin_or_scene_component)
                .insert(owning_file_component)
                .insert(FileType::new(FileExtension::Scene))
                .id();

            output.insert(net_transform_entity, ContentEntityData::new_net_transform());
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
        let Ok((meta, data)) = SceneJson::read(bytes) else {
            panic!("Error reading .scene file");
        };

        if meta.schema_version() != SceneJson::CURRENT_SCHEMA_VERSION {
            panic!("Invalid schema version");
        }

        let result = Self::data_to_world(world, project, file_key, file_entity, &data);

        result
    }
}
