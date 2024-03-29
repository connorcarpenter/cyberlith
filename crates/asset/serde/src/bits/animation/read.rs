use std::collections::HashMap;

use asset_id::AssetId;
use naia_serde::{BitReader, Serde, SerdeErr, UnsignedVariableInteger};

use crate::bits::{
    animation::{AnimAction, AnimActionType, Transition},
    common::SerdeQuat,
};

impl AnimAction {
    pub fn read(bytes: &[u8]) -> Result<Vec<Self>, SerdeErr> {
        let mut bit_reader = BitReader::new(bytes);
        let bit_reader = &mut bit_reader;
        let mut actions = Vec::new();

        loop {
            let action_type = AnimActionType::de(bit_reader)?;

            match action_type {
                AnimActionType::SkelFile => {
                    let val = u32::de(bit_reader)?;
                    actions.push(Self::SkelFile(AssetId::from_u32(val).unwrap()));
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
