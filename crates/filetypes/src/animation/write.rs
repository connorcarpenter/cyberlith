use naia_serde::{FileBitWriter, Serde, UnsignedVariableInteger};

use crate::animation::{AnimAction, AnimActionType};

impl AnimAction {
    pub fn write(actions: Vec<Self>) -> Box<[u8]> {
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
