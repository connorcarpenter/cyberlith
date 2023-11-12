use std::collections::HashMap;

use naia_serde::{BitReader, FileBitWriter, SerdeInternal as Serde, SerdeErr, UnsignedVariableInteger};
use crate::common::SerdeQuat;

// Transition
#[derive(Clone, PartialEq, Serde)]
pub struct Transition {
    duration_5ms: UnsignedVariableInteger<7>,
    //pub easing: Easing,
}

// Actions
enum AnimAction {
    // path, file_name
    SkelFile(String, String),
    // shape name -> shape_index
    ShapeIndex(String),
    // shape_index -> rotation
    Frame(HashMap<u16, SerdeQuat>, Transition),
}

#[derive(Serde, Clone, PartialEq)]
enum AnimActionType {
    SkelFile,
    ShapeIndex,
    Frame,
    None,
}

// Writer
pub struct AnimWriter;

impl AnimWriter {

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
                    for (shape_index, pose_quat) in poses {
                        // continue bit
                        true.ser(&mut bit_writer);

                        UnsignedVariableInteger::<5>::from(shape_index).ser(&mut bit_writer);
                        pose_quat.ser(&mut bit_writer);
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

                        let shape_index: u16 = UnsignedVariableInteger::<5>::de(bit_reader)?.to();
                        let pose_quat = SerdeQuat::de(bit_reader)?;
                        poses.insert(shape_index, pose_quat);
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
}
