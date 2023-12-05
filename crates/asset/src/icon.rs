use std::fs;

use bevy_log::info;

use naia_serde::BitReader;

use render_api::{AssetHash, Handle};

use crate::{AssetHandle, PaletteData, asset_dependency::AssetDependency};
use crate::asset_handle::AssetHandleImpl;

impl AssetHash<IconData> for String {}

pub struct IconData {
    palette_file: AssetDependency<PaletteData>,
}

impl Default for IconData {
    fn default() -> Self {
        panic!("");
    }
}

impl IconData {
    pub(crate) fn load_dependencies(&self, handle: Handle<Self>, dependencies: &mut Vec<(AssetHandle, String)>) {
        let AssetDependency::<PaletteData>::Path(path) = &self.palette_file else {
            panic!("expected path right after load");
        };
        dependencies.push((handle.into(), path.clone()));
    }

    pub(crate) fn finish_dependency(&mut self, _dependency_path: String, dependency_handle: AssetHandle) {
        match dependency_handle.to_impl() {
            AssetHandleImpl::Palette(handle) => {
                self.palette_file.load_handle(handle);
            }
            _ => {
                panic!("unexpected type of handle");
            }
        }
    }
}

impl From<String> for IconData {
    fn from(path: String) -> Self {
        let file_path = format!("assets/{}", path);

        let Ok(data) = fs::read(&file_path) else {
            panic!("unable to read file: {:?}", &file_path);
        };

        let mut bit_reader = BitReader::new(&data);

        let actions =
            filetypes::IconAction::read(&mut bit_reader).expect("unable to parse file");

        let mut palette_file_opt = None;
        let mut frame_index = 0;
        for action in actions {
            match action {
                filetypes::IconAction::PaletteFile(path, file_name) => {
                    palette_file_opt = Some(format!("{}/{}", path, file_name));
                }
                filetypes::IconAction::Frame(frame_actions) => {
                    info!("- Frame Start: {} -", frame_index);

                    for frame_action in frame_actions {
                        match frame_action {
                            filetypes::IconFrameAction::Vertex(x, y) => {
                                info!("Vertex: ({}, {})", x, y);
                            }
                            filetypes::IconFrameAction::Edge(vertex_1_id, vertex_2_id) => {
                                info!("Edge: ({}, {})", vertex_1_id, vertex_2_id);
                            }
                            filetypes::IconFrameAction::Face(order_index, palette_index, vertex_1_id, vertex_2_id, vertex_3_id, edge_1_id, edge_2_id, edge_3_id) => {
                                info!("Face: ({}, {}, {}, {}, {}, {}, {}, {})", order_index, palette_index, vertex_1_id, vertex_2_id, vertex_3_id, edge_1_id, edge_2_id, edge_3_id);
                            }
                        }
                    }

                    info!("- Frame End -");

                    frame_index += 1;
                }
            }
        }

        // todo: lots here

        Self {
            palette_file: AssetDependency::Path(palette_file_opt.unwrap()),
        }
    }
}