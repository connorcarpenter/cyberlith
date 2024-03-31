use std::collections::HashMap;

use asset_serde::{
    bits::{
        AnimAction, ComponentFileType, IconAction, IconFrameAction, MeshAction, ModelAction,
        PaletteAction, SceneAction, SerdeQuat, SerdeRotation, SkelAction, SkinAction, Transition,
    },
    json::{
        AnimationJson, FileComponentType, IconJson, MeshJson, ModelJson, PaletteJson, SceneJson,
        SkeletonJson, SkinJson, UiConfigJson,
    },
};

pub(crate) fn palette(data: PaletteJson) -> Vec<u8> {
    let mut actions = Vec::new();

    for color in data.get_colors() {
        let (r, g, b) = color.deconstruct();
        actions.push(PaletteAction::Color(r, g, b));
    }

    PaletteAction::write(actions).to_vec()
}

pub(crate) fn skeleton(data: SkeletonJson) -> Vec<u8> {
    let mut actions = Vec::new();

    for vertex in data.get_vertices() {
        let (x, y, z, parent_opt, name_opt) = vertex.deconstruct();
        let parent_opt =
            parent_opt.map(|(id, rotation)| (id, SerdeRotation::from_inner_value(rotation)));
        actions.push(SkelAction::Vertex(x, y, z, parent_opt, name_opt));
    }

    SkelAction::write(actions).to_vec()
}

pub(crate) fn mesh(data: MeshJson) -> Vec<u8> {
    let mut actions = Vec::new();

    for vertex in data.get_vertices() {
        let (x, y, z) = vertex.deconstruct();
        actions.push(MeshAction::Vertex(x, y, z));
    }
    for face in data.get_faces() {
        let (face_id, vertex_a, vertex_b, vertex_c) = face.deconstruct();
        actions.push(MeshAction::Face(face_id, vertex_a, vertex_b, vertex_c));
    }

    MeshAction::write(actions).to_vec()
}

pub(crate) fn skin(data: SkinJson) -> Vec<u8> {
    let mut actions = Vec::new();

    let palette_asset_id = data.get_palette_asset_id();
    actions.push(SkinAction::PaletteFile(palette_asset_id));

    let mesh_asset_id = data.get_mesh_asset_id();
    actions.push(SkinAction::MeshData(mesh_asset_id));

    let background_color = data.get_background_color_id();
    actions.push(SkinAction::BackgroundColor(background_color));

    for face_color in data.get_face_colors() {
        let face_id = face_color.face_id();
        let color_id = face_color.color_id();
        actions.push(SkinAction::SkinColor(face_id, color_id));
    }

    SkinAction::write(actions).to_vec()
}

pub(crate) fn animation(data: AnimationJson) -> Vec<u8> {
    let mut actions = Vec::new();

    let skeleton_asset_id = data.get_skeleton_asset_id();
    actions.push(AnimAction::SkelFile(skeleton_asset_id));

    for name in data.get_edge_names() {
        actions.push(AnimAction::ShapeIndex(name.clone()));
    }

    for frame in data.get_frames() {
        let mut rotations = HashMap::new();
        for pose in frame.get_poses() {
            let id = pose.get_edge_id();
            let rotation = pose.get_rotation();
            rotations.insert(
                id,
                SerdeQuat::from_xyzw(
                    rotation.get_x(),
                    rotation.get_y(),
                    rotation.get_z(),
                    rotation.get_w(),
                ),
            );
        }
        let transition_ms = frame.get_transition_ms();
        actions.push(AnimAction::Frame(rotations, Transition::new(transition_ms)));
    }

    AnimAction::write(actions).to_vec()
}

pub(crate) fn icon(data: IconJson) -> Vec<u8> {
    let mut actions = Vec::new();

    let palette_id = data.get_palette_asset_id();
    actions.push(IconAction::PaletteFile(palette_id));

    for frame in data.get_frames() {
        let mut frame_actions = Vec::new();
        for vertex in frame.get_vertices() {
            frame_actions.push(IconFrameAction::Vertex(vertex.x(), vertex.y()));
        }
        for face in frame.get_faces() {
            let face_id = face.face_id();
            let color_id = face.color_id();
            let vertex_a = face.vertex_a();
            let vertex_b = face.vertex_b();
            let vertex_c = face.vertex_c();

            frame_actions.push(IconFrameAction::Face(
                face_id, color_id, vertex_a, vertex_b, vertex_c,
            ));
        }
        actions.push(IconAction::Frame(frame_actions));
    }

    IconAction::write(actions).to_vec()
}

pub(crate) fn scene(data: SceneJson) -> Vec<u8> {
    let mut actions = Vec::new();

    for component in data.get_components() {
        let asset_id = component.asset_id();
        let kind = match component.kind() {
            FileComponentType::Scene => ComponentFileType::Scene,
            FileComponentType::Skin => ComponentFileType::Skin,
        };
        actions.push(SceneAction::Component(asset_id, kind));
    }

    for transform in data.get_transforms() {
        let component_id = transform.component_id();
        let position = transform.position();
        let scale = transform.scale();
        let rotation = transform.rotation();
        actions.push(SceneAction::NetTransform(
            component_id,
            position.x(),
            position.y(),
            position.z(),
            scale.x(),
            scale.y(),
            scale.z(),
            SerdeQuat::from_xyzw(rotation.x(), rotation.y(), rotation.z(), rotation.w()),
        ));
    }

    SceneAction::write(actions).to_vec()
}

pub(crate) fn model(data: ModelJson) -> Vec<u8> {
    let mut actions = Vec::new();

    let skel_id = data.get_skeleton_id();
    actions.push(ModelAction::SkelFile(skel_id));

    for component in data.get_components() {
        let asset_id = component.asset_id();
        let kind = match component.kind() {
            FileComponentType::Scene => ComponentFileType::Scene,
            FileComponentType::Skin => ComponentFileType::Skin,
        };
        actions.push(ModelAction::Component(asset_id, kind));
    }

    for transform in data.get_transforms() {
        let component_id = transform.component_id();
        let name = transform.name();
        let position = transform.position();
        let scale = transform.scale();
        let rotation = transform.rotation();
        actions.push(ModelAction::NetTransform(
            component_id,
            name,
            position.x(),
            position.y(),
            position.z(),
            scale.x(),
            scale.y(),
            scale.z(),
            SerdeQuat::from_xyzw(rotation.x(), rotation.y(), rotation.z(), rotation.w()),
        ));
    }

    ModelAction::write(actions).to_vec()
}

pub(crate) fn ui(data: UiConfigJson) -> Vec<u8> {
    let ui = data.into();
    let ui_bytes = asset_serde::bits::write_ui_bits(&ui);

    ui_bytes
}
