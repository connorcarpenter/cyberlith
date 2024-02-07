use std::collections::HashMap;

use naia_serde::{BitReader, Serde, SerdeErr, UnsignedVariableInteger};

use crate::{
    animation::{AnimAction, AnimActionType, Transition},
    common::SerdeQuat,
};

impl AnimAction {
    pub fn read(bit_reader: &mut BitReader) -> Result<Vec<Self>, SerdeErr> {
        let mut actions = Vec::new();

        loop {
            let action_type = AnimActionType::de(bit_reader)?;

            match action_type {
                AnimActionType::SkelFile => {
                    let path = String::de(bit_reader)?;
                    let file_name = String::de(bit_reader)?;
                    actions.push(Self::SkelFile(path, file_name));
                }
                AnimActionType::ShapeIndex => {
                    let name = String::de(bit_reader)?;
                    actions.push(Self::ShapeIndex(name));
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
                    actions.push(Self::Frame(poses, transition));
                }
                AnimActionType::None => {
                    break;
                }
            }
        }

        Ok(actions)
    }
}
