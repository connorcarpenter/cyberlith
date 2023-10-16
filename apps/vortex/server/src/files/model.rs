use std::collections::HashMap;

use bevy_ecs::{
    entity::Entity,
    prelude::{Commands, World},
    system::{Query, Res, SystemState},
};
use bevy_log::info;

use naia_bevy_server::{
    BitReader, CommandsExt, FileBitWriter, ReplicationConfig, Serde, SerdeErr, Server,
};

use vortex_proto::{components::FileExtension, resources::FileKey};

use crate::{
    files::{add_file_dependency, FileWriter},
    resources::{ContentEntityData, Project},
};

// Actions
#[derive(Clone)]
enum ModelAction {
    // path, file_name
    SkelFile(String, String),
}

#[derive(Serde, Clone, PartialEq)]
enum ModelActionType {
    SkelFile,
    None,
}

// Writer
pub struct ModelWriter;

impl ModelWriter {
    fn world_to_actions(
        &self,
        world: &mut World,
        project: &Project,
        content_entities: &HashMap<Entity, ContentEntityData>,
    ) -> Vec<ModelAction> {
        let working_file_entries = project.working_file_entries();

        let mut skel_dependency_key_opt = None;

        for (content_entity, content_data) in content_entities {
            match content_data {
                ContentEntityData::Dependency(dependency_key) => {
                    let dependency_value = working_file_entries.get(dependency_key).unwrap();
                    let dependency_file_ext = dependency_value.extension().unwrap();
                    match dependency_file_ext {
                        FileExtension::Skel => {
                            skel_dependency_key_opt = Some(dependency_key);
                        }
                        _ => {
                            panic!("model file should depend on a single .skel file & potentially many .skin or .scene files");
                        }
                    }
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

        actions
    }

    fn write_from_actions(&self, actions: Vec<ModelAction>) -> Box<[u8]> {
        let mut bit_writer = FileBitWriter::new();

        for action in actions {
            match action {
                ModelAction::SkelFile(path, file_name) => {
                    ModelActionType::SkelFile.ser(&mut bit_writer);
                    path.ser(&mut bit_writer);
                    file_name.ser(&mut bit_writer);
                }
            }
        }

        // continue bit
        ModelActionType::None.ser(&mut bit_writer);

        bit_writer.to_bytes()
    }
}

impl FileWriter for ModelWriter {
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
pub struct ModelReader;

impl ModelReader {
    fn read_to_actions(bit_reader: &mut BitReader) -> Result<Vec<ModelAction>, SerdeErr> {
        let mut actions = Vec::new();

        loop {
            let action_type = ModelActionType::de(bit_reader)?;

            match action_type {
                ModelActionType::SkelFile => {
                    let path = String::de(bit_reader)?;
                    let file_name = String::de(bit_reader)?;
                    actions.push(ModelAction::SkelFile(path, file_name));
                }
                ModelActionType::None => {
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
        actions: Vec<ModelAction>,
    ) -> HashMap<Entity, ContentEntityData> {
        let mut output = HashMap::new();

        let mut system_state: SystemState<(Commands, Server)> = SystemState::new(world);
        let (mut commands, mut server) = system_state.get_mut(world);

        for action in actions {
            match action {
                ModelAction::SkelFile(palette_path, palette_file_name) => {
                    let (new_entity, new_file_key) = add_file_dependency(
                        project,
                        file_key,
                        file_entity,
                        &mut commands,
                        &mut server,
                        FileExtension::Skel,
                        &palette_path,
                        &palette_file_name,
                    );
                    output.insert(new_entity, ContentEntityData::new_dependency(new_file_key));
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
            panic!("Error reading .model file");
        };

        let result = Self::actions_to_world(world, project, file_key, file_entity, actions);

        result
    }
}
