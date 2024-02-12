use asset_io::{json::{AnimFile, IconFile, MeshFile, ModelFile, PaletteFile, SceneFile, SkelFile, SkinFile}, bits::{AnimAction, IconAction, MeshAction, ModelAction, PaletteAction, SceneAction, SkelAction, SkinAction}};
use asset_io::bits::SerdeRotation;

pub(crate) fn palette(data: &PaletteFile) -> Vec<u8> {
    let mut actions = Vec::new();

    for color in data.get_colors() {
        let (r, g, b) = color.deconstruct();
        actions.push(PaletteAction::Color(r, g, b));
    }

    PaletteAction::write(actions).to_vec()
}

pub(crate) fn skeleton(data: &SkelFile) -> Vec<u8> {
    let mut actions = Vec::new();

    for vertex in data.get_vertices() {
        let (x, y, z, parent_opt, name_opt) = vertex.deconstruct();
        let parent_opt = parent_opt.map(|(id, rotation)| (id, SerdeRotation::from_inner_value(rotation)));
        actions.push(SkelAction::Vertex(x, y, z, parent_opt, name_opt));
    }

    SkelAction::write(actions).to_vec()
}

pub(crate) fn mesh(data: &MeshFile) -> Vec<u8> {
    let mut actions = Vec::new();

    for vertex in data.get_vertices() {
        let (x, y, z) = vertex.deconstruct();
        actions.push(MeshAction::Vertex(x, y, z));
    }
    for face in data.get_faces() {
        let (face_id, vertex_a, vertex_b, vertex_c, _, _, _) = face.deconstruct();
        actions.push(MeshAction::Face(face_id, vertex_a, vertex_b, vertex_c));
    }

    MeshAction::write(actions).to_vec()
}

pub(crate) fn skin(data: &SkinFile) -> Vec<u8> {
    let mut actions = Vec::new();

    todo!();

    SkinAction::write(actions).to_vec()
}

pub(crate) fn scene(data: &SceneFile) -> Vec<u8> {
    let mut actions = Vec::new();

    todo!();

    SceneAction::write(actions).to_vec()
}

pub(crate) fn model(data: &ModelFile) -> Vec<u8> {
    let mut actions = Vec::new();

    todo!();

    ModelAction::write(actions).to_vec()
}

pub(crate) fn icon(data: &IconFile) -> Vec<u8> {
    let mut actions = Vec::new();

    todo!();

    IconAction::write(actions).to_vec()
}



pub(crate) fn animation(data: &AnimFile) -> Vec<u8> {
    let mut actions = Vec::new();

    todo!();

    AnimAction::write(actions).to_vec()
}