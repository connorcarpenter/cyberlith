
use bevy_log::warn;

use render_api::{
    base::{CpuMaterial, CpuMesh},
    components::Viewport,
    shapes::UnitSquare,
};
use storage::{Handle, Storage};
use render_api::base::Color;

use ui_types::{NodeId, UiConfig, UiRuntimeConfig, UiVisibilityStore, WidgetKind};
use ui_layout::{Cache, Node, TextMeasurer};

use crate::{
    button::ButtonState, cache::LayoutCache, panel::PanelState, state_store::UiStateStore,
    text::TextState, textbox::TextboxState,
    UiNodeState, widget::WidgetState
};

pub struct UiState {
    pub globals: StateGlobals,
    pub cache: LayoutCache,
    pub store: UiStateStore,
    pub visibility_store: UiVisibilityStore,

    recalc_layout: bool,
    last_viewport: Viewport,
}

impl UiState {

    pub fn from_ui_config(ui_config: &UiConfig) -> Self {
        let mut me = Self {
            globals: StateGlobals::new(),
            cache: LayoutCache::new(),
            store: UiStateStore::new(),
            visibility_store: UiVisibilityStore::new(),

            recalc_layout: false,
            last_viewport: Viewport::new_at_origin(0, 0),
        };

        for node in ui_config.store.nodes_iter() {
            me.store.node_state_init(node);
            me.visibility_store.node_state_init();
        }

        me
    }

    pub fn load_cpu_data(
        &mut self,
        ui_config: &UiRuntimeConfig,
        meshes: &mut Storage<CpuMesh>,
        materials: &mut Storage<CpuMaterial>,
    ) {
        // set box mesh handle
        {
            let mesh_handle = meshes.add(UnitSquare);
            self.globals.box_mesh_handle_opt = Some(mesh_handle);
        }

        // set text color handle
        {
            let mat_handle = materials.add(ui_config.get_text_color());
            self.globals.text_color_handle_opt = Some(mat_handle);
        }

        // set color handles
        let ids = self.collect_color_handles();
        for id in ids {
            let node_ref = ui_config.get_node(&id).unwrap();

            match node_ref.widget_kind() {
                WidgetKind::Panel => {
                    let background_color = ui_config.node_background_color(&id).copied().unwrap_or(Color::BLACK);
                    let panel_mut = self.panel_mut(&id).unwrap();
                    let mat_handle = materials.add(background_color);
                    panel_mut.background_color_handle = Some(mat_handle);
                }
                WidgetKind::Text => {
                    let background_color = ui_config.node_background_color(&id).copied().unwrap_or(Color::BLACK);
                    let text_mut = self.text_mut(&id).unwrap();
                    let mat_handle = materials.add(background_color);
                    text_mut.background_color_handle = Some(mat_handle);
                }
                WidgetKind::Button => {
                    let background_color = ui_config.node_background_color(&id).copied().unwrap_or(Color::BLACK);
                    let button_style = ui_config.button_style(&id);
                    let hover_color = button_style.map(|style| style.hover_color).flatten().unwrap_or(Color::BLACK);
                    let down_color = button_style.map(|style| style.down_color).flatten().unwrap_or(Color::BLACK);

                    let button_mut = self.button_mut(&id).unwrap();

                    let background_color_handle = materials.add(background_color);
                    button_mut.panel.background_color_handle = Some(background_color_handle);

                    let hover_color_handle = materials.add(hover_color);
                    button_mut.set_hover_color_handle(hover_color_handle);

                    let down_color_handle = materials.add(down_color);
                    button_mut.set_down_color_handle(down_color_handle);
                }
                WidgetKind::Textbox => {
                    let background_color = ui_config.node_background_color(&id).copied().unwrap_or(Color::BLACK);
                    let textbox_style = ui_config.textbox_style(&id);
                    let hover_color = textbox_style.map(|style| style.hover_color).flatten().unwrap_or(Color::BLACK);
                    let active_color = textbox_style.map(|style| style.active_color).flatten().unwrap_or(Color::BLACK);
                    let select_color = textbox_style.map(|style| style.select_color).flatten().unwrap_or(Color::BLACK);

                    let textbox_mut = self.textbox_mut(&id).unwrap();

                    let background_color_handle = materials.add(background_color);
                    textbox_mut.panel.background_color_handle = Some(background_color_handle);

                    let hover_color_handle = materials.add(hover_color);
                    textbox_mut.set_hover_color_handle(hover_color_handle);

                    let active_color_handle = materials.add(active_color);
                    textbox_mut.set_active_color_handle(active_color_handle);

                    let select_color_handle = materials.add(select_color);
                    textbox_mut.set_selection_color_handle(select_color_handle);
                }
            }
        }
    }

    pub fn collect_color_handles(&mut self) -> Vec<NodeId> {
        let mut pending_mat_handles = Vec::new();
        for id in self.store.node_ids() {
            let Some(node_ref) = self.node_ref(&id) else {
                continue;
            };
            match &node_ref.widget {
                WidgetState::Panel(panel_state) => {
                    if panel_state.background_color_handle.is_none() {
                        pending_mat_handles.push(id);
                    }
                }
                WidgetState::Text(text_state) => {
                    if text_state.background_color_handle.is_none() {
                        pending_mat_handles.push(id);
                    }
                }
                WidgetState::Button(button_state) => {
                    if button_state.needs_color_handle() {
                        pending_mat_handles.push(id);
                    }
                }
                WidgetState::Textbox(txtbox_state) => {
                    if txtbox_state.needs_color_handle() {
                        pending_mat_handles.push(id);
                    }
                }
            }
        }
        pending_mat_handles
    }

    // interface

    pub fn update_viewport(&mut self, viewport: &Viewport) {
        let viewport = *viewport;
        if self.last_viewport == viewport {
            return;
        }
        self.last_viewport = viewport;
        self.queue_recalculate_layout();
    }

    fn queue_recalculate_layout(&mut self) {
        self.recalc_layout = true;
    }

    pub fn needs_to_recalculate_layout(&self) -> bool {
        self.recalc_layout
    }

    pub fn recalculate_layout(&mut self, ui_config: &UiRuntimeConfig, text_measurer: &dyn TextMeasurer) {
        self.recalc_layout = false;
        self.recalculate_layout_impl(ui_config, text_measurer);
    }

    fn recalculate_layout_impl(&mut self, ui_config: &UiRuntimeConfig, text_measurer: &dyn TextMeasurer) {
        //info!("recalculating layout. viewport_width: {:?}, viewport_height: {:?}", self.viewport.width, self.viewport.height);

        let last_viewport_width: f32 = self.last_viewport.width as f32;
        let last_viewport_height: f32 = self.last_viewport.height as f32;

        let cache_mut = &mut self.cache;
        let visibility_store_ref = &self.visibility_store;

        // this calculates all the rects in cache_mut
        UiConfig::ROOT_NODE_ID.layout(cache_mut, ui_config, visibility_store_ref, text_measurer, last_viewport_width, last_viewport_height);
        finalize_rects(ui_config, self, &UiConfig::ROOT_NODE_ID, (0.0, 0.0, 0.0))

        // print_node(&Self::ROOT_PANEL_ID, &self.cache, &self.panels, true, false, "".to_string());
    }

    pub(crate) fn node_ref(&self, id: &NodeId) -> Option<&UiNodeState> {
        self.store.get_node(&id)
    }

    pub(crate) fn node_mut(&mut self, id: &NodeId) -> Option<&mut UiNodeState> {
        self.queue_recalculate_layout();
        self.store.get_node_mut(&id)
    }

    pub(crate) fn panel_mut(&mut self, id: &NodeId) -> Option<&mut PanelState> {
        let node_mut = self.node_mut(id)?;
        let panel_mut = node_mut.widget_panel_mut()?;
        Some(panel_mut)
    }

    pub(crate) fn text_mut(&mut self, id: &NodeId) -> Option<&mut TextState> {
        let node_mut = self.node_mut(id)?;
        let text_mut = node_mut.widget_text_mut()?;
        Some(text_mut)
    }

    pub(crate) fn button_mut(&mut self, id: &NodeId) -> Option<&mut ButtonState> {
        let node_mut = self.node_mut(id)?;
        let button_mut = node_mut.widget_button_mut()?;
        Some(button_mut)
    }

    pub fn textbox_mut(&mut self, id: &NodeId) -> Option<&mut TextboxState> {
        let node_mut = self.node_mut(id)?;
        let textbox_mut = node_mut.widget_textbox_mut()?;
        Some(textbox_mut)
    }
}

pub struct StateGlobals {
    box_mesh_handle_opt: Option<Handle<CpuMesh>>,
    text_color_handle_opt: Option<Handle<CpuMaterial>>,
}

impl StateGlobals {
    pub(crate) fn new() -> Self {
        Self {
            box_mesh_handle_opt: None,
            text_color_handle_opt: None,
        }
    }

    pub fn get_text_color_handle(&self) -> Option<&Handle<CpuMaterial>> {
        self.text_color_handle_opt.as_ref()
    }

    pub fn get_box_mesh_handle(&self) -> Option<&Handle<CpuMesh>> {
        self.box_mesh_handle_opt.as_ref()
    }
}

// recurses through tree and sets the bounds of each node to their absolute position
fn finalize_rects(
    ui_config: &UiRuntimeConfig,
    ui_state: &mut UiState,
    id: &NodeId,
    parent_position: (f32, f32, f32),
) {
    let Some(node) = ui_config.get_node(&id) else {
        warn!("no panel for id: {:?}", id);
        return;
    };

    let Some((width, height, child_offset_x, child_offset_y, _)) = ui_state.cache.bounds(id) else {
        warn!("no bounds for id 3: {:?}", id);
        return;
    };

    let child_position = (
        parent_position.0 + child_offset_x,
        parent_position.1 + child_offset_y,
        parent_position.2 + 0.1,
    );

    ui_state.cache.set_bounds(id, child_position.0, child_position.1, child_position.2, width, height);

    match node.widget_kind() {
        WidgetKind::Panel => {
            let Some(panel_ref) = ui_config.panel_ref(id) else {
                panic!("no panel ref for node_id: {:?}", id);
            };

            // update children
            let child_ids = panel_ref.children.clone();
            for child_id in child_ids {
                finalize_rects(ui_config, ui_state, &child_id, child_position);
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
                finalize_rects(ui_config, ui_state, &child_id, child_position);
            }
        }
        _ => {}
    }
}