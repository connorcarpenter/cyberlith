use std::str::FromStr;

use ascii::AsciiString;

use asset_id::AssetId;
use logging::{info, warn};
use render_api::{
    base::{Color, CpuMaterial},
    components::Viewport,
};
use storage::Storage;
use ui_runner_config::{
    LayoutCache, NodeId, TextMeasurer, UiNode, UiRuntimeConfig, UiVisibilityStore, WidgetKind,
};

use crate::{
    button::ButtonStyleState, panel::PanelStyleState, spinner::SpinnerStyleState,
    state_store::UiStateStore, style_state::StyleState, text::TextStyleState,
    textbox::TextboxState, textbox::TextboxStyleState, widget::WidgetState, UiNodeState,
};

pub struct UiState {
    pub cache: LayoutCache,
    pub store: UiStateStore,
    pub visibility_store: UiVisibilityStore,

    ms_since_startup: f32,
}

impl UiState {
    pub fn from_ui_config(asset_id: &AssetId, ui_config: &UiRuntimeConfig) -> Self {
        let mut me = Self {
            cache: LayoutCache::new(),
            store: UiStateStore::new(asset_id),
            visibility_store: UiVisibilityStore::new(),

            ms_since_startup: 0.0,
        };

        for (id, node) in ui_config.nodes_iter() {
            me.add_node(id, &node);
        }

        for style in ui_config.styles_iter() {
            me.style_state_init(&style.widget_style.kind())
        }

        me
    }

    pub fn add_node(&mut self, id: &NodeId, node: &UiNode) {
        self.store.add_node(id, node);
        self.visibility_store.add_node(id, node.init_visible);
    }

    pub fn delete_node(&mut self, node: &NodeId) {
        self.store.delete_node(node);
        self.visibility_store.delete_node(node);
    }

    pub fn style_state_init(&mut self, widget_kind: &WidgetKind) {
        self.store.style_state_init(widget_kind);
    }

    pub fn update(&mut self, delta_ms: f32) {
        self.ms_since_startup += delta_ms;
    }

    pub fn time_since_startup(&self) -> f32 {
        self.ms_since_startup
    }

    // nodes

    pub(crate) fn node_ref(&self, id: &NodeId) -> Option<&UiNodeState> {
        self.store.get_node(&id)
    }

    pub(crate) fn node_mut(&mut self, id: &NodeId) -> Option<&mut UiNodeState> {
        self.store.get_node_mut(&id)
    }

    pub fn get_button_enabled(&self, node_id: &NodeId) -> Option<bool> {
        let node = self.store.get_node(node_id)?;
        let button = node.widget_button_ref()?;
        Some(button.enabled)
    }

    pub fn set_button_enabled(&mut self, node_id: &NodeId, val: bool) {
        let Some(node) = self.store.get_node_mut(node_id) else {
            return;
        };
        let Some(button) = node.widget_button_mut() else {
            return;
        };
        button.enabled = val;
    }

    pub fn textbox_ref(&self, id: &NodeId) -> Option<&TextboxState> {
        let node_ref = self.node_ref(id)?;
        let textbox_ref = node_ref.widget_textbox_ref()?;
        Some(textbox_ref)
    }

    pub fn textbox_mut(&mut self, id: &NodeId) -> Option<&mut TextboxState> {
        let node_mut = self.node_mut(id)?;
        let textbox_mut = node_mut.widget_textbox_mut()?;
        Some(textbox_mut)
    }

    pub fn get_textbox_text(&self, node_id: &NodeId) -> Option<String> {
        let node = self.store.get_node(node_id)?;
        let textbox = node.widget_textbox_ref()?;
        Some(textbox.text.to_string())
    }

    pub fn set_textbox_text(&mut self, node_id: &NodeId, val: &str) {
        let Some(node) = self.store.get_node_mut(node_id) else {
            return;
        };
        let Some(textbox) = node.widget_textbox_mut() else {
            return;
        };
        textbox.text = AsciiString::from_str(val).unwrap();
    }

    pub fn get_text(&self, node_id: &NodeId) -> Option<String> {
        let node = self.store.get_node(node_id)?;
        let text = node.widget_text_ref()?;
        Some(text.text.clone())
    }

    pub fn set_text(&mut self, node_id: &NodeId, val: &str) {
        let Some(node) = self.store.get_node_mut(node_id) else {
            warn!("set_text: node not found for node_id: {:?}", node_id);
            return;
        };
        match &mut node.widget {
            WidgetState::Text(text) => {
                text.text = val.to_string();
            }
            WidgetState::Textbox(textbox) => {
                textbox.text = AsciiString::from_str(val).unwrap();
            }
            _ => {
                warn!(
                    "set_text: node is not a text widget for node_id: {:?}",
                    node_id
                );
            }
        }
    }

    pub fn set_textbox_password_eye_visible(&mut self, node_id: &NodeId, val: bool) {
        let Some(node) = self.store.get_node_mut(node_id) else {
            logging::warn!(
                "set_textbox_password_eye_visible: node not found for node_id: {:?}",
                node_id
            );
            return;
        };
        let Some(textbox) = node.widget_textbox_mut() else {
            return;
        };
        // password eye visible == password mask off
        textbox.password_mask = !val;
    }

    pub fn set_node_visible(&mut self, node_id: &NodeId, val: bool) {
        self.visibility_store.set_node_visibility(node_id, val);
    }

    pub fn get_ui_container_asset_id_opt(&self, node_id: &NodeId) -> Option<AssetId> {
        let Some(node) = self.store.get_node(node_id) else {
            logging::warn!(
                "get_ui_container_asset_id_opt: node not found for node_id: {:?}",
                node_id
            );
            return None;
        };
        let Some(ui_container) = node.widget_ui_container_ref() else {
            return None;
        };
        ui_container.ui_handle_opt
    }

    pub fn set_ui_container_asset_id(&mut self, node_id: &NodeId, asset_id: &AssetId) {
        let Some(node) = self.store.get_node_mut(node_id) else {
            logging::warn!(
                "set_ui_container_asset_id: node not found for node_id: {:?}",
                node_id
            );
            return;
        };
        let Some(ui_container) = node.widget_ui_container_mut() else {
            return;
        };
        info!(
            "set_ui_container_asset_id for node {:?} -> asset_id: {:?}",
            node_id, asset_id
        );
        ui_container.ui_handle_opt = Some(asset_id.clone());
    }

    pub fn clear_ui_container(&mut self, node_id: &NodeId) {
        let Some(node) = self.store.get_node_mut(node_id) else {
            logging::warn!(
                "clear_ui_container: node not found for node_id: {:?}",
                node_id
            );
            return;
        };
        let Some(ui_container) = node.widget_ui_container_mut() else {
            return;
        };
        ui_container.ui_handle_opt = None;
    }

    // styles

    fn node_style_state(&self, config: &UiRuntimeConfig, id: &NodeId) -> Option<&StyleState> {
        let node = config.get_node(id)?;
        let widget_kind = node.widget_kind();
        let style_id = node.style_id();
        self.store.get_style_state(widget_kind, style_id)
    }

    pub fn panel_style_state(
        &self,
        config: &UiRuntimeConfig,
        id: &NodeId,
    ) -> Option<&PanelStyleState> {
        let style_state = self.node_style_state(config, id)?;
        let StyleState::Panel(panel_style_state) = style_state else {
            return None;
        };
        return Some(panel_style_state);
    }

    pub fn text_style_state(
        &self,
        config: &UiRuntimeConfig,
        id: &NodeId,
    ) -> Option<&TextStyleState> {
        let style_state = self.node_style_state(config, id)?;
        let StyleState::Text(text_style_state) = style_state else {
            return None;
        };
        return Some(text_style_state);
    }

    pub fn button_style_state(
        &self,
        config: &UiRuntimeConfig,
        id: &NodeId,
    ) -> Option<&ButtonStyleState> {
        let style_state = self.node_style_state(config, id)?;
        let StyleState::Button(button_style_state) = style_state else {
            return None;
        };
        return Some(button_style_state);
    }

    pub fn textbox_style_state(
        &self,
        config: &UiRuntimeConfig,
        id: &NodeId,
    ) -> Option<&TextboxStyleState> {
        let style_state = self.node_style_state(config, id)?;
        let StyleState::Textbox(textbox_style_state) = style_state else {
            return None;
        };
        return Some(textbox_style_state);
    }

    pub fn spinner_style_state(
        &self,
        config: &UiRuntimeConfig,
        id: &NodeId,
    ) -> Option<&SpinnerStyleState> {
        let style_state = self.node_style_state(config, id)?;
        let StyleState::Spinner(spinner_style_state) = style_state else {
            return None;
        };
        return Some(spinner_style_state);
    }

    pub fn load_cpu_data(
        &mut self,
        ui_handle: &AssetId,
        ui_config: &UiRuntimeConfig,
        materials: &mut Storage<CpuMaterial>,
    ) {
        // set color handles
        for id in std::mem::take(&mut self.store.nodes_needing_cpu_data) {
            let Some(node) = ui_config.get_node(&id) else {
                panic!(
                    "error in load_cpu_data! in ui: {:?}, no node for id: {:?}!",
                    ui_handle, id
                );
            };
            let widget_kind = node.widget_kind();
            let style_id = node.style_id();

            // info!("(ui: {:?}) load_cpu_data: nodeid: {:?}, widget_kind: {:?}, style_id: {:?}", ui_handle, id, widget_kind, style_id);

            match widget_kind {
                WidgetKind::Panel => {
                    if let Some(panel_style_mut) = self.store.create_panel_style(style_id) {
                        let background_color = ui_config
                            .node_background_color(&id)
                            .copied()
                            .unwrap_or(Color::BLACK);
                        let background_color_handle = materials.add(background_color);
                        panel_style_mut.set_background_color_handle(background_color_handle);
                    } else {
                        // warn!("do not need to load cpu data for style: {:?}", style_id);
                    }
                }
                WidgetKind::Text => {
                    if let Some(text_style_mut) = self.store.create_text_style(style_id) {
                        // background color
                        let background_color = ui_config
                            .node_background_color(&id)
                            .copied()
                            .unwrap_or(Color::BLACK);
                        let background_color_handle = materials.add(background_color);
                        text_style_mut.set_background_color_handle(background_color_handle);

                        // text color
                        let text_color = ui_config
                            .node_text_color(&id)
                            .copied()
                            .unwrap_or(Color::WHITE);
                        let text_color_handle = materials.add(text_color);
                        text_style_mut.set_text_color_handle(text_color_handle);
                    } else {
                        // warn!("do not need to load cpu data for style: {:?}", style_id);
                    }
                }
                WidgetKind::Button => {
                    if let Some(button_style_mut) = self.store.create_button_style(style_id) {
                        // background color
                        let background_color = ui_config
                            .node_background_color(&id)
                            .copied()
                            .unwrap_or(Color::BLACK);
                        let background_color_handle = materials.add(background_color);
                        button_style_mut.set_background_color_handle(background_color_handle);

                        // button-specific
                        let button_style = ui_config.button_style(&id);
                        // hover color
                        let hover_color = button_style
                            .map(|style| style.hover_color)
                            .flatten()
                            .unwrap_or(Color::BLACK);
                        let hover_color_handle = materials.add(hover_color);
                        button_style_mut.set_hover_color_handle(hover_color_handle);

                        // down color
                        let down_color = button_style
                            .map(|style| style.down_color)
                            .flatten()
                            .unwrap_or(Color::BLACK);
                        let down_color_handle = materials.add(down_color);
                        button_style_mut.set_down_color_handle(down_color_handle);

                        // disabled color
                        let disabled_color = button_style
                            .map(|style| style.disabled_color)
                            .flatten()
                            .unwrap_or(Color::BLACK);
                        let disabled_color_handle = materials.add(disabled_color);
                        button_style_mut.set_disabled_color_handle(disabled_color_handle);
                    } else {
                        // warn!("do not need to load cpu data for style: {:?}", style_id);
                    }
                }
                WidgetKind::Textbox => {
                    if let Some(textbox_style_mut) = self.store.create_textbox_style(style_id) {
                        // background color
                        let background_color = ui_config
                            .node_background_color(&id)
                            .copied()
                            .unwrap_or(Color::BLACK);
                        let background_color_handle = materials.add(background_color);
                        textbox_style_mut.set_background_color_handle(background_color_handle);

                        // text color
                        let text_color = ui_config
                            .node_text_color(&id)
                            .copied()
                            .unwrap_or(Color::WHITE);
                        let text_color_handle = materials.add(text_color);
                        textbox_style_mut.set_text_color_handle(text_color_handle);

                        // textbox-specific
                        let textbox_style = ui_config.textbox_style(&id);
                        // hover color
                        let hover_color = textbox_style
                            .map(|style| style.hover_color)
                            .flatten()
                            .unwrap_or(Color::BLACK);
                        let hover_color_handle = materials.add(hover_color);
                        textbox_style_mut.set_hover_color_handle(hover_color_handle);

                        // active color
                        let active_color = textbox_style
                            .map(|style| style.active_color)
                            .flatten()
                            .unwrap_or(Color::BLACK);
                        let active_color_handle = materials.add(active_color);
                        textbox_style_mut.set_active_color_handle(active_color_handle);

                        // select color
                        let select_color = textbox_style
                            .map(|style| style.select_color)
                            .flatten()
                            .unwrap_or(Color::BLACK);
                        let select_color_handle = materials.add(select_color);
                        textbox_style_mut.set_select_color_handle(select_color_handle);
                    } else {
                        // warn!("do not need to load cpu data for style: {:?}", style_id);
                    }
                }
                WidgetKind::Spinner => {
                    if let Some(spinner_style_mut) = self.store.create_spinner_style(style_id) {
                        // background color
                        let background_color = ui_config
                            .node_background_color(&id)
                            .copied()
                            .unwrap_or(Color::BLACK);
                        let background_color_handle = materials.add(background_color);
                        spinner_style_mut.set_background_color_handle(background_color_handle);

                        // spinner color
                        let spinner_color = ui_config
                            .node_spinner_color(&id)
                            .copied()
                            .unwrap_or(Color::WHITE);
                        let spinner_color_handle = materials.add(spinner_color);
                        spinner_style_mut.set_spinner_color_handle(spinner_color_handle);
                    } else {
                        // warn!("do not need to load cpu data for style: {:?}", style_id);
                    }
                }
                WidgetKind::UiContainer => self.store.create_ui_container_style(style_id),
            }
        }
    }

    // layout

    pub fn recalculate_layout(
        &mut self,
        ui_config: &UiRuntimeConfig,
        text_measurer: &dyn TextMeasurer,
        viewport: &Viewport,
        z: f32,
    ) -> Vec<(AssetId, Viewport, f32)> {
        //info!("recalculating layout. viewport_width: {:?}, viewport_height: {:?}", viewport.width, viewport.height);

        let last_viewport_width: f32 = viewport.width as f32;
        let last_viewport_height: f32 = viewport.height as f32;

        let cache_mut = &mut self.cache;
        let state_store_ref = &self.store;
        let visibility_store_ref = &self.visibility_store;

        // this calculates all the rects in cache_mut
        UiRuntimeConfig::ROOT_NODE_ID.layout(
            cache_mut,
            ui_config,
            state_store_ref,
            visibility_store_ref,
            text_measurer,
            last_viewport_width,
            last_viewport_height,
        );

        let mut children = Vec::new();
        finalize_rects(
            ui_config,
            self,
            &mut children,
            &UiRuntimeConfig::ROOT_NODE_ID,
            (viewport.x as f32, viewport.y as f32, z),
        );

        // print_node(&Self::ROOT_PANEL_ID, &self.cache, &self.panels, true, false, "".to_string());

        children
    }
}

// recurses through tree and sets the bounds of each node to their absolute position
fn finalize_rects(
    ui_config: &UiRuntimeConfig,
    ui_state: &mut UiState,
    child_uis_output: &mut Vec<(AssetId, Viewport, f32)>,
    id: &NodeId,
    parent_position: (f32, f32, f32),
) {
    let Some(node) = ui_config.get_node(&id) else {
        warn!("no panel for id: {:?}", id);
        return;
    };

    let Some((width, height, child_offset_x, child_offset_y, _)) = ui_state.cache.bounds(id) else {
        // warn!("no bounds for id 3: {:?}", id);
        return;
    };

    let child_position = (
        parent_position.0 + child_offset_x,
        parent_position.1 + child_offset_y,
        parent_position.2 + UiRuntimeConfig::Z_STEP_RENDER,
    );

    ui_state.cache.set_bounds(
        id,
        child_position.0,
        child_position.1,
        child_position.2,
        width,
        height,
    );

    match node.widget_kind() {
        WidgetKind::Panel => {
            let Some(panel_ref) = ui_config.panel_ref(id) else {
                panic!("no panel ref for node_id: {:?}", id);
            };

            // update children
            let child_ids = panel_ref.children.clone();
            for child_id in child_ids {
                finalize_rects(
                    ui_config,
                    ui_state,
                    child_uis_output,
                    &child_id,
                    child_position,
                );
            }
        }
        WidgetKind::Button => {
            let Some(button_ref) = ui_config.button_ref(id) else {
                panic!("no button ref for node_id: {:?}", id);
            };
            let panel_ref = &button_ref.panel;

            // update children
            let child_ids = panel_ref.children.clone();
            for child_id in child_ids {
                finalize_rects(
                    ui_config,
                    ui_state,
                    child_uis_output,
                    &child_id,
                    child_position,
                );
            }
        }
        WidgetKind::UiContainer => {
            if let Some(asset_id) = ui_state.get_ui_container_asset_id_opt(id) {
                let mut viewport = Viewport::default();
                viewport.x = child_position.0 as i32;
                viewport.y = child_position.1 as i32;
                viewport.width = width as u32;
                viewport.height = height as u32;

                let z = child_position.2;

                child_uis_output.push((asset_id, viewport, z));
            }
        }
        _ => {}
    }
}
