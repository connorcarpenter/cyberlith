use logging::warn;

use asset_loader::{AssetHandle, AssetManager, IconData, UiTextMeasurer};
use asset_render::AssetRender;
use render_api::base::Color;
use render_api::components::AmbientLight;
use render_api::{
    base::{CpuMaterial, CpuMesh},
    components::{RenderLayer, Transform},
    resources::RenderFrame,
};
use storage::Handle;
use ui_runner::{
    config::{text_get_raw_rects, text_get_subimage_indices, NodeId, UiRuntimeConfig, WidgetKind},
    input::UiInputState,
    state::{NodeActiveState, UiState},
    Blinkiness, UiHandle, UiManager,
};
use ui_runner::config::get_carat_offset_and_scale;

pub trait UiRender {
    fn draw_ui(&self, asset_manager: &AssetManager, render_frame: &mut RenderFrame);
}

impl UiRender for UiManager {
    fn draw_ui(&self, asset_manager: &AssetManager, render_frame: &mut RenderFrame) {
        if let Some(active_ui_handle) = self.active_ui() {
            UiRenderer::draw_ui(
                self,
                asset_manager,
                render_frame,
                &self.blinkiness,
                &active_ui_handle,
            );
        }
    }
}

pub struct UiRenderer;

impl UiRenderer {
    pub fn draw_ui(
        ui_manager: &UiManager,
        asset_manager: &AssetManager,
        render_frame: &mut RenderFrame,
        blinkiness: &Blinkiness,
        ui_handle: &UiHandle,
    ) {
        let Some(ui_runner) = ui_manager.ui_runtimes.get(ui_handle) else {
            warn!("ui data not loaded 2: {:?}", ui_handle.asset_id());
            return;
        };

        let (ui_state, ui_input_state, ui, _, camera_bundle) = ui_runner.decompose_to_refs();

        render_frame.draw_camera(
            Some(&RenderLayer::UI),
            &camera_bundle.camera,
            &camera_bundle.transform,
            &camera_bundle.projection,
        );
        render_frame.draw_ambient_light(
            Some(&RenderLayer::UI),
            &AmbientLight::new(1.0, Color::WHITE),
        );

        let text_icon_handle = ui_runner.get_text_icon_handle();
        let eye_icon_handle = ui_runner.get_eye_icon_handle();

        let carat_blink = blinkiness.enabled() || ui_input_state.interact_timer_within_seconds(1.0);

        for node_id in 0..ui.nodes_len() {
            let node_id = NodeId::from_usize(node_id);
            draw_ui_node(
                render_frame,
                asset_manager,
                carat_blink,
                ui,
                ui_state,
                ui_input_state,
                &text_icon_handle,
                &eye_icon_handle,
                &node_id,
            );
        }
    }

    pub fn draw_text_carat(
        render_frame: &mut RenderFrame,
        asset_manager: &AssetManager,
        text_icon_handle: &AssetHandle<IconData>,
        text_color_mat_handle: &Handle<CpuMaterial>,
        transform: &Transform,
        text: &str,
        text_offset_index: usize,
        carat_index: usize,
    ) {
        let Some(icon_data) = asset_manager.get_store().icons.get(text_icon_handle) else {
            return;
        };
        let text_measurer = UiTextMeasurer::new(icon_data);

        let (carat_offset_x, carat_scale) = get_carat_offset_and_scale(
            &text_measurer,
            transform.scale.y,
            text,
            text_offset_index,
            carat_index
        );

        let mut carat_transform = Transform::from_xyz(
            transform.translation.x + carat_offset_x,
            transform.translation.y + (transform.scale.y * 0.5),
            transform.translation.z,
        );

        // if we want to fill 200px, but raw_width is 100px, then scale_x would be 2.0
        carat_transform.scale.y = carat_scale;
        carat_transform.scale.x = carat_scale;

        asset_manager.draw_icon_with_material(
            render_frame,
            Some(&RenderLayer::UI),
            text_icon_handle,
            text_color_mat_handle,
            (124 - 32) as usize, // pipe character '|'
            &carat_transform,
        );
    }

    pub fn draw_text_selection(
        render_frame: &mut RenderFrame,
        asset_manager: &AssetManager,
        text_icon_handle: &AssetHandle<IconData>,
        mesh_handle: &Handle<CpuMesh>,
        mat_handle: &Handle<CpuMaterial>,
        transform: &Transform,
        text: &str,
        offset_index: usize,
        select_index: usize,
        carat_index: usize,
    ) {
        let Some(icon_data) = asset_manager.get_store().icons.get(text_icon_handle) else {
            return;
        };
        let text_measurer = UiTextMeasurer::new(icon_data);
        let subimage_indices = text_get_subimage_indices(&text[offset_index..text.len()]);
        let (x_positions, text_height) = text_get_raw_rects(&text_measurer, &subimage_indices);
        let text_scale = transform.scale.y / text_height;

        let carat_index = if offset_index > carat_index { 0 } else { carat_index - offset_index };
        let select_index = if offset_index > select_index { 0 } else { select_index - offset_index };

        let pos_a = transform.translation.x + (x_positions[carat_index] * text_scale);
        let pos_b = transform.translation.x + (x_positions[select_index] * text_scale);
        let (x_pos, x_scale) = if carat_index < select_index {
            (pos_a, pos_b - pos_a)
        } else {
            (pos_b, pos_a - pos_b)
        };
        let x_scale = x_scale.min(transform.scale.x);

        let mut box_transform = transform.clone();
        box_transform.translation.x = x_pos;
        box_transform.scale.x = x_scale;
        let selection_height_offset = text_height * 0.02;
        box_transform.translation.y += selection_height_offset;
        box_transform.scale.y -= selection_height_offset * 2.0;
        render_frame.draw_mesh(
            Some(&RenderLayer::UI),
            mesh_handle,
            mat_handle,
            &box_transform,
        );
    }
}

fn draw_ui_node(
    render_frame: &mut RenderFrame,
    asset_manager: &AssetManager,
    carat_blink: bool,
    ui_config: &UiRuntimeConfig,
    ui_state: &UiState,
    ui_input_state: &UiInputState,
    text_icon_handle: &AssetHandle<IconData>,
    eye_icon_handle: &AssetHandle<IconData>,
    id: &NodeId,
) {
    let Some((width, height, child_offset_x, child_offset_y, child_offset_z)) =
        ui_state.cache.bounds(id)
    else {
        warn!("no bounds for id 1: {:?}", id);
        return;
    };

    let Some(node) = ui_config.get_node(&id) else {
        warn!("no node for id 1: {:?}", id);
        return;
    };
    let Some(node_visible) = ui_state.visibility_store.get_node_visibility(&id) else {
        warn!("no node for id 2: {:?}", id);
        return;
    };

    let mut transform = Transform::from_xyz(child_offset_x, child_offset_y, child_offset_z - 20.0);
    transform.scale.x = width;
    transform.scale.y = height;

    if node_visible {
        match node.widget_kind() {
            WidgetKind::Panel => {
                draw_ui_panel(render_frame, ui_config, ui_state, id, &transform);
            }
            WidgetKind::Text => {
                draw_ui_text(
                    render_frame,
                    asset_manager,
                    ui_config,
                    ui_state,
                    text_icon_handle,
                    id,
                    &transform,
                );
            }
            WidgetKind::Button => {
                draw_ui_button(
                    render_frame,
                    ui_config,
                    ui_state,
                    ui_input_state,
                    id,
                    &transform,
                );
            }
            WidgetKind::Textbox => {
                draw_ui_textbox(
                    render_frame,
                    asset_manager,
                    carat_blink,
                    ui_config,
                    ui_state,
                    ui_input_state,
                    text_icon_handle,
                    eye_icon_handle,
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
    ui_config: &UiRuntimeConfig,
    ui_state: &UiState,
    id: &NodeId,
    transform: &Transform,
) {
    let Some(panel_style_state) = ui_state.panel_style_state(ui_config, id) else {
        panic!("no panel ref for node_id: {:?}", id);
    };

    // draw panel
    if let Some(mat_handle) = panel_style_state.background_color_handle() {
        let background_alpha = ui_config.node_background_alpha(id);
        if background_alpha > 0.0 {
            if background_alpha != 1.0 {
                panic!("partial background_alpha not implemented yet!");
            }
            let box_handle = ui_state.globals.get_box_mesh_handle().unwrap();
            render_frame.draw_mesh(Some(&RenderLayer::UI), box_handle, &mat_handle, &transform);
        }
    } else {
        warn!("no color handle for panel"); // probably will need to do better debugging later
        return;
    };
}

fn draw_ui_text(
    //&self, // self was text widget
    render_frame: &mut RenderFrame,
    asset_manager: &AssetManager,
    ui_config: &UiRuntimeConfig,
    ui_state: &UiState,
    text_icon_handle: &AssetHandle<IconData>,
    id: &NodeId,
    transform: &Transform,
) {
    let Some(text_style_state) = ui_state.text_style_state(ui_config, id) else {
        panic!("no text style state ref for node_id: {:?}", id);
    };

    // draw background
    if let Some(mat_handle) = text_style_state.background_color_handle() {
        let background_alpha = ui_config.node_background_alpha(id);
        if background_alpha > 0.0 {
            if background_alpha != 1.0 {
                panic!("partial background_alpha not implemented yet!");
            }
            let box_handle = ui_state.globals.get_box_mesh_handle().unwrap();
            let mut new_transform = transform.clone();
            new_transform.translation.z += UiRuntimeConfig::Z_STEP_RENDER;
            render_frame.draw_mesh(
                Some(&RenderLayer::UI),
                box_handle,
                &mat_handle,
                &new_transform,
            );
        }
    } else {
        warn!("no background color handle for text"); // probably will need to do better debugging later
        return;
    };

    if let Some(mat_handle) = text_style_state.text_color_handle() {
        if let Some(text_ref) = ui_state.store.text_ref(id) {
            asset_manager.draw_text(
                render_frame,
                Some(&RenderLayer::UI),
                text_icon_handle,
                &mat_handle,
                transform,
                &text_ref.text,
            );
        }
    }
}

fn draw_ui_button(
    //self was Button
    render_frame: &mut RenderFrame,
    ui_config: &UiRuntimeConfig,
    ui_state: &UiState,
    ui_input_state: &UiInputState,
    id: &NodeId,
    transform: &Transform,
) {
    let Some(button_style_state) = ui_state.button_style_state(ui_config, id) else {
        panic!("no button style state ref for node_id: {:?}", id);
    };

    // draw button
    let active_state = ui_input_state.get_active_state(id);
    if let Some(mat_handle) = button_style_state.current_color_handle(active_state) {
        let background_alpha = ui_config.node_background_alpha(id);
        if background_alpha > 0.0 {
            if background_alpha != 1.0 {
                panic!("partial background_alpha not implemented yet!");
            }
            let box_handle = ui_state.globals.get_box_mesh_handle().unwrap();
            render_frame.draw_mesh(Some(&RenderLayer::UI), box_handle, &mat_handle, &transform);
        }
    } else {
        warn!("no color handle for button"); // probably will need to do better debugging later
        return;
    };
}

fn draw_ui_textbox(
    //self was Textbox
    render_frame: &mut RenderFrame,
    asset_manager: &AssetManager,
    carat_blink: bool,
    ui_config: &UiRuntimeConfig,
    ui_state: &UiState,
    ui_input_state: &UiInputState,
    text_icon_handle: &AssetHandle<IconData>,
    eye_icon_handle: &AssetHandle<IconData>,
    id: &NodeId,
    transform: &Transform,
) {
    let Some(textbox_state) = ui_state.store.textbox_ref(id) else {
        panic!("no textbox state for node_id: {:?}", id);
    };
    let Some(textbox_style_state) = ui_state.textbox_style_state(ui_config, id) else {
        panic!("no textbox style state for node_id: {:?}", id);
    };

    // draw textbox
    let active_state = ui_input_state.get_active_state(id);
    if let Some(mat_handle) = textbox_style_state.current_color_handle(active_state) {
        let background_alpha = ui_config.node_background_alpha(id);
        if background_alpha > 0.0 {
            if background_alpha != 1.0 {
                panic!("partial background_alpha not implemented yet!");
            }
            let box_handle = ui_state.globals.get_box_mesh_handle().unwrap();
            render_frame.draw_mesh(Some(&RenderLayer::UI), box_handle, &mat_handle, &transform);
        }
    } else {
        warn!("no color handle for textbox"); // probably will need to do better debugging later
        return;
    };

    // draw text
    if let Some(text_color_handle) = textbox_style_state.text_color_handle() {
        let password_masked_string = if textbox_state.password_mask { Some(textbox_state.get_masked_text()) } else { None };
        let textbox_text = if password_masked_string.is_some() {
            password_masked_string.as_ref().unwrap()
        } else {
            &textbox_state.text
        };

        // draw text
        let mut text_transform = transform.clone();
        text_transform.translation.x += 8.0;
        text_transform.scale.x -= 16.0;

        {
            text_transform.translation.z =
                transform.translation.z + (UiRuntimeConfig::Z_STEP_RENDER * 2.0);
            asset_manager.draw_text(
                render_frame,
                Some(&RenderLayer::UI),
                text_icon_handle,
                &text_color_handle,
                &text_transform,
                &textbox_text[textbox_state.offset_index..textbox_text.len()],
            );
        }

        if active_state == NodeActiveState::Active {
            // draw selection box if needed
            if let Some(select_index) = ui_input_state.select_index {
                if let Some(mat_handle) = textbox_style_state.select_color_handle() {
                    text_transform.translation.z =
                        transform.translation.z + (UiRuntimeConfig::Z_STEP_RENDER * 1.0);
                    UiRenderer::draw_text_selection(
                        render_frame,
                        asset_manager,
                        text_icon_handle,
                        ui_state.globals.get_box_mesh_handle().unwrap(),
                        &mat_handle,
                        &text_transform,
                        textbox_text,
                        textbox_state.offset_index,
                        select_index,
                        ui_input_state.carat_index,
                    );
                }
            }

            // draw carat if needed
            if carat_blink {
                text_transform.translation.z =
                    transform.translation.z + (UiRuntimeConfig::Z_STEP_RENDER * 2.0);
                UiRenderer::draw_text_carat(
                    render_frame,
                    asset_manager,
                    text_icon_handle,
                    &text_color_handle,
                    &text_transform,
                    textbox_text,
                    textbox_state.offset_index,
                    ui_input_state.carat_index,
                );
            }

            let textbox = ui_config.get_node(id).unwrap().widget_textbox_ref().unwrap();
            if textbox.is_password {
                let currently_masked = textbox_state.password_mask;

                let mut eye_transform = transform.clone();

                let eye_size = transform.scale.y * 0.5;
                eye_transform.translation.x += transform.scale.x - (eye_size * 1.2);
                eye_transform.translation.y += eye_size;
                eye_transform.translation.z = transform.translation.z + (UiRuntimeConfig::Z_STEP_RENDER * 3.0);

                let eye_size = (transform.scale.y / 100.0) * 0.8 * if textbox_state.eye_hover { 1.2 } else { 1.0 };
                eye_transform.scale.x = eye_size;
                eye_transform.scale.y = eye_size * 0.9;

                asset_manager.draw_icon(
                    render_frame,
                    eye_icon_handle,
                    if currently_masked { 1 } else { 0 },
                    &eye_transform,
                    Some(&RenderLayer::UI),
                );
            }
        }
    }
}
