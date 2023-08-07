use std::collections::HashSet;

use bevy_ecs::{entity::Entity, system::Commands, world::World};

use naia_bevy_server::{CommandsExt, RoomKey, Server};
use vortex_proto::{components::OwnedByTab, types::TabId, FileExtension};

use crate::files::{MeshReader, MeshWriter, SkelReader, SkelWriter};

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
            FileExtension::Mesh => MeshReader.read(commands, server, bytes),
            _ => panic!("File extension {:?} not implemented", self),
        }
    }
}

impl FileWriter for FileExtension {
    fn write(&self, world: &mut World, content_entities: &HashSet<Entity>) -> Box<[u8]> {
        match self {
            FileExtension::Skel => SkelWriter.write(world, content_entities),
            FileExtension::Mesh => MeshWriter.write(world, content_entities),
            _ => panic!("File extension {:?} not implemented", self),
        }
    }

    fn write_new_default(&self) -> Box<[u8]> {
        match self {
            FileExtension::Skel => SkelWriter.write_new_default(),
            FileExtension::Mesh => MeshWriter.write_new_default(),
            _ => panic!("File extension {:?} not implemented", self),
        }
    }
}

pub enum FileReadOutput {
    // Skel file, list of vertex entities and an optional parent per
    Skel(Vec<(Entity, Option<Entity>)>),
    // Mesh file, list of entities, list of edges, list of faces
    Mesh(Vec<Entity>, Vec<Entity>, Vec<Entity>),
}

pub fn post_process_networked_entities(
    commands: &mut Commands,
    server: &mut Server,
    room_key: &RoomKey,
    entities: &HashSet<Entity>,
    tab_id: TabId,
    pause_replication: bool,
) {
    for entity in entities.iter() {
        // associate all new Entities with the new Room
        server.room_mut(room_key).add_entity(entity);

        // add tab ownership
        commands.entity(*entity).insert(OwnedByTab::new(tab_id));

        // pause replication if indicated
        if pause_replication {
            commands
                .entity(*entity)
                // call "pause_replication" on all Entities (they will be resumed when tab is selected)
                .pause_replication(server);
        }
    }
}
