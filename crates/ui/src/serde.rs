use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use asset_id::AssetId;
use layout::{Alignment, LayoutType, MarginUnits, PositionType, SizeUnits, Solid};
use render_api::base::Color;

use crate::{node::{UiNode, WidgetKind}, panel::{PanelStyle, PanelStyleMut, Panel}, style::{NodeStyle,StyleId, WidgetStyle}, text::{TextStyle, TextStyleMut, Text}, Ui, widget::Widget};
use crate::panel::{PanelContentsMut, PanelMut};

impl Ui {
    pub fn write_json(&self) -> Vec<u8> {
        let ui_serde = UiSerde::from_ui(self);
        serde_json::to_vec_pretty(&ui_serde).unwrap()
    }

    pub fn read_json(data: Vec<u8>) -> Self {
        let ui_serde: UiSerde = serde_json::from_slice(data.as_slice()).unwrap();

        let mut ui = Ui::new();

        // ui_serde -> ui
        let UiSerde {
            styles,
            nodes,
        } = ui_serde;

        let mut style_index_to_id = HashMap::new();

        // styles
        for (style_index, style_serde) in styles.iter().enumerate() {
            if style_index == 0 {
                // style 0 is for root, which already exists
                style_index_to_id.insert(style_index, Ui::ROOT_STYLE_ID);
            } else {
                let style_id = match style_serde.widget_kind() {
                    WidgetKind::Panel => ui.create_panel_style(|style| {
                        style_serde_to_panel_style(style_serde, style);
                    }),
                    WidgetKind::Text => ui.create_text_style(|style| {
                        style_serde_to_text_style(style_serde, style);
                    }),
                };
                style_index_to_id.insert(style_index, style_id);
            }
        }

        // nodes
        let root_node_serde = nodes.first().unwrap();
        let mut root_mut = ui.root_mut();
        root_mut.set_visible(root_node_serde.visible);
        for style_index in &root_node_serde.style_ids {
            let style_id = *style_index_to_id.get(style_index).unwrap();
            root_mut.add_style(style_id);
        }
        let WidgetSerde::Panel(panel_serde) = &root_node_serde.widget else {
            panic!("Expected panel widget");
        };
        convert_nodes_recurse(&style_index_to_id, &nodes, panel_serde, &mut root_mut);

        ui
    }
}

fn convert_nodes_recurse<'a>(
    style_index_to_id: &HashMap<usize, StyleId>,
    nodes: &Vec<UiNodeSerde>,
    panel_serde: &PanelSerde,
    panel_mut: &'a mut PanelMut<'a>
) {
    panel_mut.contents(|c| {
        for child_index in &panel_serde.children {
            let child_index = *child_index;
            let child_node_serde = &nodes[child_index];
            match child_node_serde.widget_kind() {
                WidgetKind::Panel => {
                    let mut child_panel_mut = c.add_panel();
                    child_panel_mut.set_visible(child_node_serde.visible);
                    for style_index in &child_node_serde.style_ids {
                        let style_id = *style_index_to_id.get(style_index).unwrap();
                        child_panel_mut.add_style(style_id);
                    }
                    let WidgetSerde::Panel(child_panel_serde) = &child_node_serde.widget else {
                        panic!("Expected panel widget");
                    };
                    convert_nodes_recurse(style_index_to_id, nodes, child_panel_serde, &mut child_panel_mut);
                }
                WidgetKind::Text => {
                    let WidgetSerde::Text(child_text_serde) = &child_node_serde.widget else {
                        panic!("Expected text widget");
                    };
                    let mut child_text_mut = c.add_text(child_text_serde.text.as_str());
                    child_text_mut.set_visible(child_node_serde.visible);
                    for style_index in &child_node_serde.style_ids {
                        let style_id = *style_index_to_id.get(style_index).unwrap();
                        child_text_mut.add_style(style_id);
                    }
                }
            }
        }
    });
}

#[derive(Serialize, Deserialize, Clone)]
struct UiSerde {
    styles: Vec<UiStyleSerde>,
    nodes: Vec<UiNodeSerde>,
}

impl UiSerde {
    const CURRENT_SCHEMA_VERSION: u32 = 0;

    fn from_ui(ui: &Ui) -> Self {

        let mut style_id_to_index = HashMap::new();

        let mut me = Self {
            styles: Vec::new(),
            nodes: Vec::new(),
        };

        // styles
        for (style_id, style) in ui.styles_iter() {
            if *style_id == Ui::BASE_TEXT_STYLE_ID {
                continue;
            }
            let next_index = me.styles.len();
            style_id_to_index.insert(*style_id, next_index);
            me.styles.push(UiStyleSerde::from_style(style));
        }

        // nodes
        for (_, node) in ui.nodes_iter() {
            me.nodes.push(UiNodeSerde::from_node(&style_id_to_index, node));
        }

        me
    }

    fn dependencies(&self) -> Vec<AssetId> {
        Vec::new()
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct UiStyleSerde {

    widget_style: WidgetStyleSerde,

    position_type: Option<PositionTypeSerde>,

    width: Option<SizeUnitsSerde>,
    height: Option<SizeUnitsSerde>,
    width_min: Option<SizeUnitsSerde>,
    width_max: Option<SizeUnitsSerde>,
    height_min: Option<SizeUnitsSerde>,
    height_max: Option<SizeUnitsSerde>,

    margin_left: Option<MarginUnitsSerde>,
    margin_right: Option<MarginUnitsSerde>,
    margin_top: Option<MarginUnitsSerde>,
    margin_bottom: Option<MarginUnitsSerde>,

    solid_override: Option<SolidSerde>,
    aspect_ratio_w_over_h: Option<f32>,

    self_halign: Option<AlignmentSerde>,
    self_valign: Option<AlignmentSerde>,
}

impl UiStyleSerde {
    fn from_style(style: &NodeStyle) -> Self {
        Self {
            widget_style: WidgetStyleSerde::from_style(&style.widget_style),

            position_type: style.position_type.map(PositionTypeSerde::from_position_type),

            width: style.width.map(SizeUnitsSerde::from_size_units),
            height: style.height.map(SizeUnitsSerde::from_size_units),
            width_min: style.width_min.map(SizeUnitsSerde::from_size_units),
            width_max: style.width_max.map(SizeUnitsSerde::from_size_units),
            height_min: style.height_min.map(SizeUnitsSerde::from_size_units),
            height_max: style.height_max.map(SizeUnitsSerde::from_size_units),

            margin_left: style.margin_left.map(MarginUnitsSerde::from_margin_units),
            margin_right: style.margin_right.map(MarginUnitsSerde::from_margin_units),
            margin_top: style.margin_top.map(MarginUnitsSerde::from_margin_units),
            margin_bottom: style.margin_bottom.map(MarginUnitsSerde::from_margin_units),

            solid_override: style.solid_override.map(SolidSerde::from_solid),
            aspect_ratio_w_over_h: style.aspect_ratio_w_over_h,

            self_halign: style.self_halign.map(AlignmentSerde::from_alignment),
            self_valign: style.self_valign.map(AlignmentSerde::from_alignment),
        }
    }

    fn to_style(&self) -> NodeStyle {
        NodeStyle {
            widget_style: self.widget_style.to_style(),

            position_type: self.position_type.as_ref().map(PositionTypeSerde::to_position_type),

            width: self.width.as_ref().map(SizeUnitsSerde::to_size_units),
            height: self.height.as_ref().map(SizeUnitsSerde::to_size_units),
            width_min: self.width_min.as_ref().map(SizeUnitsSerde::to_size_units),
            width_max: self.width_max.as_ref().map(SizeUnitsSerde::to_size_units),
            height_min: self.height_min.as_ref().map(SizeUnitsSerde::to_size_units),
            height_max: self.height_max.as_ref().map(SizeUnitsSerde::to_size_units),

            margin_left: self.margin_left.as_ref().map(MarginUnitsSerde::to_margin_units),
            margin_right: self.margin_right.as_ref().map(MarginUnitsSerde::to_margin_units),
            margin_top: self.margin_top.as_ref().map(MarginUnitsSerde::to_margin_units),
            margin_bottom: self.margin_bottom.as_ref().map(MarginUnitsSerde::to_margin_units),

            solid_override: self.solid_override.as_ref().map(SolidSerde::to_solid),
            aspect_ratio_w_over_h: self.aspect_ratio_w_over_h,

            self_halign: self.self_halign.as_ref().map(AlignmentSerde::to_alignment),
            self_valign: self.self_valign.as_ref().map(AlignmentSerde::to_alignment),
        }
    }

    fn widget_kind(&self) -> WidgetKind {
        match &self.widget_style {
            WidgetStyleSerde::Panel(_) => WidgetKind::Panel,
            WidgetStyleSerde::Text(_) => WidgetKind::Text,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
enum WidgetStyleSerde {
    Panel(PanelStyleSerde),
    Text(TextStyleSerde),
}

impl WidgetStyleSerde {
    fn from_style(style: &WidgetStyle) -> Self {
        match style {
            WidgetStyle::Panel(panel) => Self::Panel(PanelStyleSerde::from_panel_style(panel)),
            WidgetStyle::Text(text) => Self::Text(TextStyleSerde::from_text_style(text)),
        }
    }

    fn to_style(&self) -> WidgetStyle {
        match self {
            Self::Panel(panel) => WidgetStyle::Panel(panel.to_panel_style()),
            Self::Text(text) => WidgetStyle::Text(text.to_text_style()),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct PanelStyleSerde {
    background_color: Option<ColorSerde>,
    background_alpha: Option<f32>,

    layout_type: Option<LayoutTypeSerde>,

    padding_left: Option<SizeUnitsSerde>,
    padding_right: Option<SizeUnitsSerde>,
    padding_top: Option<SizeUnitsSerde>,
    padding_bottom: Option<SizeUnitsSerde>,

    row_between: Option<SizeUnitsSerde>,
    col_between: Option<SizeUnitsSerde>,
    children_halign: Option<AlignmentSerde>,
    children_valign: Option<AlignmentSerde>,
}

impl PanelStyleSerde {
    fn from_panel_style(style: &PanelStyle) -> Self {
        Self {
            background_color: style.background_color.map(ColorSerde::from_color),
            background_alpha: style.background_alpha,

            layout_type: style.layout_type.map(LayoutTypeSerde::from_layout_type),

            padding_left: style.padding_left.map(SizeUnitsSerde::from_size_units),
            padding_right: style.padding_right.map(SizeUnitsSerde::from_size_units),
            padding_top: style.padding_top.map(SizeUnitsSerde::from_size_units),
            padding_bottom: style.padding_bottom.map(SizeUnitsSerde::from_size_units),

            row_between: style.row_between.map(SizeUnitsSerde::from_size_units),
            col_between: style.col_between.map(SizeUnitsSerde::from_size_units),
            children_halign: style.children_halign.map(AlignmentSerde::from_alignment),
            children_valign: style.children_valign.map(AlignmentSerde::from_alignment),
        }
    }

    fn to_panel_style(&self) -> PanelStyle {
        PanelStyle {
            background_color: self.background_color.as_ref().map(ColorSerde::to_color),
            background_alpha: self.background_alpha,

            layout_type: self.layout_type.as_ref().map(LayoutTypeSerde::to_layout_type),

            padding_left: self.padding_left.as_ref().map(SizeUnitsSerde::to_size_units),
            padding_right: self.padding_right.as_ref().map(SizeUnitsSerde::to_size_units),
            padding_top: self.padding_top.as_ref().map(SizeUnitsSerde::to_size_units),
            padding_bottom: self.padding_bottom.as_ref().map(SizeUnitsSerde::to_size_units),

            row_between: self.row_between.as_ref().map(SizeUnitsSerde::to_size_units),
            col_between: self.col_between.as_ref().map(SizeUnitsSerde::to_size_units),
            children_halign: self.children_halign.as_ref().map(AlignmentSerde::to_alignment),
            children_valign: self.children_valign.as_ref().map(AlignmentSerde::to_alignment),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct TextStyleSerde {

}

impl TextStyleSerde {
    fn from_text_style(style: &TextStyle) -> Self {
        Self {

        }
    }

    fn to_text_style(&self) -> TextStyle {
        TextStyle {

        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
enum PositionTypeSerde {
    Absolute,
    Relative,
}

impl PositionTypeSerde {
    fn from_position_type(position_type: PositionType) -> Self {
        match position_type {
            PositionType::Absolute => Self::Absolute,
            PositionType::Relative => Self::Relative,
        }
    }

    fn to_position_type(&self) -> PositionType {
        match self {
            Self::Absolute => PositionType::Absolute,
            Self::Relative => PositionType::Relative,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
enum SizeUnitsSerde {
    Pixels(f32),
    Percentage(f32),
    Auto,
}

impl SizeUnitsSerde {
    fn from_size_units(size_units: SizeUnits) -> Self {
        match size_units {
            SizeUnits::Pixels(pixels) => Self::Pixels(pixels),
            SizeUnits::Percentage(percentage) => Self::Percentage(percentage),
            SizeUnits::Auto => Self::Auto,
        }
    }

    fn to_size_units(&self) -> SizeUnits {
        match self {
            Self::Pixels(pixels) => SizeUnits::Pixels(*pixels),
            Self::Percentage(percentage) => SizeUnits::Percentage(*percentage),
            Self::Auto => SizeUnits::Auto,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
enum MarginUnitsSerde {
    Pixels(f32),
    Percentage(f32),
}

impl MarginUnitsSerde {
    fn from_margin_units(margin_units: MarginUnits) -> Self {
        match margin_units {
            MarginUnits::Pixels(pixels) => Self::Pixels(pixels),
            MarginUnits::Percentage(percentage) => Self::Percentage(percentage),
        }
    }

    fn to_margin_units(&self) -> MarginUnits {
        match self {
            Self::Pixels(pixels) => MarginUnits::Pixels(*pixels),
            Self::Percentage(percentage) => MarginUnits::Percentage(*percentage),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
enum SolidSerde {
    Fit,
    Fill,
}

impl SolidSerde {
    fn from_solid(solid: Solid) -> Self {
        match solid {
            Solid::Fit => Self::Fit,
            Solid::Fill => Self::Fill,
        }
    }

    fn to_solid(&self) -> Solid {
        match self {
            Self::Fit => Solid::Fit,
            Self::Fill => Solid::Fill,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
enum AlignmentSerde {
    Start,
    Center,
    End,
}

impl AlignmentSerde {
    fn from_alignment(alignment: Alignment) -> Self {
        match alignment {
            Alignment::Start => Self::Start,
            Alignment::Center => Self::Center,
            Alignment::End => Self::End,
        }
    }

    fn to_alignment(&self) -> Alignment {
        match self {
            Self::Start => Alignment::Start,
            Self::Center => Alignment::Center,
            Self::End => Alignment::End,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
enum LayoutTypeSerde {
    Row,
    Column,
}

impl LayoutTypeSerde {
    fn from_layout_type(layout_type: LayoutType) -> Self {
        match layout_type {
            LayoutType::Row => Self::Row,
            LayoutType::Column => Self::Column,
        }
    }

    fn to_layout_type(&self) -> LayoutType {
        match self {
            Self::Row => LayoutType::Row,
            Self::Column => LayoutType::Column,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct ColorSerde {
    r: u8,
    g: u8,
    b: u8,
}

impl ColorSerde {
    fn from_color(color: Color) -> Self {
        Self {
            r: color.r,
            g: color.g,
            b: color.b,
        }
    }

    fn to_color(&self) -> Color {
        Color::new(self.r, self.g, self.b)
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct UiNodeSerde {
    visible: bool,
    style_ids: Vec<usize>,
    widget: WidgetSerde,
}

impl UiNodeSerde {

    fn widget_kind(&self) -> WidgetKind {
        match &self.widget {
            WidgetSerde::Panel(_) => WidgetKind::Panel,
            WidgetSerde::Text(_) => WidgetKind::Text,
        }
    }

    fn from_node(style_id_to_index: &HashMap<StyleId, usize>, node: &UiNode) -> Self {
        let mut me = Self {
            visible: node.visible,
            style_ids: Vec::new(),
            widget: WidgetSerde::from_widget(node.kind, node.widget.as_ref()),
        };

        for style_id in &node.style_ids {
            if style_id == &Ui::BASE_TEXT_STYLE_ID {
                continue;
            }
            let style_index: usize = *style_id_to_index.get(style_id).unwrap();
            me.style_ids.push(style_index);
        }

        me
    }

    fn to_node(&self, index_to_style_id: &Vec<StyleId>, widget: Box<dyn Widget>) -> UiNode {
        let mut style_ids = Vec::new();
        for style_index in &self.style_ids {
            let style_id = index_to_style_id[*style_index];
            style_ids.push(style_id);
        }

        let kind = self.widget_kind();

        UiNode {
            visible: self.visible,
            style_ids,
            kind,
            widget,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
enum WidgetSerde {
    Panel(PanelSerde),
    Text(TextSerde),
}

impl WidgetSerde {
    fn from_widget(kind: WidgetKind, widget: &dyn Widget) -> Self {
        match kind {
            WidgetKind::Panel => {
                let panel = UiNode::downcast_ref::<Panel>(widget).unwrap();
                Self::Panel(PanelSerde::from_panel(panel))
            },
            WidgetKind::Text => {
                let text = UiNode::downcast_ref::<Text>(widget).unwrap();
                Self::Text(TextSerde::from_text(text))
            },
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct PanelSerde {
    children: Vec<usize>,
}

impl PanelSerde {
    fn from_panel(panel: &Panel) -> Self {
        let mut me = Self {
            children: Vec::new(),
        };
        for child_id in panel.children.iter() {
            me.children.push(child_id.as_usize());
        }
        me
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct TextSerde {
    text: String,
}

impl TextSerde {
    fn from_text(text: &Text) -> Self {
        Self {
            text: text.inner_text().to_string(),
        }
    }
}

// conversion
fn style_serde_to_panel_style(style_serde: &UiStyleSerde, style: &mut PanelStyleMut) {
    // node-specific
    if let Some(position_type_serde) = &style_serde.position_type {
        let position_type = position_type_serde.to_position_type();
        match position_type {
            PositionType::Absolute => style.set_absolute(),
            PositionType::Relative => style.set_relative(),
        };
    }
    if let Some(val_serde) = &style_serde.width {
        let val = val_serde.to_size_units();
        match val {
            SizeUnits::Pixels(pixels) => style.set_width_px(pixels),
            SizeUnits::Percentage(percentage) => style.set_width_pc(percentage),
            SizeUnits::Auto => style.set_width_auto(),
        };
    }
    if let Some(val_serde) = &style_serde.height {
        let val = val_serde.to_size_units();
        match val {
            SizeUnits::Pixels(pixels) => style.set_height_px(pixels),
            SizeUnits::Percentage(percentage) => style.set_height_pc(percentage),
            SizeUnits::Auto => style.set_height_auto(),
        };
    }
    if let Some(val_serde) = &style_serde.width_min {
        let val = val_serde.to_size_units();
        match val {
            SizeUnits::Pixels(pixels) => style.set_width_min_px(pixels),
            SizeUnits::Percentage(percentage) => style.set_width_min_pc(percentage),
            SizeUnits::Auto => style.set_width_min_auto(),
        };
    }
    if let Some(val_serde) = &style_serde.width_max {
        let val = val_serde.to_size_units();
        match val {
            SizeUnits::Pixels(pixels) => style.set_width_max_px(pixels),
            SizeUnits::Percentage(percentage) => style.set_width_max_pc(percentage),
            SizeUnits::Auto => style.set_width_max_auto(),
        };
    }
    if let Some(val_serde) = &style_serde.height_min {
        let val = val_serde.to_size_units();
        match val {
            SizeUnits::Pixels(pixels) => style.set_height_min_px(pixels),
            SizeUnits::Percentage(percentage) => style.set_height_min_pc(percentage),
            SizeUnits::Auto => style.set_height_min_auto(),
        };
    }
    if let Some(val_serde) = &style_serde.height_max {
        let val = val_serde.to_size_units();
        match val {
            SizeUnits::Pixels(pixels) => style.set_height_max_px(pixels),
            SizeUnits::Percentage(percentage) => style.set_height_max_pc(percentage),
            SizeUnits::Auto => style.set_height_max_auto(),
        };
    }
    if let Some(val_serde) = &style_serde.margin_left {
        let val = val_serde.to_margin_units();
        match val {
            MarginUnits::Pixels(pixels) => style.set_margin_left_px(pixels),
            MarginUnits::Percentage(percentage) => style.set_margin_left_pc(percentage),
        };
    }
    if let Some(val_serde) = &style_serde.margin_right {
        let val = val_serde.to_margin_units();
        match val {
            MarginUnits::Pixels(pixels) => style.set_margin_right_px(pixels),
            MarginUnits::Percentage(percentage) => style.set_margin_right_pc(percentage),
        };
    }
    if let Some(val_serde) = &style_serde.margin_top {
        let val = val_serde.to_margin_units();
        match val {
            MarginUnits::Pixels(pixels) => style.set_margin_top_px(pixels),
            MarginUnits::Percentage(percentage) => style.set_margin_top_pc(percentage),
        };
    }
    if let Some(val_serde) = &style_serde.margin_bottom {
        let val = val_serde.to_margin_units();
        match val {
            MarginUnits::Pixels(pixels) => style.set_margin_bottom_px(pixels),
            MarginUnits::Percentage(percentage) => style.set_margin_bottom_pc(percentage),
        };
    }
    if let Some(solid_override_serde) = &style_serde.solid_override {
        let solid_override = solid_override_serde.to_solid();
        match solid_override {
            Solid::Fit => style.set_solid_fit(),
            Solid::Fill => style.set_solid_fill(),
        };
    }
    if let Some(aspect_ratio_w_over_h) = style_serde.aspect_ratio_w_over_h {
        style.set_aspect_ratio(aspect_ratio_w_over_h, 1.0);
    }
    if let Some(val_serde) = &style_serde.self_halign {
        let val = val_serde.to_alignment();
        style.set_self_halign(val);
    }
    if let Some(val_serde) = &style_serde.self_valign {
        let val = val_serde.to_alignment();
        style.set_self_valign(val);
    }

    // panel-specific
    let WidgetStyleSerde::Panel(panel_style_serde) = &style_serde.widget_style else {
        panic!("Expected panel style");
    };

    if let Some(background_color_serde) = &panel_style_serde.background_color {
        style.set_background_color(background_color_serde.to_color());
    }
    if let Some(background_alpha) = panel_style_serde.background_alpha {
        style.set_background_alpha(background_alpha);
    }
    if let Some(layout_type_serde) = &panel_style_serde.layout_type {
        let layout_type = layout_type_serde.to_layout_type();
        match layout_type {
            LayoutType::Row => style.set_horizontal(),
            LayoutType::Column => style.set_vertical(),
        };
    }
    if let Some(val_serde) = &panel_style_serde.padding_left {
        let val = val_serde.to_size_units();
        match val {
            SizeUnits::Pixels(pixels) => style.set_padding_left_px(pixels),
            SizeUnits::Percentage(percentage) => style.set_padding_left_pc(percentage),
            SizeUnits::Auto => style.set_padding_left_auto(),
        };
    }
    if let Some(val_serde) = &panel_style_serde.padding_right {
        let val = val_serde.to_size_units();
        match val {
            SizeUnits::Pixels(pixels) => style.set_padding_right_px(pixels),
            SizeUnits::Percentage(percentage) => style.set_padding_right_pc(percentage),
            SizeUnits::Auto => style.set_padding_right_auto(),
        };
    }
    if let Some(val_serde) = &panel_style_serde.padding_top {
        let val = val_serde.to_size_units();
        match val {
            SizeUnits::Pixels(pixels) => style.set_padding_top_px(pixels),
            SizeUnits::Percentage(percentage) => style.set_padding_top_pc(percentage),
            SizeUnits::Auto => style.set_padding_top_auto(),
        };
    }
    if let Some(val_serde) = &panel_style_serde.padding_bottom {
        let val = val_serde.to_size_units();
        match val {
            SizeUnits::Pixels(pixels) => style.set_padding_bottom_px(pixels),
            SizeUnits::Percentage(percentage) => style.set_padding_bottom_pc(percentage),
            SizeUnits::Auto => style.set_padding_bottom_auto(),
        };
    }
    if let Some(val_serde) = &panel_style_serde.row_between {
        let val = val_serde.to_size_units();
        match val {
            SizeUnits::Pixels(pixels) => style.set_row_between_px(pixels),
            SizeUnits::Percentage(percentage) => style.set_row_between_pc(percentage),
            SizeUnits::Auto => style.set_row_between_auto(),
        };
    }
    if let Some(val_serde) = &panel_style_serde.col_between {
        let val = val_serde.to_size_units();
        match val {
            SizeUnits::Pixels(pixels) => style.set_col_between_px(pixels),
            SizeUnits::Percentage(percentage) => style.set_col_between_pc(percentage),
            SizeUnits::Auto => style.set_col_between_auto(),
        };
    }
    if let Some(val_serde) = &panel_style_serde.children_halign {
        let val = val_serde.to_alignment();
        style.set_children_halign(val);
    }
    if let Some(val_serde) = &panel_style_serde.children_valign {
        let val = val_serde.to_alignment();
        style.set_children_valign(val);
    }
}

fn style_serde_to_text_style(style_serde: &UiStyleSerde, style: &mut TextStyleMut) {
    // node-specific
    if let Some(position_type_serde) = &style_serde.position_type {
        let position_type = position_type_serde.to_position_type();
        match position_type {
            PositionType::Absolute => style.set_absolute(),
            PositionType::Relative => style.set_relative(),
        };
    }
    if let Some(val_serde) = &style_serde.width {
        let val = val_serde.to_size_units();
        match val {
            SizeUnits::Pixels(pixels) => style.set_width_px(pixels),
            SizeUnits::Percentage(percentage) => style.set_width_pc(percentage),
            SizeUnits::Auto => style.set_width_auto(),
        };
    }
    if let Some(val_serde) = &style_serde.height {
        let val = val_serde.to_size_units();
        match val {
            SizeUnits::Pixels(pixels) => style.set_height_px(pixels),
            SizeUnits::Percentage(percentage) => style.set_height_pc(percentage),
            SizeUnits::Auto => style.set_height_auto(),
        };
    }
    if let Some(val_serde) = &style_serde.width_min {
        let val = val_serde.to_size_units();
        match val {
            SizeUnits::Pixels(pixels) => style.set_width_min_px(pixels),
            SizeUnits::Percentage(percentage) => style.set_width_min_pc(percentage),
            SizeUnits::Auto => style.set_width_min_auto(),
        };
    }
    if let Some(val_serde) = &style_serde.width_max {
        let val = val_serde.to_size_units();
        match val {
            SizeUnits::Pixels(pixels) => style.set_width_max_px(pixels),
            SizeUnits::Percentage(percentage) => style.set_width_max_pc(percentage),
            SizeUnits::Auto => style.set_width_max_auto(),
        };
    }
    if let Some(val_serde) = &style_serde.height_min {
        let val = val_serde.to_size_units();
        match val {
            SizeUnits::Pixels(pixels) => style.set_height_min_px(pixels),
            SizeUnits::Percentage(percentage) => style.set_height_min_pc(percentage),
            SizeUnits::Auto => style.set_height_min_auto(),
        };
    }
    if let Some(val_serde) = &style_serde.height_max {
        let val = val_serde.to_size_units();
        match val {
            SizeUnits::Pixels(pixels) => style.set_height_max_px(pixels),
            SizeUnits::Percentage(percentage) => style.set_height_max_pc(percentage),
            SizeUnits::Auto => style.set_height_max_auto(),
        };
    }
    if let Some(val_serde) = &style_serde.margin_left {
        let val = val_serde.to_margin_units();
        match val {
            MarginUnits::Pixels(pixels) => style.set_margin_left_px(pixels),
            MarginUnits::Percentage(percentage) => style.set_margin_left_pc(percentage),
        };
    }
    if let Some(val_serde) = &style_serde.margin_right {
        let val = val_serde.to_margin_units();
        match val {
            MarginUnits::Pixels(pixels) => style.set_margin_right_px(pixels),
            MarginUnits::Percentage(percentage) => style.set_margin_right_pc(percentage),
        };
    }
    if let Some(val_serde) = &style_serde.margin_top {
        let val = val_serde.to_margin_units();
        match val {
            MarginUnits::Pixels(pixels) => style.set_margin_top_px(pixels),
            MarginUnits::Percentage(percentage) => style.set_margin_top_pc(percentage),
        };
    }
    if let Some(val_serde) = &style_serde.margin_bottom {
        let val = val_serde.to_margin_units();
        match val {
            MarginUnits::Pixels(pixels) => style.set_margin_bottom_px(pixels),
            MarginUnits::Percentage(percentage) => style.set_margin_bottom_pc(percentage),
        };
    }
    if let Some(solid_override_serde) = &style_serde.solid_override {
        let solid_override = solid_override_serde.to_solid();
        match solid_override {
            Solid::Fit => style.set_solid_fit(),
            Solid::Fill => style.set_solid_fill(),
        };
    }
    if let Some(aspect_ratio_w_over_h) = style_serde.aspect_ratio_w_over_h {
        style.set_aspect_ratio(aspect_ratio_w_over_h, 1.0);
    }
    if let Some(val_serde) = &style_serde.self_halign {
        let val = val_serde.to_alignment();
        style.set_self_halign(val);
    }
    if let Some(val_serde) = &style_serde.self_valign {
        let val = val_serde.to_alignment();
        style.set_self_valign(val);
    }
}