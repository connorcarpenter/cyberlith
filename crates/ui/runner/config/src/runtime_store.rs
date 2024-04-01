use std::slice::Iter;

use render_api::base::Color;

use ui_builder_config::{BaseNodeStyle, Button, ButtonStyle, Panel, PanelStyle, StyleId, Text, TextboxStyle, TextStyle, UiNode, UiStore, WidgetKind, WidgetStyle};
use ui_layout::{Alignment, LayoutType, MarginUnits, NodeId, NodeStore, PositionType, SizeUnits, Solid, TextMeasurer};

use crate::utils::text_measure_raw_size;

pub struct UiRuntimeStore {
    styles: Vec<BaseNodeStyle>,
    nodes: Vec<UiNode>,
}

impl UiRuntimeStore {
    pub fn new(store: UiStore) -> Self {

        let (styles, nodes) = store.decompose();

        let styles = styles.into_iter().map(|style| style.base).collect();
        let nodes = nodes.into_iter().map(|node| node.into()).collect();

        Self {
            styles,
            nodes,
        }
    }

    // nodes

    pub fn nodes_len(&self) -> usize {
        self.nodes.len()
    }

    pub fn nodes_iter(&self) -> Iter<'_, UiNode> {
        self.nodes.iter()
    }

    pub fn get_node(&self, node_id: &NodeId) -> Option<&UiNode> {
        self.nodes.get(node_id.as_usize())
    }

    pub fn node_kind(&self, node_id: &NodeId) -> WidgetKind {
        self.get_node(node_id).unwrap().widget_kind()
    }

    pub fn panel_ref(&self, node_id: &NodeId) -> Option<&Panel> {
        let node = self.get_node(node_id)?;
        if node.widget_kind() == WidgetKind::Panel {
            return node.widget_panel_ref();
        }
        None
    }

    pub fn button_ref(&self, node_id: &NodeId) -> Option<&Button> {
        let node = self.get_node(node_id)?;
        if node.widget_kind() == WidgetKind::Button {
            return node.widget_button_ref();
        }
        None
    }

    pub fn text_ref(&self, node_id: &NodeId) -> Option<&Text> {
        let node = self.get_node(node_id)?;
        if node.widget_kind() == WidgetKind::Text {
            return node.widget_text_ref();
        }
        None
    }

    // styles

    pub fn get_style(&self, style_id: &StyleId) -> Option<&BaseNodeStyle> {
        self.styles.get(style_id.as_usize())
    }

    pub fn node_background_color(&self, node_id: &NodeId) -> Option<&Color> {
        match self.widget_style(node_id)? {
            WidgetStyle::Text(text_style) => text_style.background_color.as_ref(),
            WidgetStyle::Button(button_style) => button_style.panel.background_color.as_ref(),
            WidgetStyle::Textbox(textbox_style) => textbox_style.background_color.as_ref(),
            WidgetStyle::Panel(panel_style) => panel_style.background_color.as_ref(),
        }
    }

    pub fn node_style(&self, node_id: &NodeId) -> Option<&BaseNodeStyle> {
        let node = self.get_node(node_id)?;
        node.style_id().map(|style_id| self.get_style(&style_id)).flatten()
    }

    fn widget_style(&self, node_id: &NodeId) -> Option<&WidgetStyle> {
        let style = self.node_style(node_id)?;
        Some(&style.widget_style)
    }

    pub fn panel_style(&self, node_id: &NodeId) -> Option<&PanelStyle> {
        let widget_style = self.widget_style(node_id)?;
        match widget_style {
            WidgetStyle::Panel(panel_style) => Some(panel_style),
            WidgetStyle::Button(button_style) => Some(&button_style.panel),
            _ => None,
        }
    }

    pub fn text_style(&self, node_id: &NodeId) -> Option<&TextStyle> {
        let widget_style = self.widget_style(node_id)?;
        match widget_style {
            WidgetStyle::Text(text_style) => Some(text_style),
            _ => None,
        }
    }

    pub fn button_style(&self, node_id: &NodeId) -> Option<&ButtonStyle> {
        let widget_style = self.widget_style(node_id)?;
        match widget_style {
            WidgetStyle::Button(button_style) => Some(button_style),
            _ => None,
        }
    }

    pub fn textbox_style(&self, node_id: &NodeId) -> Option<&TextboxStyle> {
        let widget_style = self.widget_style(node_id)?;
        match widget_style {
            WidgetStyle::Textbox(textbox_style) => Some(textbox_style),
            _ => None,
        }
    }
}

impl NodeStore for UiRuntimeStore {
    
    fn node_children<'t>(&'t self, id: &NodeId) -> Iter<NodeId> {
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
    fn node_padding_left(&self, id: &NodeId) -> SizeUnits {
        let mut output = SizeUnits::default();

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
    fn node_padding_right(&self, id: &NodeId) -> SizeUnits {
        let mut output = SizeUnits::default();

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
    fn node_padding_top(&self, id: &NodeId) -> SizeUnits {
        let mut output = SizeUnits::default();

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
    fn node_padding_bottom(&self, id: &NodeId) -> SizeUnits {
        let mut output = SizeUnits::default();

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
    fn node_row_between(&self, id: &NodeId) -> SizeUnits {
        let mut output = SizeUnits::default();

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
    fn node_col_between(&self, id: &NodeId) -> SizeUnits {
        let mut output = SizeUnits::default();

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
    fn node_is_solid(&self, id: &NodeId) -> Option<Solid> {
        if !self.node_kind(id).can_solid() {
            return None;
        }

        self.node_style(id)?.solid_override
    }

    fn node_is_text(&self, id: &NodeId) -> bool {
        self.node_kind(id).is_text()
    }

    fn node_calculate_text_width(&self, id: &NodeId, text_measurer: &dyn TextMeasurer, height: f32) -> f32 {
        let text_ref = self.text_ref(id).unwrap();
        let text = text_ref.text.as_str();
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