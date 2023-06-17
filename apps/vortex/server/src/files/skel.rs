use bevy_ecs::{entity::Entity, prelude::{Commands, World}};

use crate::files::{FileReader, FileWriter};

// Writer

pub struct SkelWriter;

impl FileWriter for SkelWriter {
    fn write(&self, world: &World, content_entities: &Vec<Entity>) -> Box<[u8]> {
        todo!()
    }
}

// Reader

pub struct SkelReader;

impl FileReader for SkelReader {
    fn read(&self, commands: &mut Commands, bytes: &Box<[u8]>) -> Vec<Entity> {
        todo!()
    }
}