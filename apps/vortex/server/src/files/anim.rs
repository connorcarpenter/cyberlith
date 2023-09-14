use std::collections::HashMap;

use bevy_ecs::{
    entity::Entity,
    prelude::{Commands, World},
};

use naia_bevy_server::{BitReader, BitWrite, FileBitWriter, Serde, SerdeErr, Server, UnsignedVariableInteger};
use vortex_proto::SerdeQuat;

use crate::{
    files::{FileReadOutput, FileReader, FileWriter, ShapeTypeData},
    resources::{ContentEntityData, ShapeManager},
};

// Actions
enum AnimAction {
    // file path
    SkelFile(String),
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
    pub duration_ms: f32,
    //pub easing: Easing,
}

impl Serde for Transition {
    fn ser(&self, writer: &mut dyn BitWrite) {
        let duration_5ms = (self.duration_ms / 5.0).round() as u32;
        UnsignedVariableInteger::<7>::from(duration_5ms).ser(writer);
    }

    fn de(reader: &mut BitReader) -> Result<Self, SerdeErr> {
        let duration_5ms: u32 = UnsignedVariableInteger::<7>::de(reader)?.to();
        let duration_ms = (duration_5ms as f32) * 5.0;
        Ok(Self {
            duration_ms,
        })
    }

    fn bit_length(&self) -> u32 {
        let duration_5ms = (self.duration_ms / 5.0).round() as u32;
        UnsignedVariableInteger::<7>::from(duration_5ms).bit_length()
    }
}

// Writer
pub struct AnimWriter;

impl AnimWriter {
    fn world_to_actions(
        &self,
        world: &mut World,
        content_entities: &Vec<Entity>,
    ) -> Vec<AnimAction> {
        let mut actions = Vec::new();

        actions
    }

    fn write_from_actions(&self, actions: Vec<AnimAction>) -> Box<[u8]> {
        let mut bit_writer = FileBitWriter::new();

        for action in actions {
            match action {
                AnimAction::SkelFile(path) => {
                    AnimActionType::SkelFile.ser(&mut bit_writer);
                    path.ser(&mut bit_writer);
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
        content_entities: &HashMap<Entity, ContentEntityData>,
    ) -> Box<[u8]> {
        let content_entities_vec: Vec<Entity> = content_entities
            .iter()
            .map(|(entity, _data)| *entity)
            .collect();
        let actions = self.world_to_actions(world, &content_entities_vec);
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
                    actions.push(AnimAction::SkelFile(path));
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
        commands: &mut Commands,
        server: &mut Server,
        actions: Vec<AnimAction>,
    ) -> Result<FileReadOutput, SerdeErr> {
        Ok(FileReadOutput::Anim)
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

impl AnimReader {
    pub fn post_process_entities() -> HashMap<Entity, ContentEntityData> {
        let mut new_content_entities = HashMap::new();

        new_content_entities
    }
}
