use std::collections::HashMap;
use bevy_log::warn;

use asset_id::AssetId;
use input::CursorIcon;
use render_api::{
    base::{Color, CpuMaterial, CpuMesh},
    components::Viewport,
    shapes::UnitSquare,
};
use storage::{Handle, Storage};
use ui_layout::{Cache, Node, SizeUnits, TextMeasurer};
use instant::Instant;
use math::Vec2;

use crate::{cache::LayoutCache, node::UiNode, node_id::NodeId, panel::{Panel, PanelMut, PanelStyle, PanelStyleMut}, store::UiStore, style::{NodeStyle, StyleId, WidgetStyle}, text::{TextStyle, TextStyleMut}, widget::{Widget, WidgetKind}, input::ui_receive_input, Button, ButtonStyle, ButtonStyleMut, UiNodeEvent, button::NodeActiveState, Text, Textbox, TextboxStyleMut, TextboxStyle, Navigation};
use crate::events::UiGlobalEvent;
use crate::input::UiInputEvent;

pub struct Ui {
    pub globals: Globals,
    pub cache: LayoutCache,
    pub store: UiStore,

    recalc_layout: bool,
    last_viewport: Viewport,
    id_str_to_node_id_map: HashMap<String, NodeId>,
    global_events: Vec<UiGlobalEvent>,
    node_events: Vec<(NodeId, UiNodeEvent)>,
    hovering_node: Option<NodeId>,
    selected_node: Option<NodeId>,
    cursor_icon: Option<CursorIcon>,
    interact_timer: Instant,
}

impl Ui {
    pub const ROOT_NODE_ID: NodeId = NodeId::new(0);
    // pub(crate) const ROOT_STYLE_ID: StyleId = StyleId::new(0);
    pub const BASE_TEXT_STYLE_ID: StyleId = StyleId::new(0);

    pub fn new() -> Self {
        let mut me = Self {
            globals: Globals::new(),
            cache: LayoutCache::new(),
            store: UiStore::new(),

            recalc_layout: false,
            last_viewport: Viewport::new_at_origin(0, 0),
            id_str_to_node_id_map: HashMap::new(),
            global_events: Vec::new(),
            node_events: Vec::new(),
            hovering_node: None,
            selected_node: None,
            cursor_icon: None,
            interact_timer: Instant::now(),
        };

        // Root Node
        let root_panel_id = me.create_node(Widget::Panel(Panel::new()));
        if root_panel_id != Self::ROOT_NODE_ID {
            panic!("root panel id is not 0");
        }

        // Base Text Style
        let base_text_style_id =
            me.create_style(NodeStyle::empty(WidgetStyle::Text(TextStyle::empty())));
        if base_text_style_id != Self::BASE_TEXT_STYLE_ID {
            panic!("base text style id is {:?}, not 1!", base_text_style_id);
        }
        me.style_mut(&base_text_style_id).unwrap().width = Some(SizeUnits::Percentage(100.0));
        me.style_mut(&base_text_style_id).unwrap().height = Some(SizeUnits::Percentage(100.0));

        me
    }

    // events
    pub fn receive_input(&mut self, text_measurer: &dyn TextMeasurer, mouse_position: Option<Vec2>, events: Vec<UiInputEvent>) {
        ui_receive_input(self, text_measurer, mouse_position, events);
    }

    pub fn take_cursor_icon(&mut self) -> Option<CursorIcon> {
        let output = self.cursor_icon;
        self.cursor_icon = None;
        output
    }

    pub fn set_cursor_icon(&mut self, cursor_icon: CursorIcon) {
        self.cursor_icon = Some(cursor_icon);
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

    pub fn get_first_input(&self) -> Option<NodeId> {
        self.globals.first_input
    }

    pub fn set_first_input(&mut self, id: NodeId) {
        self.globals.first_input = Some(id);
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

    // system methods

    pub fn set_handles(
        &mut self,
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
            let mat_handle = materials.add(self.globals.text_color);
            self.globals.text_color_handle_opt = Some(mat_handle);
        }

        // set color handles
        let ids = self.collect_color_handles();
        for id in ids {
            let node_ref = self.node_ref(&id).unwrap();

            match node_ref.widget_kind() {
                WidgetKind::Panel => {
                    let panel_style_ref = self.store.panel_style_ref(&id);
                    let color = panel_style_ref.background_color();
                    let panel_mut = self.panel_mut(&id).unwrap();
                    let mat_handle = materials.add(color);
                    panel_mut.background_color_handle = Some(mat_handle);
                }
                WidgetKind::Text => {
                    let text_style_ref = self.store.text_style_ref(&id);
                    let color = text_style_ref.background_color();
                    let text_mut = self.text_mut(&id).unwrap();
                    let mat_handle = materials.add(color);
                    text_mut.background_color_handle = Some(mat_handle);
                }
                WidgetKind::Button => {
                    let button_style_ref = self.store.button_style_ref(&id);

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
                    let textbox_style_ref = self.store.textbox_style_ref(&id);

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

    // interface

    pub fn get_text_icon_asset_id(&self) -> &AssetId {
        self.globals.text_icon_asset_id_opt.as_ref().unwrap()
    }

    pub fn set_text_icon_asset_id(&mut self, text_icon_asset_id: &AssetId) -> &mut Self {
        self.globals.text_icon_asset_id_opt = Some(text_icon_asset_id.clone());
        self
    }

    pub fn get_text_color(&self) -> Color {
        self.globals.text_color
    }

    pub fn set_text_color(&mut self, text_color: Color) -> &mut Self {
        self.globals.set_text_color(text_color);
        self
    }

    pub fn root_mut(&mut self) -> PanelMut {
        self.queue_recalculate_layout();
        PanelMut::new(self, Self::ROOT_NODE_ID)
    }

    pub fn create_panel_style(&mut self, inner_fn: impl FnOnce(&mut PanelStyleMut)) -> StyleId {
        let new_style_id =
            self.create_style(NodeStyle::empty(WidgetStyle::Panel(PanelStyle::empty())));
        let mut panel_style_mut = PanelStyleMut::new(self, new_style_id);
        inner_fn(&mut panel_style_mut);
        new_style_id
    }

    pub fn create_text_style(&mut self, inner_fn: impl FnOnce(&mut TextStyleMut)) -> StyleId {
        let new_style_id =
            self.create_style(NodeStyle::empty(WidgetStyle::Text(TextStyle::empty())));
        let mut text_style_mut = TextStyleMut::new(self, new_style_id);
        inner_fn(&mut text_style_mut);
        new_style_id
    }

    pub fn create_button_style(&mut self, inner_fn: impl FnOnce(&mut ButtonStyleMut)) -> StyleId {
        let new_style_id =
            self.create_style(NodeStyle::empty(WidgetStyle::Button(ButtonStyle::empty())));
        let mut button_style_mut = ButtonStyleMut::new(self, new_style_id);
        inner_fn(&mut button_style_mut);
        new_style_id
    }

    pub fn create_textbox_style(&mut self, inner_fn: impl FnOnce(&mut TextboxStyleMut)) -> StyleId {
        let new_style_id =
            self.create_style(NodeStyle::empty(WidgetStyle::Textbox(TextboxStyle::empty())));
        let mut textbox_style_mut = TextboxStyleMut::new(self, new_style_id);
        inner_fn(&mut textbox_style_mut);
        new_style_id
    }

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

    pub fn recalculate_layout(&mut self, text_measurer: &dyn TextMeasurer) {
        self.recalc_layout = false;
        self.recalculate_layout_impl(text_measurer);
    }

    fn recalculate_layout_impl(&mut self, text_measurer: &dyn TextMeasurer) {
        //info!("recalculating layout. viewport_width: {:?}, viewport_height: {:?}", self.viewport.width, self.viewport.height);

        let last_viewport_width: f32 = self.last_viewport.width as f32;
        let last_viewport_height: f32 = self.last_viewport.height as f32;

        let root_panel_style_id = *self
            .node_ref(&Self::ROOT_NODE_ID)
            .unwrap()
            .style_ids
            .last()
            .unwrap();
        let root_panel_style = self.style_mut(&root_panel_style_id).unwrap();
        root_panel_style.width = Some(SizeUnits::Pixels(last_viewport_width));
        root_panel_style.height = Some(SizeUnits::Pixels(last_viewport_height));

        let panels_ref = &self.store;
        let cache_mut = &mut self.cache;

        // this calculates all the rects in cache_mut
        Self::ROOT_NODE_ID.layout(cache_mut, panels_ref, panels_ref, text_measurer);
        finalize_rects(self, &Self::ROOT_NODE_ID, (0.0, 0.0, 0.0))

        // print_node(&Self::ROOT_PANEL_ID, &self.cache, &self.panels, true, false, "".to_string());
    }

    pub fn collect_color_handles(&mut self) -> Vec<NodeId> {
        let mut pending_mat_handles = Vec::new();
        for id in self.store.node_ids() {
            let Some(node_ref) = self.node_ref(&id) else {
                continue;
            };
            match node_ref.widget_kind() {
                WidgetKind::Panel => {
                    let panel_ref = node_ref.widget_panel_ref().unwrap();
                    if panel_ref.background_color_handle.is_none() {
                        pending_mat_handles.push(id);
                    }
                }
                WidgetKind::Text => {
                    let text_ref = node_ref.widget_text_ref().unwrap();
                    if text_ref.background_color_handle.is_none() {
                        pending_mat_handles.push(id);
                    }
                }
                WidgetKind::Button => {
                    let button_ref = node_ref.widget_button_ref().unwrap();
                    if button_ref.needs_color_handle() {
                        pending_mat_handles.push(id);
                    }
                }
                WidgetKind::Textbox => {
                    let textbox_ref = node_ref.widget_textbox_ref().unwrap();
                    if textbox_ref.needs_color_handle() {
                        pending_mat_handles.push(id);
                    }
                }
            }
        }
        pending_mat_handles
    }

    pub fn get_node_id_by_id_str(&self, id_str: &str) -> Option<NodeId> {
        self.id_str_to_node_id_map.get(id_str).cloned()
    }

    pub(crate) fn create_node(&mut self, widget: Widget) -> NodeId {
        let mut id_str_opt = None;
        match &widget {
            Widget::Button(button) => {
                id_str_opt = Some(button.id_str.clone());
            }
            Widget::Textbox(textbox) => {
                id_str_opt = Some(textbox.id_str.clone());
            }
            _ => {}
        }

        let ui_node = UiNode::new(widget);
        let node_id = self.store.insert_node(ui_node);

        if let Some(id_str) = id_str_opt {
            self.id_str_to_node_id_map.insert(id_str, node_id);
        }

        node_id
    }

    pub(crate) fn node_ref(&self, id: &NodeId) -> Option<&UiNode> {
        self.store.get_node(&id)
    }

    pub(crate) fn node_mut(&mut self, id: &NodeId) -> Option<&mut UiNode> {
        self.queue_recalculate_layout();
        self.store.get_node_mut(&id)
    }

    pub(crate) fn panel_mut(&mut self, id: &NodeId) -> Option<&mut Panel> {
        let node_mut = self.node_mut(id)?;
        let panel_mut = node_mut.widget_panel_mut()?;
        Some(panel_mut)
    }

    pub(crate) fn text_mut(&mut self, id: &NodeId) -> Option<&mut Text> {
        let node_mut = self.node_mut(id)?;
        let text_mut = node_mut.widget_text_mut()?;
        Some(text_mut)
    }

    pub(crate) fn button_mut(&mut self, id: &NodeId) -> Option<&mut Button> {
        let node_mut = self.node_mut(id)?;
        let button_mut = node_mut.widget_button_mut()?;
        Some(button_mut)
    }

    pub(crate) fn textbox_mut(&mut self, id: &NodeId) -> Option<&mut Textbox> {
        let node_mut = self.node_mut(id)?;
        let textbox_mut = node_mut.widget_textbox_mut()?;
        Some(textbox_mut)
    }

    pub(crate) fn style_mut(&mut self, id: &StyleId) -> Option<&mut NodeStyle> {
        self.store.get_style_mut(&id)
    }

    pub(crate) fn create_style(&mut self, style: NodeStyle) -> StyleId {
        self.store.insert_style(style)
    }

    pub(crate) fn reset_interact_timer(&mut self) {
        self.interact_timer = Instant::now();
    }

    pub fn interact_timer_was_recent(&self) -> bool {
        self.interact_timer.elapsed().as_secs_f32() < 1.0
    }

    // navigation
    pub(crate) fn nav_get_up_id(&self, id: &NodeId) -> Option<NodeId> {
        let nav = self.get_node_nav(id)?;
        let up_str: &str = nav.up_goes_to.as_ref()?;
        self.get_node_id_by_id_str(up_str)
    }

    pub(crate) fn nav_get_down_id(&self, id: &NodeId) -> Option<NodeId> {
        let nav = self.get_node_nav(id)?;
        let down_str: &str = nav.down_goes_to.as_ref()?;
        self.get_node_id_by_id_str(down_str)
    }

    pub(crate) fn nav_get_left_id(&self, id: &NodeId) -> Option<NodeId> {
        let nav = self.get_node_nav(id)?;
        let left_str: &str = nav.left_goes_to.as_ref()?;
        self.get_node_id_by_id_str(left_str)
    }

    pub(crate) fn nav_get_right_id(&self, id: &NodeId) -> Option<NodeId> {
        let nav = self.get_node_nav(id)?;
        let right_str: &str = nav.right_goes_to.as_ref()?;
        self.get_node_id_by_id_str(right_str)
    }

    fn get_node_nav(&self, id: &NodeId) -> Option<&Navigation> {
        let node = self.node_ref(id)?;
        match node.widget_kind() {
            WidgetKind::Button => Some(&node.widget_button_ref()?.navigation),
            WidgetKind::Textbox => Some(&node.widget_textbox_ref()?.navigation),
            _ => None,
        }
    }
}

pub struct Globals {
    box_mesh_handle_opt: Option<Handle<CpuMesh>>,
    text_icon_asset_id_opt: Option<AssetId>,
    text_color: Color,
    text_color_handle_opt: Option<Handle<CpuMaterial>>,
    first_input: Option<NodeId>,
}

impl Globals {
    pub(crate) fn new() -> Self {
        Self {
            box_mesh_handle_opt: None,
            text_icon_asset_id_opt: None,
            text_color: Color::BLACK,
            text_color_handle_opt: None,
            first_input: None,
        }
    }

    pub fn get_text_icon_handle(&self) -> Option<&AssetId> {
        self.text_icon_asset_id_opt.as_ref()
    }

    pub fn get_text_color_handle(&self) -> Option<&Handle<CpuMaterial>> {
        self.text_color_handle_opt.as_ref()
    }

    pub fn get_box_mesh_handle(&self) -> Option<&Handle<CpuMesh>> {
        self.box_mesh_handle_opt.as_ref()
    }

    pub fn set_text_color(&mut self, color: Color) {
        if color == self.text_color {
            return;
        }
        self.text_color = color;
        self.text_color_handle_opt = None;
    }
}

// recurses through tree and sets the bounds of each node to their absolute position
fn finalize_rects(
    ui: &mut Ui,
    id: &NodeId,
    parent_position: (f32, f32, f32),
) {
    let Some(node) = ui.store.get_node(&id) else {
        warn!("no panel for id: {:?}", id);
        return;
    };

    let Some((width, height, child_offset_x, child_offset_y, _)) = ui.cache.bounds(id) else {
        warn!("no bounds for id: {:?}", id);
        return;
    };

    let child_position = (
        parent_position.0 + child_offset_x,
        parent_position.1 + child_offset_y,
        parent_position.2 + 0.1,
    );

    ui.cache.set_bounds(id, child_position.0, child_position.1, child_position.2, width, height);

    match node.widget_kind() {
        WidgetKind::Panel => {
            let Some(panel_ref) = ui.store.panel_ref(id) else {
                panic!("no panel ref for node_id: {:?}", id);
            };

            // update children
            let child_ids = panel_ref.children.clone();
            for child_id in child_ids {
                finalize_rects(ui, &child_id, child_position);
            }
        }
        WidgetKind::Button => {
            let Some(button_ref) = ui.store.button_ref(id) else {
                panic!("no button ref for node_id: {:?}", id);
            };
            let panel_ref = &button_ref.panel;

            // update children
            let child_ids = panel_ref.children.clone();
            for child_id in child_ids {
                finalize_rects(ui, &child_id, child_position);
            }
        }
        _ => {}
    }
}