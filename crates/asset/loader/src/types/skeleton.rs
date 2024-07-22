use std::collections::{HashMap, HashSet};

use logging::info;

use math::{quat_from_spin_direction, Quat, Vec3};
use render_api::components::Transform;

pub struct SkeletonData {
    // x, y, z, Option<parent_id, angle>, Option<vertex_name>
    vertices: Vec<(Vec3, Option<(usize, f32)>, Option<String>)>,
    //
    root_vertex_id: usize,
    vertex_parent_map: HashMap<usize, HashSet<usize>>,
    //
    bone_transform_map: HashMap<String, Transform>,
}

impl Default for SkeletonData {
    fn default() -> Self {
        panic!("");
    }
}

impl SkeletonData {
    pub(crate) fn get_bone_transform(&self, bone_name: &str) -> Option<&Transform> {
        self.bone_transform_map.get(bone_name)
    }

    pub(crate) fn get_interpolated_skeleton(
        &self,
        interpolated_rotations: HashMap<String, Quat>,
    ) -> HashMap<String, Transform> {
        let mut output = HashMap::new();

        if let Some(children) = self.vertex_parent_map.get(&self.root_vertex_id) {
            for child_id in children {
                self.recurse(
                    Vec3::ZERO,
                    Vec3::ZERO,
                    Quat::IDENTITY,
                    *child_id,
                    &interpolated_rotations,
                    &mut output,
                );
            }
        }

        output
    }

    fn recurse(
        &self,
        original_parent_pos: Vec3,
        rotated_parent_pos: Vec3,
        parent_rotation: Quat,
        vertex_id: usize,
        interpolated_rotations: &HashMap<String, Quat>,
        output: &mut HashMap<String, Transform>,
    ) {
        let (original_child_pos, Some((_, spin)), name_opt) = &self.vertices[vertex_id] else {
            panic!("impossible");
        };

        let mut child_rotation = Quat::IDENTITY;
        if let Some(name) = name_opt {
            if let Some(interpolated_rotation) = interpolated_rotations.get(name) {
                child_rotation = *interpolated_rotation;
            }
        }

        let child_rotation = (parent_rotation * child_rotation).normalize();
        let original_child_displacement = *original_child_pos - original_parent_pos;
        let rotated_child_displacement = child_rotation * original_child_displacement;
        let rotated_child_pos = rotated_parent_pos + rotated_child_displacement;
        let original_bone_rotation =
            quat_from_spin_direction(*spin, Vec3::X, original_child_displacement);

        if let Some(name) = name_opt {
            let child_transform = Transform::from_translation(rotated_parent_pos)
                .with_rotation(child_rotation * original_bone_rotation);
            output.insert(name.clone(), child_transform);
        }

        if let Some(children) = self.vertex_parent_map.get(&vertex_id) {
            for child_id in children {
                self.recurse(
                    *original_child_pos,
                    rotated_child_pos,
                    child_rotation,
                    *child_id,
                    interpolated_rotations,
                    output,
                );
            }
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let actions = asset_serde::bits::SkelAction::read(bytes).expect("unable to parse file");

        let mut vertices = Vec::new();
        for action in actions {
            match action {
                asset_serde::bits::SkelAction::Vertex(x, y, z, parent_opt, name_opt) => {
                    // info!(
                    //     "Vertex: ({}, {}, {}), parent: {:?}, name: {:?}",
                    //     x, y, z, parent_opt, name_opt
                    // );
                    let parent_opt = parent_opt
                        .map(|(parent_id, rotation)| (parent_id as usize, rotation.get_radians()));
                    vertices.push((
                        Vec3::new(x as f32, y as f32, z as f32),
                        parent_opt,
                        name_opt,
                    ));
                }
            }
        }

        let mut root_vertex_id = None;
        let mut bone_map = HashMap::new();
        let mut vertex_parent_map = HashMap::new();
        for (vertex_index, (vertex_position, parent_opt, name_opt)) in vertices.iter().enumerate() {
            if let Some(name) = name_opt {
                let (parent_id, spin) = parent_opt.expect("named bone has no parent. invalid.");
                let (parent_position, _, _) = vertices[parent_id];
                let direction = *vertex_position - parent_position;
                let rotation = quat_from_spin_direction(spin, Vec3::X, direction);
                let transform =
                    Transform::from_translation(parent_position).with_rotation(rotation);
                bone_map.insert(name.clone(), transform);
            } else {
                if parent_opt.is_none() {
                    root_vertex_id = Some(vertex_index);
                }
            }

            if let Some((parent_id, _)) = parent_opt {
                let parent_set = vertex_parent_map
                    .entry(*parent_id)
                    .or_insert(HashSet::new());
                parent_set.insert(vertex_index);
            }
        }

        Self {
            vertices,
            vertex_parent_map,
            bone_transform_map: bone_map,
            root_vertex_id: root_vertex_id.expect("no root vertex found"),
        }
    }
}
