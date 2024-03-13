use std::collections::HashMap;

use ui_layout::{Alignment, LayoutType, MarginUnits, PositionType, SizeUnits, Solid};
use render_api::base::Color;
use ui::{
    UiNode, WidgetKind, PanelStyle, Panel, NodeStyle,StyleId,
    WidgetStyle, TextStyle, Text, Ui, Widget
};

use super::{
    UiNodeSerde, PanelSerde, WidgetSerde, UiSerde, ColorSerde, UiStyleSerde, SizeUnitsSerde,
    PositionTypeSerde, WidgetStyleSerde, MarginUnitsSerde, SolidSerde, AlignmentSerde, PanelStyleSerde,
    TextStyleSerde, LayoutTypeSerde, TextSerde
};

impl UiSerde {

    pub(crate) fn from_ui(ui: &Ui) -> Self {

        let mut style_id_to_index = HashMap::new();

        let text_color = ColorSerde::from_color(ui.get_text_color());
        let text_icon_asset_id = ui.get_text_icon_handle().asset_id().to_string();

        let mut me = Self {
            text_color,
            text_icon_asset_id,
            styles: Vec::new(),
            nodes: Vec::new(),
        };

        // styles
        for (style_id, style) in ui.store.styles.iter().enumerate() {
            let style_id = StyleId::new(style_id as u32);
            if style_id == Ui::BASE_TEXT_STYLE_ID {
                continue;
            }
            let next_index = me.styles.len();
            style_id_to_index.insert(style_id, next_index);
            me.styles.push(UiStyleSerde::from_style(style));
        }

        // nodes
        for node in ui.store.nodes.iter() {
            me.nodes.push(UiNodeSerde::from_node(&style_id_to_index, node));
        }

        me
    }
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
}

impl WidgetStyleSerde {
    fn from_style(style: &WidgetStyle) -> Self {
        match style {
            WidgetStyle::Panel(panel) => Self::Panel(PanelStyleSerde::from_panel_style(panel)),
            WidgetStyle::Text(text) => Self::Text(TextStyleSerde::from_text_style(text)),
        }
    }
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
}

impl TextStyleSerde {
    fn from_text_style(_style: &TextStyle) -> Self {
        Self {

        }
    }
}

impl PositionTypeSerde {
    fn from_position_type(position_type: PositionType) -> Self {
        match position_type {
            PositionType::Absolute => Self::Absolute,
            PositionType::Relative => Self::Relative,
        }
    }
}

impl SizeUnitsSerde {
    fn from_size_units(size_units: SizeUnits) -> Self {
        match size_units {
            SizeUnits::Pixels(pixels) => Self::Pixels(pixels),
            SizeUnits::Percentage(percentage) => Self::Percentage(percentage),
            SizeUnits::Auto => Self::Auto,
        }
    }
}

impl MarginUnitsSerde {
    fn from_margin_units(margin_units: MarginUnits) -> Self {
        match margin_units {
            MarginUnits::Pixels(pixels) => Self::Pixels(pixels),
            MarginUnits::Percentage(percentage) => Self::Percentage(percentage),
        }
    }
}

impl SolidSerde {
    fn from_solid(solid: Solid) -> Self {
        match solid {
            Solid::Fit => Self::Fit,
            Solid::Fill => Self::Fill,
        }
    }
}

impl AlignmentSerde {
    fn from_alignment(alignment: Alignment) -> Self {
        match alignment {
            Alignment::Start => Self::Start,
            Alignment::Center => Self::Center,
            Alignment::End => Self::End,
        }
    }
}

impl LayoutTypeSerde {
    fn from_layout_type(layout_type: LayoutType) -> Self {
        match layout_type {
            LayoutType::Row => Self::Row,
            LayoutType::Column => Self::Column,
        }
    }
}

impl ColorSerde {
    fn from_color(color: Color) -> Self {
        Self {
            r: color.r,
            g: color.g,
            b: color.b,
        }
    }
}

impl UiNodeSerde {
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

impl TextSerde {
    fn from_text(text: &Text) -> Self {
        Self {
            text: text.inner_text().to_string(),
        }
    }
}