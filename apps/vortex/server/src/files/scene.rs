use std::collections::HashMap;

use bevy_ecs::{
    entity::Entity,
    prelude::{Commands, World},
    system::SystemState,
};

use naia_bevy_server::{
    BitReader, FileBitWriter, Serde, SerdeErr, Server,
};

use vortex_proto::resources::FileKey;

use crate::{
    files::FileWriter,
    resources::{ContentEntityData, Project},
};

// Actions
#[derive(Clone)]
enum SceneAction {
    None, // remove this later
}

#[derive(Serde, Clone, PartialEq)]
enum SceneActionType {
    None, // keep this later
}

// Writer
pub struct SceneWriter;

impl SceneWriter {
    fn world_to_actions(
        &self,
        _world: &mut World,
        _project: &Project,
        _content_entities: &HashMap<Entity, ContentEntityData>,
    ) -> Vec<SceneAction> {
        // let working_file_entries = project.working_file_entries();
        //
        // let mut skel_dependency_key_opt = None;
        //
        // for (content_entity, content_data) in content_entities {
        //     match content_data {
        //         _ => {
        //             panic!("model should not have this content entity type");
        //         }
        //     }
        // }

        let actions = Vec::new();

        actions
    }

    fn write_from_actions(&self, actions: Vec<SceneAction>) -> Box<[u8]> {
        let mut bit_writer = FileBitWriter::new();

        for action in actions {
            match action {
                SceneAction::None => {}
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
        let actions = Vec::new();

        loop {
            let action_type = SceneActionType::de(bit_reader)?;

            match action_type {
                SceneActionType::None => {
                    break;
                }
            }
        }

        Ok(actions)
    }

    fn actions_to_world(
        world: &mut World,
        _project: &mut Project,
        _file_key: &FileKey,
        _file_entity: &Entity,
        actions: Vec<SceneAction>,
    ) -> HashMap<Entity, ContentEntityData> {
        let output = HashMap::new();

        let mut system_state: SystemState<(Commands, Server)> = SystemState::new(world);
        let (_commands, _server) = system_state.get_mut(world);

        for action in actions {
            match action {
                SceneAction::None => {}
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
