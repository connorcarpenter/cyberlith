use std::collections::HashSet;

use bevy_ecs::{entity::Entity, system::Commands, world::World};
use naia_bevy_server::Server;

use vortex_proto::FileExtension;

use crate::files::{SkelReader, SkelWriter};

pub trait FileWriter: Send + Sync {
    fn write(&self, world: &mut World, content_entities: &HashSet<Entity>) -> Box<[u8]>;
    fn write_new_default(&self) -> Box<[u8]>;
}

pub trait FileReader: Send + Sync {
    fn read(
        &self,
        commands: &mut Commands,
        server: &mut Server,
        bytes: &Box<[u8]>,
    ) -> FileReadOutput;
}

impl FileReader for FileExtension {
    fn read(
        &self,
        commands: &mut Commands,
        server: &mut Server,
        bytes: &Box<[u8]>,
    ) -> FileReadOutput {
        match self {
            FileExtension::Skel => SkelReader.read(commands, server, bytes),
            _ => panic!("File extension {:?} not implemented", self),
        }
    }
}

impl FileWriter for FileExtension {
    fn write(&self, world: &mut World, content_entities: &HashSet<Entity>) -> Box<[u8]> {
        match self {
            FileExtension::Skel => SkelWriter.write(world, content_entities),
            _ => panic!("File extension {:?} not implemented", self),
        }
    }

    fn write_new_default(&self) -> Box<[u8]> {
        match self {
            FileExtension::Skel => SkelWriter.write_new_default(),
            _ => panic!("File extension {:?} not implemented", self),
        }
    }
}

pub enum FileReadOutput {
    // Skel file, list of entities and an optional parent per
    Skel(Vec<(Entity, Option<Entity>)>),
}
