use std::collections::HashMap;

use bevy_ecs::{
    entity::Entity,
    prelude::{Commands, World},
};
use bevy_log::info;

use naia_bevy_server::{
    BitReader, BitWrite, CommandsExt, FileBitWriter, ReplicationConfig, Serde, SerdeErr, Server,
    UnsignedVariableInteger,
};

use vortex_proto::{
    components::{EntryKind, FileDependency, FileExtension},
    resources::FileKey,
    SerdeQuat,
};

use crate::{
    files::{FileReadOutput, FileReader, FileWriter},
    resources::{ContentEntityData, Project},
};

// Actions
enum AnimAction {
    // path, file_name
    SkelFile(String, String),
    // shape name -> shape_index
    ShapeIndex(String),
    // shape_index -> rotation
    Frame(HashMap<u32, SerdeQuat>, Transition),
}

#[derive(Serde, Clone, PartialEq)]
enum AnimActionType {
    SkelFile,
    ShapeIndex,
    Frame,
    None,
}

#[derive(Clone, PartialEq)]
pub struct Transition {
    pub duration_5ms: u16,
    //pub easing: Easing,
}

impl Serde for Transition {
    fn ser(&self, writer: &mut dyn BitWrite) {
        UnsignedVariableInteger::<7>::from(self.duration_5ms).ser(writer);
    }

    fn de(reader: &mut BitReader) -> Result<Self, SerdeErr> {
        let duration_5ms: u16 = UnsignedVariableInteger::<7>::de(reader)?.to();
        Ok(Self { duration_5ms })
    }

    fn bit_length(&self) -> u32 {
        UnsignedVariableInteger::<7>::from(self.duration_5ms).bit_length()
    }
}

// Writer
pub struct AnimWriter;

impl AnimWriter {
    fn world_to_actions(
        &self,
        _world: &mut World,
        project: &Project,
        file_key: &FileKey,
        content_entities_opt: &Option<HashMap<Entity, ContentEntityData>>,
    ) -> Vec<AnimAction> {

        let working_file_entries = project.working_file_entries();

        let mut skel_dependency_key_opt = None;

        let content_entities = content_entities_opt.as_ref().unwrap();
        for (content_entity, content_data) in content_entities {
            match content_data {
                ContentEntityData::Shape(_) => {
                    panic!("animation should not have any shape entity in it");
                }
                ContentEntityData::Dependency(dependency_ext, dependency_key) => {
                    if *dependency_ext != FileExtension::Skel {
                        panic!("anim file should depend on a single .skel file");
                    }
                    skel_dependency_key_opt = Some(dependency_key);
                }
                ContentEntityData::Frame => {
                    todo!();
                }
                ContentEntityData::Rotation => {
                    todo!();
                }
            }
        }

        let mut actions = Vec::new();

        let Some(dependency_key) = skel_dependency_key_opt else {
            panic!("anim file should depend on a single .skel file");
        };
        let dependency_value = working_file_entries.get(&dependency_key).unwrap();
        if dependency_value.extension().unwrap() != FileExtension::Skel {
            panic!("anim file should depend on a single .skel file");
        }

        let full_skel_path = dependency_key.full_path();
        info!("{} writing dependency: {}", file_key.name(), full_skel_path);
        actions.push(AnimAction::SkelFile(
            dependency_key.path().to_string(),
            dependency_key.name().to_string(),
        ));

        // TODO: poses and such

        actions
    }

    fn write_from_actions(&self, actions: Vec<AnimAction>) -> Box<[u8]> {
        let mut bit_writer = FileBitWriter::new();

        for action in actions {
            match action {
                AnimAction::SkelFile(path, file_name) => {
                    AnimActionType::SkelFile.ser(&mut bit_writer);
                    path.ser(&mut bit_writer);
                    file_name.ser(&mut bit_writer);
                }
                AnimAction::ShapeIndex(name) => {
                    AnimActionType::ShapeIndex.ser(&mut bit_writer);
                    name.ser(&mut bit_writer);
                }
                AnimAction::Frame(poses, transition) => {
                    AnimActionType::Frame.ser(&mut bit_writer);
                    transition.ser(&mut bit_writer);
                    for (shape_index, pose) in poses {
                        // continue bit
                        true.ser(&mut bit_writer);

                        UnsignedVariableInteger::<5>::from(shape_index).ser(&mut bit_writer);
                        pose.ser(&mut bit_writer);
                    }
                    // continue bit
                    false.ser(&mut bit_writer);
                }
            }
        }

        // continue bit
        AnimActionType::None.ser(&mut bit_writer);

        bit_writer.to_bytes()
    }
}

impl FileWriter for AnimWriter {
    fn write(
        &self,
        world: &mut World,
        project: &Project,
        file_key: &FileKey,
        content_entities_opt: &Option<HashMap<Entity, ContentEntityData>>,
    ) -> Box<[u8]> {
        let actions = self.world_to_actions(world, project, file_key, content_entities_opt);
        self.write_from_actions(actions)
    }

    fn write_new_default(&self) -> Box<[u8]> {
        self.write_from_actions(Vec::new())
    }
}

// Reader
pub struct AnimReader;

impl AnimReader {
    fn read_to_actions(bit_reader: &mut BitReader) -> Result<Vec<AnimAction>, SerdeErr> {
        let mut actions = Vec::new();

        loop {
            let action_type = AnimActionType::de(bit_reader)?;
            match action_type {
                AnimActionType::SkelFile => {
                    let path = String::de(bit_reader)?;
                    let file_name = String::de(bit_reader)?;
                    actions.push(AnimAction::SkelFile(path, file_name));
                }
                AnimActionType::ShapeIndex => {
                    let name = String::de(bit_reader)?;
                    actions.push(AnimAction::ShapeIndex(name));
                }
                AnimActionType::Frame => {
                    let transition = Transition::de(bit_reader)?;
                    let mut poses = HashMap::new();
                    loop {
                        let continue_bit = bool::de(bit_reader)?;
                        if !continue_bit {
                            break;
                        }

                        let shape_index: u32 = UnsignedVariableInteger::<5>::de(bit_reader)?.to();
                        let pose = SerdeQuat::de(bit_reader)?;
                        poses.insert(shape_index, pose);
                    }
                    actions.push(AnimAction::Frame(poses, transition));
                }
                AnimActionType::None => {
                    break;
                }
            }
        }

        Ok(actions)
    }

    fn actions_to_world(
        _commands: &mut Commands,
        _server: &mut Server,
        actions: Vec<AnimAction>,
    ) -> Result<FileReadOutput, SerdeErr> {
        let mut skel_path = None;

        for action in actions {
            match action {
                AnimAction::SkelFile(path, file_name) => {
                    skel_path = Some((path, file_name));
                }
                AnimAction::ShapeIndex(_) => {}
                AnimAction::Frame(_, _) => {}
            }
        }

        info!("skel_path: {:?}", skel_path);

        Ok(FileReadOutput::Anim(skel_path))
    }

    pub fn post_process(
        commands: &mut Commands,
        server: &mut Server,
        project: &mut Project,
        file_key: &FileKey,
        file_entity: &Entity,
        skel_path_opt: Option<(String, String)>,
    ) -> HashMap<Entity, ContentEntityData> {
        let mut content_entities = HashMap::new();

        info!("skel_path: {:?}", skel_path_opt);
        if let Some((skel_path, skel_file_name)) = skel_path_opt {
            let skel_file_key = FileKey::new(&skel_path, &skel_file_name, EntryKind::File);
            let file_extension = project.file_extension(&skel_file_key).unwrap();
            if file_extension != FileExtension::Skel {
                panic!("anim file should depend on a single .skel file");
            }

            project.file_add_dependency(file_key, &skel_file_key);

            let skel_file_entity = project.file_entity(&skel_file_key).unwrap();

            // get all users in room with file entity
            let mut component = FileDependency::new();
            component.file_entity.set(server, file_entity);
            component.dependency_entity.set(server, &skel_file_entity);
            let entity = commands
                .spawn_empty()
                .enable_replication(server)
                .configure_replication(ReplicationConfig::Delegated)
                .insert(component)
                .id();
            content_entities.insert(entity, ContentEntityData::new_dependency(file_extension, skel_file_key));
        }

        content_entities
    }
}

impl FileReader for AnimReader {
    fn read(
        &self,
        commands: &mut Commands,
        server: &mut Server,
        bytes: &Box<[u8]>,
    ) -> FileReadOutput {
        let mut bit_reader = BitReader::new(bytes);

        let Ok(actions) = Self::read_to_actions(&mut bit_reader) else {
            panic!("Error reading .anim file");
        };

        let Ok(result) = Self::actions_to_world(commands, server, actions) else {
            panic!("Error reading .anim file");
        };

        result
    }
}
