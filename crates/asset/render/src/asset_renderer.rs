use bevy_log::warn;

use render_api::{
    base::{CpuMaterial, CpuMesh},
    components::{RenderLayer, Transform},
    resources::RenderFrame,
};
use storage::Handle;
use ui::{NodeActiveState, NodeId, Ui, WidgetKind};

use crate::{ui_manager::{Blinkiness, UiTextMeasurer}, asset_dependency::AssetComponentHandle, processed_asset_store::ProcessedAssetStore, AnimationData, AssetHandle, IconData, MeshData, ModelData, SceneData, SkinData, UiData};

pub(crate) struct AssetRenderer;

impl AssetRenderer {
    pub(crate) fn draw_mesh(
        render_frame: &mut RenderFrame,
        render_layer_opt: Option<&RenderLayer>,
        asset_store: &ProcessedAssetStore,
        mesh_handle: &AssetHandle<MeshData>,
        mat_handle: &Handle<CpuMaterial>,
        transform: &Transform,
    ) {
        let Some(mesh_file) = asset_store.meshes.get(mesh_handle) else {
            warn!("mesh file not loaded 1: {:?}", mesh_handle.asset_id());
            return;
        };
        let Some(cpu_mesh_handle) = mesh_file.get_cpu_mesh_handle() else {
            warn!("mesh file not loaded 2: {:?}", mesh_handle.asset_id());
            return;
        };
        render_frame.draw_mesh(render_layer_opt, cpu_mesh_handle, mat_handle, transform);
    }

    pub(crate) fn draw_icon(
        render_frame: &mut RenderFrame,
        render_layer_opt: Option<&RenderLayer>,
        asset_store: &ProcessedAssetStore,
        icon_handle: &AssetHandle<IconData>,
        subimage_index: usize,
        transform: &Transform,
    ) {
        let Some(icon_data) = asset_store.icons.get(icon_handle) else {
            warn!("icon data not loaded 1: {:?}", icon_handle.asset_id());
            return;
        };
        let Some((cpu_mesh_handle, cpu_skin_handle)) =
            icon_data.get_cpu_mesh_and_skin_handles(subimage_index)
        else {
            warn!("icon data not loaded 2: {:?}", icon_handle.asset_id());
            return;
        };
        render_frame.draw_skinned_mesh(
            render_layer_opt,
            &cpu_mesh_handle,
            &cpu_skin_handle,
            transform,
        );
    }

    pub(crate) fn draw_icon_with_material(
        render_frame: &mut RenderFrame,
        render_layer_opt: Option<&RenderLayer>,
        asset_store: &ProcessedAssetStore,
        icon_handle: &AssetHandle<IconData>,
        mat_handle: &Handle<CpuMaterial>,
        subimage_index: usize,
        transform: &Transform,
    ) {
        let Some(icon_data) = asset_store.icons.get(icon_handle) else {
            warn!("icon data not loaded 1: {:?}", icon_handle.asset_id());
            return;
        };
        let Some(cpu_mesh_handle) = icon_data.get_cpu_mesh_handle(subimage_index) else {
            warn!("icon data not loaded 2: {:?}", icon_handle.asset_id());
            return;
        };
        render_frame.draw_mesh(render_layer_opt, &cpu_mesh_handle, mat_handle, transform);
    }

    pub(crate) fn draw_text(
        render_frame: &mut RenderFrame,
        render_layer_opt: Option<&RenderLayer>,
        asset_store: &ProcessedAssetStore,
        text_icon_handle: &AssetHandle<IconData>,
        text_color_mat_handle: &Handle<CpuMaterial>,
        transform: &Transform,
        text: &str,
    ) {
        // info!("drawing text: {}, transform: {:?}, text_height: {:?}", text, transform);

        // will draw text string:
        // at position: (transform.translation.x, transform.translation.y, transform.translation.z),
        // with size: (transform.scale.x, transform.scale.y)

        let Some(icon_data) = asset_store.icons.get(text_icon_handle) else {
            return;
        };
        let text_measurer = UiTextMeasurer::new(icon_data);
        let subimage_indices = ui::Text::get_subimage_indices(text);
        let (x_positions, text_height) = ui::Text::get_raw_text_rects(&text_measurer, &subimage_indices);

        let mut cursor = Transform::from_xyz(
            0.0,
            transform.translation.y + (transform.scale.y * 0.5),
            transform.translation.z,
        );
        // if we want to fill 200px, but raw_width is 100px, then scale_x would be 2.0
        cursor.scale.y = transform.scale.y / text_height;
        cursor.scale.x = cursor.scale.y;

        for char_index in 0..subimage_indices.len() {
            let frame_x = x_positions[char_index]  * cursor.scale.x;
            let next_frame_x = x_positions[char_index+1] * cursor.scale.x;
            let frame_index = subimage_indices[char_index];

            cursor.translation.x = transform.translation.x + (frame_x + next_frame_x) / 2.0;

            Self::draw_icon_with_material(
                render_frame,
                render_layer_opt,
                asset_store,
                text_icon_handle,
                text_color_mat_handle,
                frame_index,
                &cursor,
            );
        }
    }

    pub(crate) fn draw_text_carat(
        render_frame: &mut RenderFrame,
        render_layer_opt: Option<&RenderLayer>,
        asset_store: &ProcessedAssetStore,
        text_icon_handle: &AssetHandle<IconData>,
        text_color_mat_handle: &Handle<CpuMaterial>,
        transform: &Transform,
        text: &str,
        carat_index: usize,
    ) {
        let Some(icon_data) = asset_store.icons.get(text_icon_handle) else {
            return;
        };
        let text_measurer = UiTextMeasurer::new(icon_data);
        let subimage_indices = ui::Text::get_subimage_indices(text);
        let (x_positions, text_height) = ui::Text::get_raw_text_rects(&text_measurer, &subimage_indices);

        let mut cursor = Transform::from_xyz(
            0.0,
            transform.translation.y + (transform.scale.y * 0.5),
            transform.translation.z,
        );

        // if we want to fill 200px, but raw_width is 100px, then scale_x would be 2.0
        cursor.scale.y = transform.scale.y / text_height;
        cursor.scale.x = cursor.scale.y;

        cursor.translation.x = transform.translation.x + (x_positions[carat_index] * cursor.scale.x);

        Self::draw_icon_with_material(
            render_frame,
            render_layer_opt,
            asset_store,
            text_icon_handle,
            text_color_mat_handle,
            (124 - 32) as usize, // pipe character '|'
            &cursor,
        );
    }

    pub(crate) fn draw_text_selection(
        render_frame: &mut RenderFrame,
        render_layer_opt: Option<&RenderLayer>,
        asset_store: &ProcessedAssetStore,
        text_icon_handle: &AssetHandle<IconData>,
        mesh_handle: &Handle<CpuMesh>,
        mat_handle: &Handle<CpuMaterial>,
        transform: &Transform,
        text: &str,
        select_index: usize,
        carat_index: usize,
    ) {
        let Some(icon_data) = asset_store.icons.get(text_icon_handle) else {
            return;
        };
        let text_measurer = UiTextMeasurer::new(icon_data);
        let subimage_indices = ui::Text::get_subimage_indices(text);
        let (x_positions, text_height) = ui::Text::get_raw_text_rects(&text_measurer, &subimage_indices);
        let text_scale = transform.scale.y / text_height;

        let pos_a = transform.translation.x + (x_positions[carat_index] * text_scale);
        let pos_b = transform.translation.x + (x_positions[select_index] * text_scale);
        let (x_pos, x_scale) = if carat_index < select_index {
            (pos_a, pos_b - pos_a)
        } else {
            (pos_b, pos_a - pos_b)
        };

        let mut box_transform = transform.clone();
        box_transform.translation.x = x_pos;
        box_transform.scale.x = x_scale;
        box_transform.translation.y += 8.0;
        box_transform.scale.y -= 16.0;
        render_frame.draw_mesh(render_layer_opt, mesh_handle, mat_handle, &box_transform);
    }

    pub(crate) fn draw_skin(
        render_frame: &mut RenderFrame,
        render_layer_opt: Option<&RenderLayer>,
        asset_store: &ProcessedAssetStore,
        skin_handle: &AssetHandle<SkinData>,
        transform: &Transform,
    ) {
        let Some(skin_data) = asset_store.skins.get(skin_handle) else {
            warn!("skin data {:?} not loaded 1", skin_handle.asset_id());
            return;
        };
        let Some(mesh_file_handle) = skin_data.get_mesh_file_handle() else {
            warn!("skin data {:?} not loaded 2", skin_handle.asset_id());
            return;
        };
        let Some(mesh_file) = asset_store.meshes.get(mesh_file_handle) else {
            warn!("skin data {:?} not loaded 3", skin_handle.asset_id());
            return;
        };
        let Some(cpu_mesh_handle) = mesh_file.get_cpu_mesh_handle() else {
            warn!("skin data {:?} not loaded 4", skin_handle.asset_id());
            return;
        };
        let Some(cpu_skin_handle) = skin_data.get_cpu_skin_handle() else {
            warn!("skin data {:?} not loaded 5", skin_handle.asset_id());
            return;
        };
        render_frame.draw_skinned_mesh(
            render_layer_opt,
            cpu_mesh_handle,
            cpu_skin_handle,
            transform,
        );
    }

    pub(crate) fn draw_scene(
        render_frame: &mut RenderFrame,
        render_layer_opt: Option<&RenderLayer>,
        asset_store: &ProcessedAssetStore,
        scene_handle: &AssetHandle<SceneData>,
        parent_transform: &Transform,
    ) {
        let Some(scene_data) = asset_store.scenes.get(scene_handle) else {
            warn!("scene data not loaded 1: {:?}", scene_handle.asset_id());
            return;
        };
        let Some(scene_components) = scene_data.get_components() else {
            // not yet completely loaded
            return;
        };
        for (skin_or_scene_handle, mut component_transform) in scene_components {
            component_transform = component_transform.multiply(parent_transform);

            match skin_or_scene_handle {
                AssetComponentHandle::Skin(skin_handle) => {
                    Self::draw_skin(
                        render_frame,
                        render_layer_opt,
                        asset_store,
                        &skin_handle,
                        &component_transform,
                    );
                }
                AssetComponentHandle::Scene(scene_handle) => {
                    Self::draw_scene(
                        render_frame,
                        render_layer_opt,
                        asset_store,
                        &scene_handle,
                        &component_transform,
                    );
                }
            }
        }
    }

    pub(crate) fn draw_model(
        render_frame: &mut RenderFrame,
        render_layer_opt: Option<&RenderLayer>,
        asset_store: &ProcessedAssetStore,
        model_handle: &AssetHandle<ModelData>,
        parent_transform: &Transform,
    ) {
        let Some(model_data) = asset_store.models.get(model_handle) else {
            warn!("model data not loaded 1: {:?}", model_handle.asset_id());
            return;
        };
        let Some(model_components) = model_data.get_components_ref() else {
            // not yet loaded all
            return;
        };
        for (skin_or_scene_handle, mut component_transform) in model_components {
            component_transform = component_transform.multiply(parent_transform);

            match skin_or_scene_handle {
                AssetComponentHandle::Skin(skin_handle) => {
                    Self::draw_skin(
                        render_frame,
                        render_layer_opt,
                        asset_store,
                        &skin_handle,
                        &component_transform,
                    );
                }
                AssetComponentHandle::Scene(scene_handle) => {
                    Self::draw_scene(
                        render_frame,
                        render_layer_opt,
                        asset_store,
                        &scene_handle,
                        &component_transform,
                    );
                }
            }
        }
    }

    pub(crate) fn draw_animated_model(
        render_frame: &mut RenderFrame,
        render_layer_opt: Option<&RenderLayer>,
        asset_store: &ProcessedAssetStore,
        model_handle: &AssetHandle<ModelData>,
        animation_handle: &AssetHandle<AnimationData>,
        parent_transform: &Transform,
        frame_time_ms: f32,
    ) {
        let Some(model_data) = asset_store.models.get(model_handle) else {
            warn!("model data not loaded 1: {:?}", model_handle.asset_id());
            return;
        };
        let Some(animation_data) = asset_store.animations.get(animation_handle) else {
            warn!(
                "animation data not loaded 1: {:?}",
                animation_handle.asset_id()
            );
            return;
        };
        let skeleton_handle = {
            let skeleton_handle_1 = model_data.get_skeleton_handle();
            let skeleton_handle_2 = animation_data.get_skeleton_handle();
            if skeleton_handle_1 != skeleton_handle_2 {
                panic!(
                    "skeleton mismatch: {:?} != {:?}",
                    skeleton_handle_1.asset_id(),
                    skeleton_handle_2.asset_id()
                );
            }
            skeleton_handle_1
        };
        let Some(skeleton_data) = asset_store.skeletons.get(&skeleton_handle) else {
            warn!(
                "skeleton data not loaded 1: {:?}",
                skeleton_handle.asset_id()
            );
            return;
        };
        let Some(model_components) =
            animation_data.get_animated_components(skeleton_data, model_data, frame_time_ms)
        else {
            // not yet loaded all
            return;
        };
        for (skin_or_scene_handle, mut component_transform) in model_components {
            component_transform = component_transform.multiply(parent_transform);

            match skin_or_scene_handle {
                AssetComponentHandle::Skin(skin_handle) => {
                    Self::draw_skin(
                        render_frame,
                        render_layer_opt,
                        asset_store,
                        &skin_handle,
                        &component_transform,
                    );
                }
                AssetComponentHandle::Scene(scene_handle) => {
                    Self::draw_scene(
                        render_frame,
                        render_layer_opt,
                        asset_store,
                        &scene_handle,
                        &component_transform,
                    );
                }
            }
        }
    }

    pub(crate) fn draw_ui(
        render_frame: &mut RenderFrame,
        render_layer_opt: Option<&RenderLayer>,
        asset_store: &ProcessedAssetStore,
        blinkiness: &Blinkiness,
        ui_handle: &AssetHandle<UiData>,
    ) {
        let Some(ui_data) = asset_store.uis.get(ui_handle) else {
            warn!("ui data not loaded 2: {:?}", ui_handle.asset_id());
            return;
        };

        let ui = ui_data.get_ui_ref();
        let text_icon_handle = ui_data.get_icon_handle();

        let carat_blink = blinkiness.enabled() || ui.interact_timer_was_recent();

        for node_id in 0..ui.store.nodes.len() {
            let node_id = NodeId::from_usize(node_id);
            draw_ui_node(
                render_frame,
                render_layer_opt,
                asset_store,
                carat_blink,
                &ui,
                &text_icon_handle,
                &node_id,
            );
        }
    }
}

fn draw_ui_node(
    render_frame: &mut RenderFrame,
    render_layer_opt: Option<&RenderLayer>,
    asset_store: &ProcessedAssetStore,
    carat_blink: bool,
    ui: &Ui,
    text_icon_handle: &AssetHandle<IconData>,
    id: &NodeId,
) {
    let Some((width, height, child_offset_x, child_offset_y, child_offset_z)) = ui.cache.bounds(id) else {
        warn!("no bounds for id: {:?}", id);
        return;
    };

    let Some(node) = ui.store.get_node(&id) else {
        warn!("no panel for id: {:?}", id);
        return;
    };

    let mut transform = Transform::from_xyz(
        child_offset_x,
        child_offset_y,
        child_offset_z,
    );
    transform.scale.x = width;
    transform.scale.y = height;

    if node.visible {
        match node.widget_kind() {
            WidgetKind::Panel => {
                draw_ui_panel(
                    render_frame,
                    render_layer_opt,
                    ui,
                    id,
                    &transform,
                );
            }
            WidgetKind::Text => {
                draw_ui_text(
                    render_frame,
                    render_layer_opt,
                    asset_store,
                    ui,
                    text_icon_handle,
                    id,
                    &transform,
                );
            }
            WidgetKind::Button => {
                draw_ui_button(
                    render_frame,
                    render_layer_opt,
                    ui,
                    id,
                    &transform,
                );
            }
            WidgetKind::Textbox => {
                draw_ui_textbox(
                    render_frame,
                    render_layer_opt,
                    asset_store,
                    carat_blink,
                    ui,
                    text_icon_handle,
                    id,
                    &transform,
                );
            }
        }
    }
}

fn draw_ui_panel(
    //self was Panel
    render_frame: &mut RenderFrame,
    render_layer_opt: Option<&RenderLayer>,
    ui: &Ui,
    node_id: &NodeId,
    transform: &Transform,
) {
    let Some(panel_ref) = ui.store.panel_ref(node_id) else {
        panic!("no panel ref for node_id: {:?}", node_id);
    };

    // draw panel
    if let Some(mat_handle) = panel_ref.background_color_handle {
        let panel_style_ref = ui.store.panel_style_ref(node_id);
        let background_alpha = panel_style_ref.background_alpha();
        if background_alpha > 0.0 {
            if background_alpha != 1.0 {
                panic!("partial background_alpha not implemented yet!");
            }
            let box_handle = ui.globals.get_box_mesh_handle().unwrap();
            render_frame.draw_mesh(render_layer_opt, box_handle, &mat_handle, &transform);
        }
    } else {
        warn!("no color handle for panel"); // probably will need to do better debugging later
        return;
    };
}

fn draw_ui_text(
    //&self, // self was text widget
    render_frame: &mut RenderFrame,
    render_layer_opt: Option<&RenderLayer>,
    asset_store: &ProcessedAssetStore,
    ui: &Ui,
    text_icon_handle: &AssetHandle<IconData>,
    node_id: &NodeId,
    transform: &Transform,
) {
    let Some(text_ref) = ui
        .store
        .get_node(node_id)
        .unwrap()
        .widget_text_ref() else {
        panic!("no text ref for node_id: {:?}", node_id);
    };

    // draw background
    if let Some(mat_handle) = text_ref.background_color_handle {
        let text_style_ref = ui.store.text_style_ref(node_id);
        let background_alpha = text_style_ref.background_alpha();
        if background_alpha > 0.0 {
            if background_alpha != 1.0 {
                panic!("partial background_alpha not implemented yet!");
            }
            let box_handle = ui.globals.get_box_mesh_handle().unwrap();
            let mut new_transform = transform.clone();
            new_transform.translation.z -= 0.025;
            render_frame.draw_mesh(render_layer_opt, box_handle, &mat_handle, &new_transform);
        }
    } else {
        warn!("no background color handle for text"); // probably will need to do better debugging later
        return;
    };

    let Some(text_color_handle) = ui.globals.get_text_color_handle() else {
        panic!("No text color handle found in globals");
    };

    AssetRenderer::draw_text(
        render_frame,
        render_layer_opt,
        asset_store,
        text_icon_handle,
        text_color_handle,
        transform,
        &text_ref.text,
    );
}

fn draw_ui_button(
    //self was Button
    render_frame: &mut RenderFrame,
    render_layer_opt: Option<&RenderLayer>,
    ui: &Ui,
    node_id: &NodeId,
    transform: &Transform,
) {
    let Some(button_ref) = ui.store.button_ref(node_id) else {
        panic!("no button ref for node_id: {:?}", node_id);
    };

    // draw button
    let active_state = ui.get_active_state(node_id);
    if let Some(mat_handle) = button_ref.current_color_handle(active_state) {
        let button_style_ref = ui.store.button_style_ref(node_id);
        let background_alpha = button_style_ref.background_alpha();
        if background_alpha > 0.0 {
            if background_alpha != 1.0 {
                panic!("partial background_alpha not implemented yet!");
            }
            let box_handle = ui.globals.get_box_mesh_handle().unwrap();
            render_frame.draw_mesh(render_layer_opt, box_handle, &mat_handle, &transform);
        }
    } else {
        warn!("no color handle for button"); // probably will need to do better debugging later
        return;
    };
}

fn draw_ui_textbox(
    //self was Textbox
    render_frame: &mut RenderFrame,
    render_layer_opt: Option<&RenderLayer>,
    asset_store: &ProcessedAssetStore,
    carat_blink: bool,
    ui: &Ui,
    text_icon_handle: &AssetHandle<IconData>,
    node_id: &NodeId,
    transform: &Transform,
) {
    let Some(textbox_ref) = ui.store.textbox_ref(node_id) else {
        panic!("no textbox ref for node_id: {:?}", node_id);
    };

    // draw textbox
    let active_state = ui.get_active_state(node_id);
    if let Some(mat_handle) = textbox_ref.current_color_handle(active_state) {
        let textbox_style_ref = ui.store.textbox_style_ref(node_id);
        let background_alpha = textbox_style_ref.background_alpha();
        if background_alpha > 0.0 {
            if background_alpha != 1.0 {
                panic!("partial background_alpha not implemented yet!");
            }
            let box_handle = ui.globals.get_box_mesh_handle().unwrap();
            render_frame.draw_mesh(render_layer_opt, box_handle, &mat_handle, &transform);
        }
    } else {
        warn!("no color handle for textbox"); // probably will need to do better debugging later
        return;
    };

    // draw text
    let Some(text_color_handle) = ui.globals.get_text_color_handle() else {
        panic!("No text color handle found in globals");
    };

    // draw text
    let mut text_transform = transform.clone();
    text_transform.translation.x += 8.0;

    {
        text_transform.translation.z = transform.translation.z + 0.05;
        AssetRenderer::draw_text(
            render_frame,
            render_layer_opt,
            asset_store,
            text_icon_handle,
            text_color_handle,
            &text_transform,
            &textbox_ref.text,
        );
    }

    if active_state == NodeActiveState::Active {

        // draw selection box if needed
        if let Some(select_index) = textbox_ref.select_index {
            if let Some(mat_handle) = textbox_ref.get_selection_color_handle() {
                text_transform.translation.z = transform.translation.z + 0.025;
                AssetRenderer::draw_text_selection(
                    render_frame,
                    render_layer_opt,
                    asset_store,
                    text_icon_handle,
                    ui.globals.get_box_mesh_handle().unwrap(),
                    &mat_handle,
                    &text_transform,
                    &textbox_ref.text,
                    select_index,
                    textbox_ref.carat_index,
                );
            }
        }

        // draw carat if needed
        if carat_blink {
            text_transform.translation.z = transform.translation.z + 0.05;
            AssetRenderer::draw_text_carat(
                render_frame,
                render_layer_opt,
                asset_store,
                text_icon_handle,
                text_color_handle,
                &text_transform,
                &textbox_ref.text,
                textbox_ref.carat_index,
            );
        }
    }
}
