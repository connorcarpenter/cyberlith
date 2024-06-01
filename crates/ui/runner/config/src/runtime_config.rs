use std::{collections::HashMap, slice::Iter};

use asset_id::AssetId;
use render_api::base::Color;
use ui_builder_config::{BaseNodeStyle, Button, ButtonStyle, Navigation, Panel, PanelStyle, SpinnerStyle, StyleId, TextStyle, Textbox, TextboxStyle, UiConfig, UiNode, WidgetKind, WidgetStyle};
use ui_layout::{
    Alignment, LayoutType, MarginUnits, NodeId, NodeStore, PositionType, SizeUnits, Solid,
    TextMeasurer,
};
use ui_serde::SerdeErr;

use crate::{styles::compute_styles, text_measure_raw_size};

pub struct UiRuntimeConfig {
    styles: Vec<BaseNodeStyle>,
    nodes: Vec<UiNode>,

    first_input_opt: Option<NodeId>,
    text_icon_asset_id: AssetId,
    eye_icon_asset_id: AssetId,
    id_str_to_node_id_map: HashMap<String, NodeId>,
}

impl UiRuntimeConfig {
    pub const ROOT_NODE_ID: NodeId = NodeId::new(0);
    pub const Z_STEP_RENDER: f32 = 3.0;

    pub fn load_from_bytes(bytes: &[u8]) -> Result<Self, SerdeErr> {
        let config = ui_serde::bits::read_ui_bits(bytes)?;
        Ok(Self::load_from_builder_config(config))
    }

    pub fn load_from_builder_config(ui_config: UiConfig) -> Self {
        let (styles, nodes, first_input_opt, text_icon_asset_id, eye_icon_asset_id, node_map) =
            ui_config.decompose();

        let styles = compute_styles(styles);

        Self {
            styles,
            nodes,
            first_input_opt,
            id_str_to_node_id_map: node_map,
            text_icon_asset_id,
            eye_icon_asset_id,
        }
    }

    pub fn get_first_input(&self) -> Option<NodeId> {
        self.first_input_opt
    }

    pub fn get_text_icon_asset_id(&self) -> AssetId {
        self.text_icon_asset_id
    }

    pub fn get_eye_icon_asset_id(&self) -> AssetId {
        self.eye_icon_asset_id
    }

    pub fn get_node_id_by_id_str(&self, id_str: &str) -> Option<NodeId> {
        self.id_str_to_node_id_map.get(id_str).cloned()
    }

    // nodes

    pub fn get_node(&self, id: &NodeId) -> Option<&UiNode> {
        self.nodes.get(id.as_usize())
    }

    pub fn get_node_mut(&mut self, id: &NodeId) -> Option<&mut UiNode> {
        self.nodes.get_mut(id.as_usize())
    }

    pub(crate) fn node_kind(&self, id: &NodeId) -> WidgetKind {
        self.get_node(id).unwrap().widget_kind()
    }

    pub fn nodes_len(&self) -> usize {
        self.nodes.len()
    }

    pub fn nodes_iter(&self) -> Iter<'_, UiNode> {
        self.nodes.iter()
    }

    pub fn add_node(&mut self, node: UiNode) -> NodeId {
        let id = NodeId::from_usize(self.nodes.len());
        self.nodes.push(node);
        id
    }

    pub fn remove_nodes_after(&mut self, node: &NodeId) {
        self.nodes.truncate(node.as_usize() + 1);
    }

    pub fn panel_ref(&self, id: &NodeId) -> Option<&Panel> {
        let node = self.get_node(id)?;
        if node.widget_kind() == WidgetKind::Panel {
            return node.widget_panel_ref();
        }
        None
    }

    pub fn panel_mut(&mut self, id: &NodeId) -> Option<&mut Panel> {
        let node = self.get_node_mut(id)?;
        if node.widget_kind() == WidgetKind::Panel {
            return node.widget_panel_mut();
        }
        None
    }

    pub fn button_ref(&self, id: &NodeId) -> Option<&Button> {
        let node = self.get_node(id)?;
        if node.widget_kind() == WidgetKind::Button {
            return node.widget_button_ref();
        }
        None
    }

    pub fn textbox_ref(&self, id: &NodeId) -> Option<&Textbox> {
        let node = self.get_node(id)?;
        if node.widget_kind() == WidgetKind::Textbox {
            return node.widget_textbox_ref();
        }
        None
    }

    // styles

    pub fn styles_iter(&self) -> Iter<'_, BaseNodeStyle> {
        self.styles.iter()
    }

    fn get_style(&self, style_id: &StyleId) -> Option<&BaseNodeStyle> {
        self.styles.get(style_id.as_usize())
    }

    pub fn add_style(&mut self, style: BaseNodeStyle) -> StyleId {
        let id = StyleId::new(self.styles.len() as u32);
        self.styles.push(style);
        id
    }

    fn node_style(&self, id: &NodeId) -> Option<&BaseNodeStyle> {
        let node = self.get_node(id)?;
        node.style_id()
            .map(|style_id| self.get_style(&style_id))
            .flatten()
    }

    fn widget_style(&self, id: &NodeId) -> Option<&WidgetStyle> {
        let style = self.node_style(id)?;
        Some(&style.widget_style)
    }

    fn panel_style(&self, id: &NodeId) -> Option<&PanelStyle> {
        let widget_style = self.widget_style(id)?;
        match widget_style {
            WidgetStyle::Panel(panel_style) => Some(panel_style),
            WidgetStyle::Button(button_style) => Some(&button_style.panel),
            _ => None,
        }
    }

    fn text_style(&self, id: &NodeId) -> Option<&TextStyle> {
        let widget_style = self.widget_style(id)?;
        match widget_style {
            WidgetStyle::Text(text_style) => Some(text_style),
            _ => None,
        }
    }

    pub fn button_style(&self, id: &NodeId) -> Option<&ButtonStyle> {
        let widget_style = self.widget_style(id)?;
        match widget_style {
            WidgetStyle::Button(button_style) => Some(button_style),
            _ => None,
        }
    }

    pub fn textbox_style(&self, id: &NodeId) -> Option<&TextboxStyle> {
        let widget_style = self.widget_style(id)?;
        match widget_style {
            WidgetStyle::Textbox(textbox_style) => Some(textbox_style),
            _ => None,
        }
    }

    pub fn spinner_style(&self, id: &NodeId) -> Option<&SpinnerStyle> {
        let widget_style = self.widget_style(id)?;
        match widget_style {
            WidgetStyle::Spinner(spinner_style) => Some(spinner_style),
            _ => None,
        }
    }

    pub fn node_background_color(&self, id: &NodeId) -> Option<&Color> {
        match self.widget_style(id)? {
            WidgetStyle::Text(text_style) => text_style.background_color.as_ref(),
            WidgetStyle::Button(button_style) => button_style.panel.background_color.as_ref(),
            WidgetStyle::Textbox(textbox_style) => textbox_style.background_color.as_ref(),
            WidgetStyle::Panel(panel_style) => panel_style.background_color.as_ref(),
            WidgetStyle::Spinner(spinner_style) => spinner_style.background_color.as_ref(),
            WidgetStyle::UiContainer => None,
        }
    }

    pub fn node_text_color(&self, id: &NodeId) -> Option<&Color> {
        match self.widget_style(id)? {
            WidgetStyle::Text(text_style) => text_style.text_color.as_ref(),
            WidgetStyle::Textbox(textbox_style) => textbox_style.text_color.as_ref(),
            _ => None,
        }
    }

    pub fn node_spinner_color(&self, id: &NodeId) -> Option<&Color> {
        match self.widget_style(id)? {
            WidgetStyle::Spinner(spinner_style) => spinner_style.spinner_color.as_ref(),
            _ => None,
        }
    }

    pub fn node_background_alpha(&self, id: &NodeId) -> f32 {
        match self.get_node(id).unwrap().widget_kind() {
            WidgetKind::Panel => {
                let mut output = 1.0;
                if let Some(panel_style) = self.panel_style(id) {
                    if let Some(alpha) = panel_style.background_alpha() {
                        output = alpha;
                    }
                }
                output
            }
            WidgetKind::Text => {
                let mut output = 0.0;
                if let Some(text_style) = self.text_style(id) {
                    if let Some(alpha) = text_style.background_alpha {
                        output = alpha;
                    }
                }
                output
            }
            WidgetKind::Button => {
                let mut output = 1.0;
                if let Some(panel_style) = self.panel_style(id) {
                    if let Some(alpha) = panel_style.background_alpha() {
                        output = alpha;
                    }
                }
                output
            }
            WidgetKind::Textbox => {
                let mut output = 1.0;
                if let Some(textbox_style) = self.textbox_style(id) {
                    if let Some(alpha) = textbox_style.background_alpha {
                        output = alpha;
                    }
                }
                output
            }
            WidgetKind::Spinner => {
                let mut output = 1.0;
                if let Some(spinner_style) = self.spinner_style(id) {
                    if let Some(alpha) = spinner_style.background_alpha {
                        output = alpha;
                    }
                }
                output
            }
            WidgetKind::UiContainer => 1.0,
        }
    }

    // navigation
    pub fn nav_get_up_id(&self, id: &NodeId) -> Option<NodeId> {
        let nav = self.get_node_nav(id)?;
        let up_str: &str = nav.up_goes_to.as_ref()?;
        self.get_node_id_by_id_str(up_str)
    }

    pub fn nav_get_down_id(&self, id: &NodeId) -> Option<NodeId> {
        let nav = self.get_node_nav(id)?;
        let down_str: &str = nav.down_goes_to.as_ref()?;
        self.get_node_id_by_id_str(down_str)
    }

    pub fn nav_get_left_id(&self, id: &NodeId) -> Option<NodeId> {
        let nav = self.get_node_nav(id)?;
        let left_str: &str = nav.left_goes_to.as_ref()?;
        self.get_node_id_by_id_str(left_str)
    }

    pub fn nav_get_right_id(&self, id: &NodeId) -> Option<NodeId> {
        let nav = self.get_node_nav(id)?;
        let right_str: &str = nav.right_goes_to.as_ref()?;
        self.get_node_id_by_id_str(right_str)
    }

    pub fn nav_get_tab_id(&self, id: &NodeId) -> Option<NodeId> {
        let nav = self.get_node_nav(id)?;
        let tab_str: &str = nav.tab_goes_to.as_ref()?;
        self.get_node_id_by_id_str(tab_str)
    }

    fn get_node_nav(&self, id: &NodeId) -> Option<&Navigation> {
        let node = self.get_node(id)?;
        match node.widget_kind() {
            WidgetKind::Button => Some(&node.widget_button_ref()?.navigation),
            WidgetKind::Textbox => Some(&node.widget_textbox_ref()?.navigation),
            _ => None,
        }
    }
}

impl NodeStore for UiRuntimeConfig {
    fn node_children(&self, id: &NodeId) -> Iter<NodeId> {
        if !self.node_kind(id).has_children() {
            return [].iter();
        }
        let node_ref = self.get_node(id).unwrap();
        let widget_kind = node_ref.widget_kind();
        match widget_kind {
            WidgetKind::Panel => {
                let panel_ref = self.panel_ref(id).unwrap();
                return panel_ref.children.iter();
            }
            WidgetKind::Button => {
                let button_ref = self.button_ref(id).unwrap();
                return button_ref.panel.children.iter();
            }
            _ => panic!("impossible"),
        }
    }

    // all of these unwrap_or_default
    fn node_layout_type(&self, id: &NodeId) -> LayoutType {
        let mut output = LayoutType::default();

        if self.node_kind(id).has_children() {
            if let Some(panel_style) = self.panel_style(id) {
                if let Some(layout_type) = panel_style.layout_type {
                    output = layout_type;
                }
            }
        }

        output
    }

    // all of these unwrap_or_default
    fn node_position_type(&self, id: &NodeId) -> PositionType {
        let mut output = PositionType::default();

        if let Some(node_style) = self.node_style(id) {
            if let Some(layout_type) = node_style.position_type {
                output = layout_type;
            }
        }

        output
    }

    // all of these unwrap_or(SizeUnits::Percentage(100.0))
    fn node_width(&self, id: &NodeId) -> SizeUnits {
        let mut output = SizeUnits::default();

        if self.node_kind(id).is_text() {
            return output;
        }

        if let Some(node_style) = self.node_style(id) {
            if let Some(width) = node_style.width {
                output = width;
            }
        }

        output
    }

    // all of these unwrap_or(SizeUnits::Percentage(100.0))
    fn node_height(&self, id: &NodeId) -> SizeUnits {
        let mut output = SizeUnits::Auto;

        if self.node_kind(id).is_text() {
            output = SizeUnits::Percentage(100.0);
        }

        if let Some(node_style) = self.node_style(id) {
            if let Some(height) = node_style.height {
                output = height;
            }
        }

        output
    }

    // all of these unwrap_or(SizeUnits::Pixels(0.0))
    fn node_width_min(&self, id: &NodeId) -> SizeUnits {
        let mut output = SizeUnits::default();

        if self.node_kind(id).is_text() {
            return output;
        }

        if let Some(node_style) = self.node_style(id) {
            if let Some(width_min) = node_style.width_min {
                output = width_min;
            }
        }

        output
    }

    // all of these unwrap_or(SizeUnits::Pixels(0.0))
    fn node_height_min(&self, id: &NodeId) -> SizeUnits {
        let mut output = SizeUnits::default();

        if self.node_kind(id).is_text() {
            return output;
        }

        if let Some(node_style) = self.node_style(id) {
            if let Some(height_min) = node_style.height_min {
                output = height_min;
            }
        }

        output
    }

    // all of these unwrap_or(SizeUnits::Pixels(f32::MAX))
    fn node_width_max(&self, id: &NodeId) -> SizeUnits {
        let mut output = SizeUnits::default();

        if self.node_kind(id).is_text() {
            return output;
        }

        if let Some(node_style) = self.node_style(id) {
            if let Some(width_max) = node_style.width_max {
                output = width_max;
            }
        }

        output
    }

    // all of these unwrap_or(SizeUnits::Pixels(f32::MAX))
    fn node_height_max(&self, id: &NodeId) -> SizeUnits {
        let mut output = SizeUnits::default();

        if self.node_kind(id).is_text() {
            return output;
        }

        if let Some(node_style) = self.node_style(id) {
            if let Some(height_max) = node_style.height_max {
                output = height_max;
            }
        }

        output
    }

    // all of these unwrap_or_default
    fn node_margin_left(&self, id: &NodeId) -> MarginUnits {
        let mut output = MarginUnits::default();

        if let Some(node_style) = self.node_style(id) {
            if let Some(margin_left) = node_style.margin_left {
                output = margin_left;
            }
        }

        output
    }

    // all of these unwrap_or_default
    fn node_margin_right(&self, id: &NodeId) -> MarginUnits {
        let mut output = MarginUnits::default();

        if let Some(node_style) = self.node_style(id) {
            if let Some(margin_right) = node_style.margin_right {
                output = margin_right;
            }
        }

        output
    }

    // all of these unwrap_or_default
    fn node_margin_top(&self, id: &NodeId) -> MarginUnits {
        let mut output = MarginUnits::default();

        if let Some(node_style) = self.node_style(id) {
            if let Some(margin_top) = node_style.margin_top {
                output = margin_top;
            }
        }

        output
    }

    // all of these unwrap_or_default
    fn node_margin_bottom(&self, id: &NodeId) -> MarginUnits {
        let mut output = MarginUnits::default();

        if let Some(node_style) = self.node_style(id) {
            if let Some(margin_bottom) = node_style.margin_bottom {
                output = margin_bottom;
            }
        }

        output
    }

    // all of these unwrap_or_default
    fn node_padding_left(&self, id: &NodeId) -> MarginUnits {
        let mut output = MarginUnits::default();

        if !self.node_kind(id).has_children() {
            return output;
        }

        if let Some(panel_style) = self.panel_style(id) {
            if let Some(padding_left) = panel_style.padding_left {
                output = padding_left;
            }
        }

        output
    }

    // all of these unwrap_or_default
    fn node_padding_right(&self, id: &NodeId) -> MarginUnits {
        let mut output = MarginUnits::default();

        if !self.node_kind(id).has_children() {
            return output;
        }

        if let Some(panel_style) = self.panel_style(id) {
            if let Some(padding_right) = panel_style.padding_right {
                output = padding_right;
            }
        }

        output
    }

    // all of these unwrap_or_default
    fn node_padding_top(&self, id: &NodeId) -> MarginUnits {
        let mut output = MarginUnits::default();

        if !self.node_kind(id).has_children() {
            return output;
        }

        if let Some(panel_style) = self.panel_style(id) {
            if let Some(padding_top) = panel_style.padding_top {
                output = padding_top;
            }
        }

        output
    }

    // all of these unwrap_or_default
    fn node_padding_bottom(&self, id: &NodeId) -> MarginUnits {
        let mut output = MarginUnits::default();

        if !self.node_kind(id).has_children() {
            return output;
        }

        if let Some(panel_style) = self.panel_style(id) {
            if let Some(padding_bottom) = panel_style.padding_bottom {
                output = padding_bottom;
            }
        }

        output
    }

    // all of these unwrap_or_default
    fn node_row_between(&self, id: &NodeId) -> MarginUnits {
        let mut output = MarginUnits::default();

        if !self.node_kind(id).has_children() {
            return output;
        }

        if let Some(panel_style) = self.panel_style(id) {
            if let Some(row_between) = panel_style.row_between {
                output = row_between;
            }
        }

        output
    }

    // all of these unwrap_or_default
    fn node_col_between(&self, id: &NodeId) -> MarginUnits {
        let mut output = MarginUnits::default();

        if !self.node_kind(id).has_children() {
            return output;
        }

        if let Some(panel_style) = self.panel_style(id) {
            if let Some(col_between) = panel_style.col_between {
                output = col_between;
            }
        }

        output
    }

    // no default here .. None doesn't do anything, Some does
    fn node_is_viewport(&self, id: &NodeId) -> bool {
        if self.node_kind(id) != WidgetKind::Panel {
            return false;
        }

        if let Some(panel_style) = self.panel_style(id) {
            panel_style.is_viewport
        } else {
            false
        }
    }

    // no default here .. None doesn't do anything, Some does
    fn node_is_solid(&self, id: &NodeId) -> Option<Solid> {
        if !self.node_kind(id).can_solid() {
            return None;
        }

        self.node_style(id)?.solid_override
    }

    fn node_is_text(&self, id: &NodeId) -> bool {
        self.node_kind(id).is_text()
    }

    fn node_calculate_text_width(
        &self,
        text_measurer: &dyn TextMeasurer,
        height: f32,
        text: &str,
    ) -> f32 {
        let (raw_width, raw_height) = text_measure_raw_size(text_measurer, text);
        let scale = height / raw_height;
        raw_width * scale
    }

    // panics if solid() is None but this isn't ..
    fn node_aspect_ratio(&self, id: &NodeId) -> Option<f32> {
        let mut output = 1.0; // TODO: put this into a constant

        if !self.node_kind(id).can_solid() {
            return Some(output);
        }

        if let Some(node_style) = self.node_style(id) {
            if let Some((w, h)) = node_style.aspect_ratio() {
                output = w / h;
            }
        }

        Some(output)
    }

    // all of these unwrap_or_default
    fn node_self_halign(&self, id: &NodeId) -> Alignment {
        let mut output = Alignment::default();

        if let Some(node_style) = self.node_style(id) {
            if let Some(self_halign) = node_style.self_halign {
                output = self_halign;
            }
        }

        output
    }

    // all of these unwrap_or_default
    fn node_self_valign(&self, id: &NodeId) -> Alignment {
        let mut output = Alignment::default();

        if let Some(node_style) = self.node_style(id) {
            if let Some(self_valign) = node_style.self_valign {
                output = self_valign;
            }
        }

        output
    }

    // all of these unwrap_or_default
    fn node_children_halign(&self, id: &NodeId) -> Alignment {
        let mut output = Alignment::default();

        if !self.node_kind(id).has_children() {
            return output;
        }

        if let Some(panel_style) = self.panel_style(id) {
            if let Some(children_halign) = panel_style.children_halign {
                output = children_halign;
            }
        }

        output
    }

    // all of these unwrap_or_default
    fn node_children_valign(&self, id: &NodeId) -> Alignment {
        let mut output = Alignment::default();

        if !self.node_kind(id).has_children() {
            return output;
        }

        if let Some(panel_style) = self.panel_style(id) {
            if let Some(children_valign) = panel_style.children_valign {
                output = children_valign;
            }
        }

        output
    }
}
