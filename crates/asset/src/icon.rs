use std::fs;

use naia_serde::BitReader;

use render_api::{AssetHash};

impl AssetHash<IconData> for String {}

pub struct IconData {

}

impl Default for IconData {
    fn default() -> Self {
        Self {

        }
    }
}

impl IconData {

}

impl From<String> for IconData {
    fn from(path: String) -> Self {
        let file_path = format!("assets/{}", path);

        let data = fs::read(file_path).expect("unable to read file");

        let mut bit_reader = BitReader::new(&data);

        let actions =
            filetypes::IconAction::read(&mut bit_reader).expect("unable to parse file");

        let mut frame_index = 0;
        for action in actions {
            match action {
                filetypes::IconAction::PaletteFile(path, file_name) => {
                    println!("PaletteFile: {}/{}", path, file_name);
                }
                filetypes::IconAction::Frame(frame_actions) => {
                    println!("Frame: {}", frame_index);

                    for frame_action in frame_actions {
                        match frame_action {
                            filetypes::IconFrameAction::Vertex(x, y) => {
                                println!("Vertex: ({}, {})", x, y);
                            }
                            filetypes::IconFrameAction::Edge(vertex_1_id, vertex_2_id) => {
                                println!("Edge: ({}, {})", vertex_1_id, vertex_2_id);
                            }
                            filetypes::IconFrameAction::Face(order_index, palette_index, vertex_1_id, vertex_2_id, vertex_3_id, edge_1_id, edge_2_id, edge_3_id) => {
                                println!("Face: ({}, {}, {}, {}, {}, {}, {}, {})", order_index, palette_index, vertex_1_id, vertex_2_id, vertex_3_id, edge_1_id, edge_2_id, edge_3_id);
                            }
                        }
                    }

                    frame_index += 1;
                }
            }
        }

        // todo: lots here

        Self {

        }
    }
}