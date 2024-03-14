use std::collections::HashMap;

use naia_serde::{FileBitWriter, SerdeInternal as Serde};

use ui::{NodeStyle, Panel, PanelStyle, StyleId, Text, TextStyle, Ui, UiNode, Widget, WidgetKind, WidgetStyle};
use ui_layout::{Alignment, LayoutType, MarginUnits, PositionType, SizeUnits, Solid};

use crate::bits::{AlignmentBits, LayoutTypeBits, MarginUnitsBits, PanelBits, PanelStyleBits, PositionTypeBits, SizeUnitsBits, SolidBits, TextBits, TextStyleBits, UiAction, UiActionType, UiNodeBits, UiStyleBits, WidgetBits, WidgetStyleBits};

pub fn write_bits(ui: &Ui) -> Vec<u8> {
    let actions = convert_ui_to_actions(ui);
    actions_to_bytes(actions)
}

fn convert_ui_to_actions(ui: &Ui) -> Vec<UiAction> {
    let mut output = Vec::new();

    // write text color
    let text_color = ui.get_text_color();
    output.push(UiAction::TextColor(text_color.r, text_color.g, text_color.b));

    // write text icon AssetId
    let text_icon_asset_id = ui.get_text_icon_handle().asset_id();
    output.push(UiAction::TextIconAssetId(text_icon_asset_id));

    // write styles

    let mut style_id_to_index = HashMap::new();
    let mut style_count = 0;

    for (style_id, style) in ui.store.styles.iter().enumerate() {
        let style_id = StyleId::new(style_id as u32);
        if style_id == Ui::BASE_TEXT_STYLE_ID {
            continue;
        }
        let next_index = style_count;
        style_count += 1;
        style_id_to_index.insert(style_id, next_index);

        output.push(UiAction::Style(UiStyleBits::from_style(style)));
    }

    // write nodes
    for node in ui.store.nodes.iter() {
        output.push(UiAction::Node(UiNodeBits::from_node(&style_id_to_index, node)));
    }

    output
}

fn actions_to_bytes(actions: Vec<UiAction>) -> Vec<u8> {
    let mut bit_writer = FileBitWriter::new();

    for action in actions {
        match action {
            UiAction::TextColor(r, g, b) => {
                UiActionType::TextColor.ser(&mut bit_writer);
                r.ser(&mut bit_writer);
                g.ser(&mut bit_writer);
                b.ser(&mut bit_writer);

            }
            UiAction::TextIconAssetId(asset_id) => {
                UiActionType::TextIconAssetId.ser(&mut bit_writer);
                asset_id.as_u32().ser(&mut bit_writer);
            }
            UiAction::Style(style) => {
                UiActionType::Style.ser(&mut bit_writer);
                style.ser(&mut bit_writer);
            }
            UiAction::Node(node) => {
                UiActionType::Node.ser(&mut bit_writer);
                node.ser(&mut bit_writer);
            }
        }
    }

    // continue bit
    UiActionType::None.ser(&mut bit_writer);

    bit_writer.to_bytes().to_vec()
}

// conversion
impl UiStyleBits {
    pub(crate) fn from_style(style: &NodeStyle) -> Self {
        Self {
            widget_style: WidgetStyleBits::from_style(&style.widget_style),

            position_type: style.position_type.map(PositionTypeBits::from_position_type),

            width: style.width.map(SizeUnitsBits::from_size_units),
            height: style.height.map(SizeUnitsBits::from_size_units),
            width_min: style.width_min.map(SizeUnitsBits::from_size_units),
            width_max: style.width_max.map(SizeUnitsBits::from_size_units),
            height_min: style.height_min.map(SizeUnitsBits::from_size_units),
            height_max: style.height_max.map(SizeUnitsBits::from_size_units),

            margin_left: style.margin_left.map(MarginUnitsBits::from_margin_units),
            margin_right: style.margin_right.map(MarginUnitsBits::from_margin_units),
            margin_top: style.margin_top.map(MarginUnitsBits::from_margin_units),
            margin_bottom: style.margin_bottom.map(MarginUnitsBits::from_margin_units),

            solid_override: style.solid_override.map(SolidBits::from_solid),
            aspect_ratio_w_over_h: todo!(),

            self_halign: style.self_halign.map(AlignmentBits::from_alignment),
            self_valign: style.self_valign.map(AlignmentBits::from_alignment),
        }
    }
}

impl WidgetStyleBits {
    fn from_style(style: &WidgetStyle) -> Self {
        match style {
            WidgetStyle::Panel(panel) => Self::Panel(PanelStyleBits::from_panel_style(panel)),
            WidgetStyle::Text(text) => Self::Text(TextStyleBits::from_text_style(text)),
        }
    }
}

impl PanelStyleBits {
    fn from_panel_style(style: &PanelStyle) -> Self {
        Self {
            background_color: style.background_color.map(|val| {
                todo!()
            }),
            background_alpha: style.background_alpha.map(|val| {
                todo!()
            }),

            layout_type: style.layout_type.map(LayoutTypeBits::from_layout_type),

            padding_left: style.padding_left.map(SizeUnitsBits::from_size_units),
            padding_right: style.padding_right.map(SizeUnitsBits::from_size_units),
            padding_top: style.padding_top.map(SizeUnitsBits::from_size_units),
            padding_bottom: style.padding_bottom.map(SizeUnitsBits::from_size_units),

            row_between: style.row_between.map(SizeUnitsBits::from_size_units),
            col_between: style.col_between.map(SizeUnitsBits::from_size_units),
            children_halign: style.children_halign.map(AlignmentBits::from_alignment),
            children_valign: style.children_valign.map(AlignmentBits::from_alignment),
        }
    }
}

impl TextStyleBits {
    fn from_text_style(_style: &TextStyle) -> Self {
        Self {

        }
    }
}

impl PositionTypeBits {
    fn from_position_type(position_type: PositionType) -> Self {
        match position_type {
            PositionType::Absolute => Self::Absolute,
            PositionType::Relative => Self::Relative,
        }
    }
}

impl SizeUnitsBits {
    fn from_size_units(size_units: SizeUnits) -> Self {
        match size_units {
            SizeUnits::Pixels(val) => Self::Pixels(todo!()),
            SizeUnits::Percentage(val) => Self::Percent(todo!()),
            SizeUnits::Auto => Self::Auto,
        }
    }
}

impl MarginUnitsBits {
    fn from_margin_units(margin_units: MarginUnits) -> Self {
        match margin_units {
            MarginUnits::Pixels(val) => Self::Pixels(todo!()),
            MarginUnits::Percentage(val) => Self::Percent(todo!()),
        }
    }
}

impl SolidBits {
    fn from_solid(solid: Solid) -> Self {
        match solid {
            Solid::Fit => Self::Fit,
            Solid::Fill => Self::Fill,
        }
    }
}

impl AlignmentBits {
    fn from_alignment(alignment: Alignment) -> Self {
        match alignment {
            Alignment::Start => Self::Start,
            Alignment::Center => Self::Center,
            Alignment::End => Self::End,
        }
    }
}

impl LayoutTypeBits {
    fn from_layout_type(layout_type: LayoutType) -> Self {
        match layout_type {
            LayoutType::Row => Self::Row,
            LayoutType::Column => Self::Column,
        }
    }
}

impl UiNodeBits {
    fn from_node(style_id_to_index: &HashMap<StyleId, u8>, node: &UiNode) -> Self {
        let mut me = Self {
            visible: node.visible,
            style_ids: Vec::new(),
            widget: WidgetBits::from_widget(node.kind, node.widget.as_ref()),
        };

        for style_id in &node.style_ids {
            if style_id == &Ui::BASE_TEXT_STYLE_ID {
                continue;
            }
            let style_index: u8 = *style_id_to_index.get(style_id).unwrap();
            me.style_ids.push(style_index);
        }

        me
    }
}

impl WidgetBits {
    fn from_widget(kind: WidgetKind, widget: &dyn Widget) -> Self {
        match kind {
            WidgetKind::Panel => {
                let panel = UiNode::downcast_ref::<Panel>(widget).unwrap();
                Self::Panel(PanelBits::from_panel(panel))
            },
            WidgetKind::Text => {
                let text = UiNode::downcast_ref::<Text>(widget).unwrap();
                Self::Text(TextBits::from_text(text))
            },
        }
    }
}

impl PanelBits {
    fn from_panel(panel: &Panel) -> Self {
        let mut me = Self {
            children: Vec::new(),
        };
        for child_id in panel.children.iter() {
            me.children.push(child_id.as_usize() as u8); // TODO: cleanup conversion
        }
        me
    }
}

impl TextBits {
    fn from_text(text: &Text) -> Self {
        Self {
            text: text.inner_text().to_string(),
        }
    }
}