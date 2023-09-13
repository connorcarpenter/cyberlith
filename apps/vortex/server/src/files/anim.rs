use std::collections::HashMap;

use bevy_ecs::{
    entity::Entity,
    prelude::{Commands, World},
};

use naia_bevy_server::{BitReader, Serde, SerdeErr, Server};
use vortex_proto::SerdeQuat;

use crate::{
    files::{FileReadOutput, FileReader, FileWriter, ShapeTypeData},
    resources::{ContentEntityData, ShapeManager},
};

// Actions
enum AnimAction {
    SkelFile(String, String),
    // shape name -> shape_index
    ShapeIndex(String, u32),
    // shape_index -> rotation
    Frame(HashMap<u32, SerdeQuat>, Transition),
}

#[derive(Serde, Clone, PartialEq)]
enum AnimActionType {
    SkelFile,
    ShapeIndex,
    Frame,
}

pub struct Transition {
    pub duration_ms: f32,
    //pub easing: Easing,
}

// Writer
pub struct AnimWriter;

impl AnimWriter {
    fn world_to_actions(
        &self,
        world: &mut World,
        content_entities: &Vec<Entity>,
    ) -> Vec<AnimAction> {
        todo!()
    }

    fn write_from_actions(&self, actions: Vec<AnimAction>) -> Box<[u8]> {
        todo!()
    }
}

impl FileWriter for AnimWriter {
    fn write(
        &self,
        world: &mut World,
        content_entities: &HashMap<Entity, ContentEntityData>,
    ) -> Box<[u8]> {
        todo!()
    }

    fn write_new_default(&self) -> Box<[u8]> {
        let mut default_actions = Vec::new();

        todo!();

        self.write_from_actions(default_actions)
    }
}

// Reader
pub struct AnimReader;

impl AnimReader {
    fn read_to_actions(bit_reader: &mut BitReader) -> Result<Vec<AnimAction>, SerdeErr> {
        todo!()
    }

    fn actions_to_world(
        commands: &mut Commands,
        server: &mut Server,
        actions: Vec<AnimAction>,
    ) -> Result<FileReadOutput, SerdeErr> {
        todo!()
    }
}

impl FileReader for AnimReader {
    fn read(
        &self,
        commands: &mut Commands,
        server: &mut Server,
        bytes: &Box<[u8]>,
    ) -> FileReadOutput {
        todo!()
    }
}

impl AnimReader {
    pub fn post_process_entities() -> HashMap<Entity, ContentEntityData> {
        todo!()
    }
}
