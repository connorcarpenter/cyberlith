use bevy_log::warn;

use asset_loader::{AssetHandle, AssetManager, IconData, UiConfigData, UiTextMeasurer};
use asset_render::AssetRender;
use render_api::{
    base::{CpuMaterial, CpuMesh},
    components::{RenderLayer, Transform},
    resources::RenderFrame,
};
use storage::Handle;
use ui_types::{NodeId, UiConfig, WidgetKind};
use ui_state::{NodeActiveState, UiState};
use ui_builder::{ButtonStyleRef, PanelStyleRef, TextboxStyleRef, TextStyleRef};
use ui_input::UiInputState;
use ui_loader::{Blinkiness, UiManager};

pub struct UiRenderer;

impl UiRenderer {

    pub fn draw_ui(
        ui_manager: &UiManager,
        asset_manager: &AssetManager,
        render_frame: &mut RenderFrame,
        render_layer_opt: Option<&RenderLayer>,
        blinkiness: &Blinkiness,
        ui_handle: &AssetHandle<UiConfigData>,
    ) {
        let Some(ui_data) = ui_manager.ui_configs.get(ui_handle) else {
            warn!("ui data not loaded 2: {:?}", ui_handle.asset_id());
            return;
        };
        let ui = ui_data.get_ui_config_ref();

        let Some(ui_state_data) = ui_manager.ui_states.get(ui_handle) else {
            warn!("ui state data not loaded 2: {:?}", ui_handle.asset_id());
            return;
        };
        let ui_state = ui_state_data.get_ui_state_ref();
        let Some(ui_input_state) = ui_manager.ui_input_states.get(ui_handle) else {
            warn!("ui input state data not loaded 2: {:?}", ui_handle.asset_id());
            return;
        };

        let text_icon_handle = ui_data.get_icon_handle();

        let carat_blink = blinkiness.enabled() || ui_input_state.interact_timer_was_recent();

        for node_id in 0..ui.store.nodes.len() {
            let node_id = NodeId::from_usize(node_id);
            draw_ui_node(
                render_frame,
                render_layer_opt,
                asset_manager,
                carat_blink,
                &ui,
                &ui_state,
                ui_input_state,
                &text_icon_handle,
                &node_id,
            );
        }
    }

    pub fn draw_text_carat(
        render_frame: &mut RenderFrame,
        render_layer_opt: Option<&RenderLayer>,
        asset_manager: &AssetManager,
        text_icon_handle: &AssetHandle<IconData>,
        text_color_mat_handle: &Handle<CpuMaterial>,
        transform: &Transform,
        text: &str,
        carat_index: usize,
    ) {
        let Some(icon_data) = asset_manager.get_store().icons.get(text_icon_handle) else {
            return;
        };
        let text_measurer = UiTextMeasurer::new(icon_data);
        let subimage_indices = ui_types::Text::get_subimage_indices(text);
        let (x_positions, text_height) = ui_types::Text::get_raw_text_rects(&text_measurer, &subimage_indices);

        let mut cursor = Transform::from_xyz(
            0.0,
            transform.translation.y + (transform.scale.y * 0.5),
            transform.translation.z,
        );

        // if we want to fill 200px, but raw_width is 100px, then scale_x would be 2.0
        cursor.scale.y = transform.scale.y / text_height;
        cursor.scale.x = cursor.scale.y;

        cursor.translation.x = transform.translation.x + (x_positions[carat_index] * cursor.scale.x);

        asset_manager.draw_icon_with_material(
            render_frame,
            render_layer_opt,
            text_icon_handle,
            text_color_mat_handle,
            (124 - 32) as usize, // pipe character '|'
            &cursor,
        );
    }

    pub fn draw_text_selection(
        render_frame: &mut RenderFrame,
        render_layer_opt: Option<&RenderLayer>,
        asset_manager: &AssetManager,
        text_icon_handle: &AssetHandle<IconData>,
        mesh_handle: &Handle<CpuMesh>,
        mat_handle: &Handle<CpuMaterial>,
        transform: &Transform,
        text: &str,
        select_index: usize,
        carat_index: usize,
    ) {
        let Some(icon_data) = asset_manager.get_store().icons.get(text_icon_handle) else {
            return;
        };
        let text_measurer = UiTextMeasurer::new(icon_data);
        let subimage_indices = ui_types::Text::get_subimage_indices(text);
        let (x_positions, text_height) = ui_types::Text::get_raw_text_rects(&text_measurer, &subimage_indices);
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
}

fn draw_ui_node(
    render_frame: &mut RenderFrame,
    render_layer_opt: Option<&RenderLayer>,
    asset_manager: &AssetManager,
    carat_blink: bool,
    ui_config: &UiConfig,
    ui_state: &UiState,
    ui_input_state: &UiInputState,
    text_icon_handle: &AssetHandle<IconData>,
    id: &NodeId,
) {
    let Some((width, height, child_offset_x, child_offset_y, child_offset_z)) = ui_state.cache.bounds(id) else {
        warn!("no bounds for id 1: {:?}", id);
        return;
    };

    let Some(node) = ui_config.store.get_node(&id) else {
        warn!("no node for id 1: {:?}", id);
        return;
    };
    let Some(node_visible) = ui_state.visibility_store.get_node_visibility(&id) else {
        warn!("no node for id 2: {:?}", id);
        return;
    };

    let mut transform = Transform::from_xyz(
        child_offset_x,
        child_offset_y,
        child_offset_z,
    );
    transform.scale.x = width;
    transform.scale.y = height;

    if node_visible {
        match node.widget_kind() {
            WidgetKind::Panel => {
                draw_ui_panel(
                    render_frame,
                    render_layer_opt,
                    ui_config,
                    ui_state,
                    id,
                    &transform,
                );
            }
            WidgetKind::Text => {
                draw_ui_text(
                    render_frame,
                    render_layer_opt,
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
                    render_layer_opt,
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
                    render_layer_opt,
                    asset_manager,
                    carat_blink,
                    ui_config,
                    ui_state,
                    ui_input_state,
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
    ui_config: &UiConfig,
    ui_state: &UiState,
    node_id: &NodeId,
    transform: &Transform,
) {
    let Some(panel_state_ref) = ui_state.store.panel_ref(node_id) else {
        panic!("no panel ref for node_id: {:?}", node_id);
    };

    // draw panel
    if let Some(mat_handle) = panel_state_ref.background_color_handle {
        let panel_style_ref = PanelStyleRef::new(&ui_config.store, *node_id);
        let background_alpha = panel_style_ref.background_alpha();
        if background_alpha > 0.0 {
            if background_alpha != 1.0 {
                panic!("partial background_alpha not implemented yet!");
            }
            let box_handle = ui_state.globals.get_box_mesh_handle().unwrap();
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
    asset_manager: &AssetManager,
    ui_config: &UiConfig,
    ui_state: &UiState,
    text_icon_handle: &AssetHandle<IconData>,
    node_id: &NodeId,
    transform: &Transform,
) {
    let Some(text_ref) = ui_config
        .store
        .get_node(node_id)
        .unwrap()
        .widget_text_ref() else {
        panic!("no text ref for node_id: {:?}", node_id);
    };
    let Some(text_state_ref) = ui_state
        .store
        .get_node(node_id)
        .unwrap()
        .widget_text_ref() else {
        panic!("no text ref for node_id: {:?}", node_id);
    };

    // draw background
    if let Some(mat_handle) = text_state_ref.background_color_handle {
        let text_style_ref = TextStyleRef::new(&ui_config.store, *node_id);
        let background_alpha = text_style_ref.background_alpha();
        if background_alpha > 0.0 {
            if background_alpha != 1.0 {
                panic!("partial background_alpha not implemented yet!");
            }
            let box_handle = ui_state.globals.get_box_mesh_handle().unwrap();
            let mut new_transform = transform.clone();
            new_transform.translation.z -= 0.025;
            render_frame.draw_mesh(render_layer_opt, box_handle, &mat_handle, &new_transform);
        }
    } else {
        warn!("no background color handle for text"); // probably will need to do better debugging later
        return;
    };

    let Some(text_color_handle) = ui_state.globals.get_text_color_handle() else {
        panic!("No text color handle found in globals");
    };

    asset_manager.draw_text(
        render_frame,
        render_layer_opt,
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
    ui_config: &UiConfig,
    ui_state: &UiState,
    ui_input_state: &UiInputState,
    node_id: &NodeId,
    transform: &Transform,
) {
    let Some(button_state_ref) = ui_state.store.button_ref(node_id) else {
        panic!("no button ref for node_id: {:?}", node_id);
    };

    // draw button
    let active_state = ui_input_state.get_active_state(node_id);
    if let Some(mat_handle) = button_state_ref.current_color_handle(active_state) {
        let button_style_ref = ButtonStyleRef::new(&ui_config.store, *node_id);
        let background_alpha = button_style_ref.background_alpha();
        if background_alpha > 0.0 {
            if background_alpha != 1.0 {
                panic!("partial background_alpha not implemented yet!");
            }
            let box_handle = ui_state.globals.get_box_mesh_handle().unwrap();
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
    asset_manager: &AssetManager,
    carat_blink: bool,
    ui_config: &UiConfig,
    ui_state: &UiState,
    ui_input_state: &UiInputState,
    text_icon_handle: &AssetHandle<IconData>,
    node_id: &NodeId,
    transform: &Transform,
) {
    let Some(textbox_state_ref) = ui_state.store.textbox_ref(node_id) else {
        panic!("no textbox state ref for node_id: {:?}", node_id);
    };
    let Some(textbox_input_state_ref) = ui_input_state.store.textbox_ref(node_id) else {
        panic!("no textbox input state ref for node_id: {:?}", node_id);
    };

    // draw textbox
    let active_state = ui_input_state.get_active_state(node_id);
    if let Some(mat_handle) = textbox_state_ref.current_color_handle(active_state) {
        let textbox_style_ref = TextboxStyleRef::new(&ui_config.store, *node_id);
        let background_alpha = textbox_style_ref.background_alpha();
        if background_alpha > 0.0 {
            if background_alpha != 1.0 {
                panic!("partial background_alpha not implemented yet!");
            }
            let box_handle = ui_state.globals.get_box_mesh_handle().unwrap();
            render_frame.draw_mesh(render_layer_opt, box_handle, &mat_handle, &transform);
        }
    } else {
        warn!("no color handle for textbox"); // probably will need to do better debugging later
        return;
    };

    // draw text
    let Some(text_color_handle) = ui_state.globals.get_text_color_handle() else {
        panic!("No text color handle found in globals");
    };

    // draw text
    let mut text_transform = transform.clone();
    text_transform.translation.x += 8.0;

    {
        text_transform.translation.z = transform.translation.z + 0.05;

        asset_manager.draw_text(
            render_frame,
            render_layer_opt,
            text_icon_handle,
            text_color_handle,
            &text_transform,
            &textbox_state_ref.text,
        );
    }

    if active_state == NodeActiveState::Active {

        // draw selection box if needed
        if let Some(select_index) = textbox_input_state_ref.select_index {
            if let Some(mat_handle) = textbox_state_ref.get_selection_color_handle() {
                text_transform.translation.z = transform.translation.z + 0.025;
                UiRenderer::draw_text_selection(
                    render_frame,
                    render_layer_opt,
                    asset_manager,
                    text_icon_handle,
                    ui_state.globals.get_box_mesh_handle().unwrap(),
                    &mat_handle,
                    &text_transform,
                    &textbox_state_ref.text,
                    select_index,
                    textbox_input_state_ref.carat_index,
                );
            }
        }

        // draw carat if needed
        if carat_blink {
            text_transform.translation.z = transform.translation.z + 0.05;
            UiRenderer::draw_text_carat(
                render_frame,
                render_layer_opt,
                asset_manager,
                text_icon_handle,
                text_color_handle,
                &text_transform,
                &textbox_state_ref.text,
                textbox_input_state_ref.carat_index,
            );
        }
    }
}
