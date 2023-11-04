use std::collections::HashMap;

use bevy_ecs::{
    entity::Entity,
    prelude::{Commands, World},
    system::{Query, SystemState},
};
use bevy_log::info;

use naia_bevy_server::{
    BitReader, CommandsExt, FileBitWriter, ReplicationConfig, Serde, SerdeErr, Server,
    SignedVariableInteger, UnsignedVariableInteger,
};

use vortex_proto::{
    components::{FileType, OwnedByFile, SkinOrSceneEntity, FileExtension, NetTransform, NetTransformEntityType},
    resources::FileKey,
    SerdeQuat,
};

use crate::{
    files::{add_file_dependency, FileWriter},
    resources::{ContentEntityData, Project},
};

// Actions
#[derive(Clone)]
enum SceneAction {
    SkinOrSceneFile(String, String, NetTransformEntityType),
    NetTransform(u16, i16, i16, i16, f32, f32, f32, SerdeQuat),
}

#[derive(Serde, Clone, PartialEq)]
enum SceneActionType {
    SkinOrSceneFile,
    NetTransform,
    None,
}

pub type TranslationSerdeInt = SignedVariableInteger<4>;
pub type ScaleSerdeInt = UnsignedVariableInteger<4>;

// Writer
pub struct SceneWriter;

impl SceneWriter {
    fn world_to_actions(
        &self,
        world: &mut World,
        project: &Project,
        content_entities: &HashMap<Entity, ContentEntityData>,
    ) -> Vec<SceneAction> {
        let working_file_entries = project.working_file_entries();

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

        let mut actions = Vec::new();

        // Write Skin/Scene Dependencies
        for (dependency_key, dependency_type) in skin_dependencies {
            info!(
                "writing skin/scene dependency: {}",
                dependency_key.full_path()
            );
            actions.push(SceneAction::SkinOrSceneFile(
                dependency_key.path().to_string(),
                dependency_key.name().to_string(),
                dependency_type,
            ));
        }

        // Write NetTransforms
        for net_transform_entity in net_transform_entities {
            let mut system_state: SystemState<(Server, Query<(&NetTransform, &SkinOrSceneEntity)>)> =
                SystemState::new(world);
            let (server, transform_q) = system_state.get_mut(world);
            let Ok((transform, skin_or_scene_entity)) = transform_q.get(net_transform_entity) else {
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

            let translation_x = transform.translation_x();
            let translation_y = transform.translation_y();
            let translation_z = transform.translation_z();
            let scale_x = transform.scale_x();
            let scale_y = transform.scale_y();
            let scale_z = transform.scale_z();
            let rotation = transform.get_rotation_serde();

            info!(
                "writing action for net transform. skin index is: {}",
                skin_index
            );
            actions.push(SceneAction::NetTransform(
                *skin_index,
                translation_x,
                translation_y,
                translation_z,
                scale_x,
                scale_y,
                scale_z,
                rotation,
            ));
        }

        actions
    }

    fn write_from_actions(&self, actions: Vec<SceneAction>) -> Box<[u8]> {
        let mut bit_writer = FileBitWriter::new();

        for action in actions {
            match action {
                SceneAction::SkinOrSceneFile(path, file_name, file_type) => {
                    SceneActionType::SkinOrSceneFile.ser(&mut bit_writer);
                    path.ser(&mut bit_writer);
                    file_name.ser(&mut bit_writer);
                    file_type.ser(&mut bit_writer);
                }
                SceneAction::NetTransform(
                    skin_index,
                    translation_x,
                    translation_y,
                    translation_z,
                    scale_x,
                    scale_y,
                    scale_z,
                    rotation,
                ) => {
                    SceneActionType::NetTransform.ser(&mut bit_writer);

                    UnsignedVariableInteger::<6>::new(skin_index).ser(&mut bit_writer);

                    let translation_x = TranslationSerdeInt::new(translation_x);
                    let translation_y = TranslationSerdeInt::new(translation_y);
                    let translation_z = TranslationSerdeInt::new(translation_z);

                    translation_x.ser(&mut bit_writer);
                    translation_y.ser(&mut bit_writer);
                    translation_z.ser(&mut bit_writer);

                    let scale_x = ScaleSerdeInt::new((scale_x * 100.0) as u32);
                    let scale_y = ScaleSerdeInt::new((scale_y * 100.0) as u32);
                    let scale_z = ScaleSerdeInt::new((scale_z * 100.0) as u32);

                    scale_x.ser(&mut bit_writer);
                    scale_y.ser(&mut bit_writer);
                    scale_z.ser(&mut bit_writer);

                    rotation.ser(&mut bit_writer);
                }
            }
        }

        // continue bit
        SceneActionType::None.ser(&mut bit_writer);

        bit_writer.to_bytes()
    }
}

impl FileWriter for SceneWriter {
    fn write(
        &self,
        world: &mut World,
        project: &Project,
        content_entities: &HashMap<Entity, ContentEntityData>,
    ) -> Box<[u8]> {
        let actions = self.world_to_actions(world, project, content_entities);
        self.write_from_actions(actions)
    }

    fn write_new_default(&self) -> Box<[u8]> {
        let actions = Vec::new();

        self.write_from_actions(actions)
    }
}

// Reader
pub struct SceneReader;

impl SceneReader {
    fn read_to_actions(bit_reader: &mut BitReader) -> Result<Vec<SceneAction>, SerdeErr> {
        let mut actions = Vec::new();

        loop {
            let action_type = SceneActionType::de(bit_reader)?;

            match action_type {
                SceneActionType::SkinOrSceneFile => {
                    let path = String::de(bit_reader)?;
                    let file_name = String::de(bit_reader)?;
                    let file_type = NetTransformEntityType::de(bit_reader)?;
                    actions.push(SceneAction::SkinOrSceneFile(path, file_name, file_type));
                }
                SceneActionType::NetTransform => {
                    let skin_index: u16 = UnsignedVariableInteger::<6>::de(bit_reader)?.to();

                    let translation_x = TranslationSerdeInt::de(bit_reader)?.to();
                    let translation_y = TranslationSerdeInt::de(bit_reader)?.to();
                    let translation_z = TranslationSerdeInt::de(bit_reader)?.to();

                    let scale_x: u32 = ScaleSerdeInt::de(bit_reader)?.to();
                    let scale_y: u32 = ScaleSerdeInt::de(bit_reader)?.to();
                    let scale_z: u32 = ScaleSerdeInt::de(bit_reader)?.to();
                    let scale_x = (scale_x as f32) / 100.0;
                    let scale_y = (scale_y as f32) / 100.0;
                    let scale_z = (scale_z as f32) / 100.0;

                    let rotation = SerdeQuat::de(bit_reader)?;

                    info!("reading net transform into action",);

                    actions.push(SceneAction::NetTransform(
                        skin_index,
                        translation_x,
                        translation_y,
                        translation_z,
                        scale_x,
                        scale_y,
                        scale_z,
                        rotation,
                    ));
                }
                SceneActionType::None => {
                    break;
                }
            }
        }

        Ok(actions)
    }

    fn actions_to_world(
        world: &mut World,
        project: &mut Project,
        file_key: &FileKey,
        file_entity: &Entity,
        actions: Vec<SceneAction>,
    ) -> HashMap<Entity, ContentEntityData> {
        let mut output = HashMap::new();

        let mut system_state: SystemState<(Commands, Server)> = SystemState::new(world);
        let (mut commands, mut server) = system_state.get_mut(world);

        let mut skin_files = Vec::new();

        for action in actions {
            match action {
                SceneAction::SkinOrSceneFile(path, file_name, file_type) => {
                    let dependency_file_ext = match file_type {
                        NetTransformEntityType::Uninit => panic!("shouldn't happen"),
                        NetTransformEntityType::Skin => FileExtension::Skin,
                        NetTransformEntityType::Scene => FileExtension::Scene,
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
                SceneAction::NetTransform(
                    skin_index,
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
                    let mut skin_or_scene_component = SkinOrSceneEntity::new(*skin_or_scene_type);
                    skin_or_scene_component.value.set(&server, skin_or_scene_entity);
                    info!(
                        "reading net transform into world. skin index: {} -> entity: `{:?}`",
                        skin_index,
                        skin_or_scene_entity
                    );

                    let mut owning_file_component = OwnedByFile::new();
                    owning_file_component.file_entity.set(&mut server, file_entity);

                    let net_transform_entity = commands
                        .spawn_empty()
                        .enable_replication(&mut server)
                        .configure_replication(ReplicationConfig::Delegated)
                        .insert(NetTransform::new(
                            rotation,
                            translation_x as f32,
                            translation_y as f32,
                            translation_z as f32,
                            scale_x,
                            scale_y,
                            scale_z,
                        ))
                        .insert(skin_or_scene_component)
                        .insert(owning_file_component)
                        .insert(FileType::new(FileExtension::Scene))
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
        let mut bit_reader = BitReader::new(bytes);

        let Ok(actions) = Self::read_to_actions(&mut bit_reader) else {
            panic!("Error reading .scene file");
        };

        let result = Self::actions_to_world(world, project, file_key, file_entity, actions);

        result
    }
}
