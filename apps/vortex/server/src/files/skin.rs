use std::collections::HashMap;

use bevy_ecs::{
    entity::Entity,
    prelude::{Commands, World},
    system::{Query, ResMut, SystemState},
};
use bevy_log::info;

use naia_bevy_server::{
    BitReader, CommandsExt, FileBitWriter, ReplicationConfig, Serde, SerdeErr, Server,
};

use crate::{
    files::FileWriter,
    resources::{ContentEntityData, Project},
};

// Actions
#[derive(Clone)]
enum SkinAction {
    //
    None,
}

#[derive(Serde, Clone, PartialEq)]
enum SkinActionType {
    None,
}

// Writer
pub struct SkinWriter;

impl SkinWriter {
    fn world_to_actions(&self, world: &mut World) -> Vec<Option<SkinAction>> {

        let mut actions = Vec::new();

        actions
    }

    fn write_from_actions(&self, actions: Vec<Option<SkinAction>>) -> Box<[u8]> {
        let mut bit_writer = FileBitWriter::new();

        bit_writer.to_bytes()
    }
}

impl FileWriter for SkinWriter {
    fn write(
        &self,
        world: &mut World,
        _project: &Project,
        _content_entities: &HashMap<Entity, ContentEntityData>,
    ) -> Box<[u8]> {
        let actions = self.world_to_actions(world);
        self.write_from_actions(actions)
    }

    fn write_new_default(&self) -> Box<[u8]> {
        let mut actions = Vec::new();

        self.write_from_actions(actions)
    }
}

// Reader
pub struct SkinReader;

impl SkinReader {
    fn read_to_actions(bit_reader: &mut BitReader) -> Result<Vec<SkinAction>, SerdeErr> {
        let mut actions = Vec::new();

        Ok(actions)
    }

    fn actions_to_world(
        world: &mut World,
        actions: Vec<SkinAction>,
    ) -> HashMap<Entity, ContentEntityData> {
        let mut output = HashMap::new();

        output
    }

    pub fn read(
        &self,
        world: &mut World,
        bytes: &Box<[u8]>,
    ) -> HashMap<Entity, ContentEntityData> {
        let mut bit_reader = BitReader::new(bytes);

        let Ok(actions) = Self::read_to_actions(&mut bit_reader) else {
            panic!("Error reading .skin file");
        };

        let result = Self::actions_to_world(world, actions);

        result
    }
}
