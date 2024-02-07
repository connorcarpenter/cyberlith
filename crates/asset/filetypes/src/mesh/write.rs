use naia_serde::{FileBitWriter, Serde, UnsignedVariableInteger};

use crate::{common::VertexSerdeInt, mesh::MeshActionType, MeshAction};

impl MeshAction {
    pub fn write(actions: Vec<Self>) -> Box<[u8]> {
        let mut bit_writer = FileBitWriter::new();

        let mut test_face_index = 0;
        for action in actions.iter() {
            match action {
                Self::Vertex(x, y, z) => {
                    // continue bit
                    MeshActionType::Vertex.ser(&mut bit_writer);

                    // encode X, Y, Z
                    VertexSerdeInt::from(*x).ser(&mut bit_writer);
                    VertexSerdeInt::from(*y).ser(&mut bit_writer);
                    VertexSerdeInt::from(*z).ser(&mut bit_writer);
                }
                Self::Edge(vertex_a, vertex_b) => {
                    // continue bit
                    MeshActionType::Edge.ser(&mut bit_writer);

                    UnsignedVariableInteger::<6>::from(*vertex_a).ser(&mut bit_writer);
                    UnsignedVariableInteger::<6>::from(*vertex_b).ser(&mut bit_writer);
                }
                Self::Face(face_index, vertex_a, vertex_b, vertex_c, edge_a, edge_b, edge_c) => {
                    if *face_index != test_face_index {
                        panic!(
                            "face_index {:?} does not match test_face_index {:?}",
                            face_index, test_face_index
                        );
                    }

                    // continue bit
                    MeshActionType::Face.ser(&mut bit_writer);

                    UnsignedVariableInteger::<6>::from(*vertex_a).ser(&mut bit_writer);
                    UnsignedVariableInteger::<6>::from(*vertex_b).ser(&mut bit_writer);
                    UnsignedVariableInteger::<6>::from(*vertex_c).ser(&mut bit_writer);

                    UnsignedVariableInteger::<6>::from(*edge_a).ser(&mut bit_writer);
                    UnsignedVariableInteger::<6>::from(*edge_b).ser(&mut bit_writer);
                    UnsignedVariableInteger::<6>::from(*edge_c).ser(&mut bit_writer);

                    test_face_index += 1;
                }
            }
        }

        // continue bit
        MeshActionType::None.ser(&mut bit_writer);

        bit_writer.to_bytes()
    }
}
