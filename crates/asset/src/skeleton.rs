use std::{collections::HashMap, fs};

use bevy_log::info;

use naia_serde::BitReader;

use math::{quat_from_spin_direction, Vec3};
use render_api::{AssetHash, components::Transform};

impl AssetHash<SkeletonData> for String {}

pub struct SkeletonData {
    // x, y, z, Option<parent_id, angle>, Option<vertex_name>
    vertices: Vec<(f32, f32, f32, Option<(usize, f32)>, Option<String>)>,
    //
    bone_map: HashMap<String, Transform>,
}

impl Default for SkeletonData {
    fn default() -> Self {
        panic!("");
    }
}

impl SkeletonData {
    pub(crate) fn get_bone_transform(&self, bone_name: &str) -> Option<&Transform> {
        self.bone_map.get(bone_name)
    }
}

impl From<String> for SkeletonData {
    fn from(path: String) -> Self {
        let file_path = format!("assets/{}", path);

        let Ok(data) = fs::read(&file_path) else {
            panic!("unable to read file: {:?}", &file_path);
        };

        let mut bit_reader = BitReader::new(&data);

        let actions =
            filetypes::SkelAction::read(&mut bit_reader).expect("unable to parse file");

        let mut vertices = Vec::new();
        for action in actions {
            match action {
                filetypes::SkelAction::Vertex(x, y, z, parent_opt, name_opt) => {
                    info!("Vertex: ({}, {}, {}), parent: {:?}, name: {:?}", x, y, z, parent_opt, name_opt);
                    let parent_opt = parent_opt.map(|(parent_id, rotation)| {
                        (parent_id as usize, rotation.get_radians())
                    });
                    vertices.push((x as f32, y as f32, z as f32, parent_opt, name_opt));
                }
            }
        }

        let mut bone_map = HashMap::new();
        for (x, y, z, parent_opt, name_opt) in &vertices {
            if let Some(name) = name_opt {
                let (parent_id, spin) = parent_opt.expect("named bone has no parent. invalid.");
                let (parent_x, parent_y, parent_z, _, _) = vertices[parent_id];
                let vertex_position = Vec3::new(*x, *y, *z);
                let parent_position = Vec3::new(parent_x, parent_y, parent_z);
                let direction = vertex_position - parent_position;
                let rotation = quat_from_spin_direction(spin, Vec3::X, direction);
                let transform = Transform::from_translation(parent_position).with_rotation(rotation);
                bone_map.insert(name.clone(), transform);
            }
        }

        Self {
            vertices,
            bone_map,
        }
    }
}