use asset_io::bits::{AnimAction, IconAction, MeshAction, ModelAction, PaletteAction, SceneAction, SkelAction, SkinAction};
use asset_io::json::{AnimFile, IconFile, MeshFile, ModelFile, PaletteFile, SceneFile, SkelFile, SkinFile};

pub(crate) fn palette(data: &PaletteFile) -> Vec<u8> {
    let mut actions = Vec::new();

    todo!();

    PaletteAction::write(actions).to_vec()
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

pub(crate) fn mesh(data: &MeshFile) -> Vec<u8> {
    let mut actions = Vec::new();

    todo!();

    MeshAction::write(actions).to_vec()
}

pub(crate) fn skeleton(data: &SkelFile) -> Vec<u8> {
    let mut actions = Vec::new();

    todo!();

    SkelAction::write(actions).to_vec()
}

pub(crate) fn skin(data: &SkinFile) -> Vec<u8> {
    let mut actions = Vec::new();

    todo!();

    SkinAction::write(actions).to_vec()
}

pub(crate) fn animation(data: &AnimFile) -> Vec<u8> {
    let mut actions = Vec::new();

    todo!();

    AnimAction::write(actions).to_vec()
}