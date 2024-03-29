
use bevy_log::warn;

use input::CursorIcon;
use render_api::{
    base::{CpuMaterial, CpuMesh},
    components::Viewport,
    shapes::UnitSquare,
};
use storage::{Handle, Storage};
use ui_layout::{Cache, Node, SizeUnits, TextMeasurer};
use instant::Instant;
use math::Vec2;

use crate::{widget::WidgetState, textbox::TextboxState, text::TextState, panel::PanelState, button::ButtonState, state_store::UiStateStore, input::UiInputEvent, events::UiGlobalEvent, cache::LayoutCache, node_id::NodeId, input::ui_receive_input, UiNodeEvent, button::NodeActiveState, UiConfig, UiNodeState, WidgetKind};

pub struct UiState {
    pub globals: StateGlobals,
    pub cache: LayoutCache,
    pub store: UiStateStore,

    recalc_layout: bool,
    last_viewport: Viewport,
    global_events: Vec<UiGlobalEvent>,
    node_events: Vec<(NodeId, UiNodeEvent)>,
    hovering_node: Option<NodeId>,
    selected_node: Option<NodeId>,
    cursor_icon: CursorIcon,
    interact_timer: Instant,
}

impl UiState {

    // confirmed below
    pub fn set_handles(
        &mut self,
        ui_config: &UiConfig,
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
            let mat_handle = materials.add(ui_config.globals.get_text_color());
            self.globals.text_color_handle_opt = Some(mat_handle);
        }

        // set color handles
        let ids = self.collect_color_handles();
        for id in ids {
            let node_ref = ui_config.node_ref(&id).unwrap();

            match node_ref.widget_kind() {
                WidgetKind::Panel => {
                    let panel_style_ref = ui_config.store.panel_style_ref(&id);
                    let color = panel_style_ref.background_color();
                    let panel_mut = self.panel_mut(&id).unwrap();
                    let mat_handle = materials.add(color);
                    panel_mut.background_color_handle = Some(mat_handle);
                }
                WidgetKind::Text => {
                    let text_style_ref = ui_config.store.text_style_ref(&id);
                    let color = text_style_ref.background_color();
                    let text_mut = self.text_mut(&id).unwrap();
                    let mat_handle = materials.add(color);
                    text_mut.background_color_handle = Some(mat_handle);
                }
                WidgetKind::Button => {
                    let button_style_ref = ui_config.store.button_style_ref(&id);

                    let background_color = button_style_ref.background_color();
                    let hover_color = button_style_ref.hover_color();
                    let down_color = button_style_ref.down_color();

                    let button_mut = self.button_mut(&id).unwrap();

                    let background_color_handle = materials.add(background_color);
                    button_mut.panel.background_color_handle = Some(background_color_handle);

                    let hover_color_handle = materials.add(hover_color);
                    button_mut.set_hover_color_handle(hover_color_handle);

                    let down_color_handle = materials.add(down_color);
                    button_mut.set_down_color_handle(down_color_handle);
                }
                WidgetKind::Textbox => {
                    let textbox_style_ref = ui_config.store.textbox_style_ref(&id);

                    let background_color = textbox_style_ref.background_color();
                    let hover_color = textbox_style_ref.hover_color();
                    let active_color = textbox_style_ref.active_color();
                    let select_color = textbox_style_ref.selection_color();

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

    // confirmed above
    // not confirmed below

    pub fn new(ui_config: &UiConfig) -> Self {
        let mut me = Self {
            globals: StateGlobals::new(),
            cache: LayoutCache::new(),
            store: UiStateStore::new(),

            recalc_layout: false,
            last_viewport: Viewport::new_at_origin(0, 0),
            global_events: Vec::new(),
            node_events: Vec::new(),
            hovering_node: None,
            selected_node: None,
            cursor_icon: CursorIcon::Default,
            interact_timer: Instant::now(),
        };

        for node in ui_config.store.nodes.iter() {
            me.store.node_state_init(node);
        }

        me
    }

    // events
    pub fn receive_input(&mut self, ui_config: &UiConfig, text_measurer: &dyn TextMeasurer, mouse_position: Option<Vec2>, events: Vec<UiInputEvent>) {
        ui_receive_input(ui_config, self, text_measurer, mouse_position, events);
    }

    pub fn get_cursor_icon(&self) -> CursorIcon {
        self.cursor_icon
    }

    pub fn set_cursor_icon(&mut self, cursor_icon: CursorIcon) {
        self.cursor_icon = cursor_icon;
    }

    pub fn get_hover(&self) -> Option<NodeId> {
        self.hovering_node
    }

    pub fn receive_hover(&mut self, id: &NodeId) {
        self.hovering_node = Some(*id);
    }

    pub fn get_active_state(&self, id: &NodeId) -> NodeActiveState {
        if let Some(select_id) = self.selected_node {
            if select_id == *id {
                return NodeActiveState::Active;
            }
        }

        if let Some(hover_id) = self.hovering_node {
            if hover_id == *id {
                return NodeActiveState::Hover;
            }
        };

        return NodeActiveState::Normal;
    }

    pub fn clear_hover(&mut self) {
        self.hovering_node = None;
    }

    pub fn get_active_node(&self) -> Option<NodeId> {
        self.selected_node
    }

    pub fn set_active_node(&mut self, id_opt: Option<NodeId>) {
        self.selected_node = id_opt;
    }

    pub fn emit_global_event(&mut self, event: UiGlobalEvent) {
        self.global_events.push(event);
    }

    pub fn take_global_events(&mut self) -> Vec<UiGlobalEvent> {
        std::mem::take(&mut self.global_events)
    }

    pub fn emit_node_event(&mut self, node_id: &NodeId, event: UiNodeEvent) {
        self.node_events.push((*node_id, event));
    }

    pub fn take_node_events(&mut self) -> Vec<(NodeId, UiNodeEvent)> {
        std::mem::take(&mut self.node_events)
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

    pub fn recalculate_layout(&mut self, ui_config: &mut UiConfig, text_measurer: &dyn TextMeasurer) {
        self.recalc_layout = false;
        self.recalculate_layout_impl(ui_config, text_measurer);
    }

    fn recalculate_layout_impl(&mut self, ui_config: &mut UiConfig, text_measurer: &dyn TextMeasurer) {
        //info!("recalculating layout. viewport_width: {:?}, viewport_height: {:?}", self.viewport.width, self.viewport.height);

        let last_viewport_width: f32 = self.last_viewport.width as f32;
        let last_viewport_height: f32 = self.last_viewport.height as f32;

        let root_panel_style_id = *ui_config
            .node_ref(&UiConfig::ROOT_NODE_ID)
            .unwrap()
            .style_ids
            .last()
            .unwrap();
        let root_panel_style = ui_config.style_mut(&root_panel_style_id).unwrap();
        root_panel_style.width = Some(SizeUnits::Pixels(last_viewport_width));
        root_panel_style.height = Some(SizeUnits::Pixels(last_viewport_height));

        let cache_mut = &mut self.cache;
        let store_ref = &ui_config.store;
        let state_store_ref = &self.store;

        // this calculates all the rects in cache_mut
        UiConfig::ROOT_NODE_ID.layout(cache_mut, store_ref, state_store_ref, text_measurer);
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

    pub(crate) fn textbox_mut(&mut self, id: &NodeId) -> Option<&mut TextboxState> {
        let node_mut = self.node_mut(id)?;
        let textbox_mut = node_mut.widget_textbox_mut()?;
        Some(textbox_mut)
    }

    pub(crate) fn reset_interact_timer(&mut self) {
        self.interact_timer = Instant::now();
    }

    pub fn interact_timer_was_recent(&self) -> bool {
        self.interact_timer.elapsed().as_secs_f32() < 1.0
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
    ui_config: &UiConfig,
    ui_state: &mut UiState,
    id: &NodeId,
    parent_position: (f32, f32, f32),
) {
    let Some(node) = ui_config.store.get_node(&id) else {
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
            let Some(panel_ref) = ui_config.store.panel_ref(id) else {
                panic!("no panel ref for node_id: {:?}", id);
            };

            // update children
            let child_ids = panel_ref.children.clone();
            for child_id in child_ids {
                finalize_rects(ui_config, ui_state, &child_id, child_position);
            }
        }
        WidgetKind::Button => {
            let Some(button_ref) = ui_config.store.button_ref(id) else {
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