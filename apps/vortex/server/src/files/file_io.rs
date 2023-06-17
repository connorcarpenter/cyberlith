use bevy_ecs::{entity::Entity, system::Commands, world::World};

use crate::files::{SkelReader, SkelWriter};

pub trait FileWriter: Send + Sync {
    fn write(&self, world: &mut World, content_entities: &Vec<Entity>) -> Box<[u8]>;
}

pub trait FileReader: Send + Sync {
    fn read(&self, commands: &mut Commands, bytes: &Box<[u8]>) -> Vec<Entity>;
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileExtension {
    Skel,
    Mesh,
    Skin,
    Mask,
}

impl FileExtension {
    pub(crate) fn from_file_name(file_name: &str) -> Self {
        // split file name by '.'
        let split: Vec<_> = file_name.split('.').collect();
        let ext: &str = split.last().unwrap();

        // match file extension to enum
        match ext {
            "skel" => FileExtension::Skel,
            "mesh" => FileExtension::Mesh,
            "skin" => FileExtension::Skin,
            "mask" => FileExtension::Mask,
            _ => panic!("Unknown file extension: {}", ext)
        }
    }

    pub(crate) fn get_reader(&self) -> Box<dyn FileReader> {
        match self {
            FileExtension::Skel => Box::new(SkelReader),
            FileExtension::Mesh => todo!(),
            FileExtension::Skin => todo!(),
            FileExtension::Mask => todo!(),
        }
    }

    pub(crate) fn get_writer(&self) -> Box<dyn FileWriter> {
        match self {
            FileExtension::Skel => Box::new(SkelWriter),
            FileExtension::Mesh => todo!(),
            FileExtension::Skin => todo!(),
            FileExtension::Mask => todo!(),
        }
    }
}