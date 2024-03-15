use std::collections::HashMap;

use render_api::base::Color;
use ui::{
    NodeStyle, Panel, PanelStyle, StyleId, Text, TextStyle, Ui, UiNode, Widget,
    WidgetStyle, Alignment, LayoutType, MarginUnits, PositionType, SizeUnits, Solid
};

use super::{
    AlignmentJson, ColorJson, LayoutTypeJson, MarginUnitsJson, PanelJson, PanelStyleJson,
    PositionTypeJson, SizeUnitsJson, SolidJson, TextJson, TextStyleJson, UiJson, UiNodeJson,
    UiStyleJson, WidgetJson, WidgetStyleJson,
};

// conversion

impl UiJson {
    pub fn from_ui(ui: &Ui) -> Self {
        let mut style_id_to_index = HashMap::new();

        let text_color = ColorJson::from_color(ui.get_text_color());
        let text_icon_asset_id = ui.get_text_icon_asset_id().to_string();

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
            me.styles.push(UiStyleJson::from_style(style));
        }

        // nodes
        for node in ui.store.nodes.iter() {
            me.nodes
                .push(UiNodeJson::from_node(&style_id_to_index, node));
        }

        me
    }
}

impl UiStyleJson {
    fn from_style(style: &NodeStyle) -> Self {
        Self {
            widget_style: WidgetStyleJson::from_style(&style.widget_style),

            position_type: style
                .position_type
                .map(PositionTypeJson::from_position_type),

            width: style.width.map(SizeUnitsJson::from_size_units),
            height: style.height.map(SizeUnitsJson::from_size_units),
            width_min: style.width_min.map(SizeUnitsJson::from_size_units),
            width_max: style.width_max.map(SizeUnitsJson::from_size_units),
            height_min: style.height_min.map(SizeUnitsJson::from_size_units),
            height_max: style.height_max.map(SizeUnitsJson::from_size_units),

            margin_left: style.margin_left.map(MarginUnitsJson::from_margin_units),
            margin_right: style.margin_right.map(MarginUnitsJson::from_margin_units),
            margin_top: style.margin_top.map(MarginUnitsJson::from_margin_units),
            margin_bottom: style.margin_bottom.map(MarginUnitsJson::from_margin_units),

            solid_override: style.solid_override.map(SolidJson::from_solid),
            aspect_ratio: style.aspect_ratio(),

            self_halign: style.self_halign.map(AlignmentJson::from_alignment),
            self_valign: style.self_valign.map(AlignmentJson::from_alignment),
        }
    }
}

impl WidgetStyleJson {
    fn from_style(style: &WidgetStyle) -> Self {
        match style {
            WidgetStyle::Panel(panel) => Self::Panel(PanelStyleJson::from_panel_style(panel)),
            WidgetStyle::Text(text) => Self::Text(TextStyleJson::from_text_style(text)),
        }
    }
}

impl PanelStyleJson {
    fn from_panel_style(style: &PanelStyle) -> Self {
        Self {
            background_color: style.background_color.map(ColorJson::from_color),
            background_alpha: style.background_alpha(),

            layout_type: style.layout_type.map(LayoutTypeJson::from_layout_type),

            padding_left: style.padding_left.map(SizeUnitsJson::from_size_units),
            padding_right: style.padding_right.map(SizeUnitsJson::from_size_units),
            padding_top: style.padding_top.map(SizeUnitsJson::from_size_units),
            padding_bottom: style.padding_bottom.map(SizeUnitsJson::from_size_units),

            row_between: style.row_between.map(SizeUnitsJson::from_size_units),
            col_between: style.col_between.map(SizeUnitsJson::from_size_units),
            children_halign: style.children_halign.map(AlignmentJson::from_alignment),
            children_valign: style.children_valign.map(AlignmentJson::from_alignment),
        }
    }
}

impl TextStyleJson {
    fn from_text_style(_style: &TextStyle) -> Self {
        Self {}
    }
}

impl PositionTypeJson {
    fn from_position_type(position_type: PositionType) -> Self {
        match position_type {
            PositionType::Absolute => Self::Absolute,
            PositionType::Relative => Self::Relative,
        }
    }
}

impl SizeUnitsJson {
    fn from_size_units(size_units: SizeUnits) -> Self {
        match size_units {
            SizeUnits::Pixels(pixels) => Self::Pixels(pixels),
            SizeUnits::Percentage(percentage) => Self::Percentage(percentage),
            SizeUnits::Auto => Self::Auto,
        }
    }
}

impl MarginUnitsJson {
    fn from_margin_units(margin_units: MarginUnits) -> Self {
        match margin_units {
            MarginUnits::Pixels(pixels) => Self::Pixels(pixels),
            MarginUnits::Percentage(percentage) => Self::Percentage(percentage),
        }
    }
}

impl SolidJson {
    fn from_solid(solid: Solid) -> Self {
        match solid {
            Solid::Fit => Self::Fit,
            Solid::Fill => Self::Fill,
        }
    }
}

impl AlignmentJson {
    fn from_alignment(alignment: Alignment) -> Self {
        match alignment {
            Alignment::Start => Self::Start,
            Alignment::Center => Self::Center,
            Alignment::End => Self::End,
        }
    }
}

impl LayoutTypeJson {
    fn from_layout_type(layout_type: LayoutType) -> Self {
        match layout_type {
            LayoutType::Row => Self::Row,
            LayoutType::Column => Self::Column,
        }
    }
}

impl ColorJson {
    fn from_color(color: Color) -> Self {
        Self {
            r: color.r,
            g: color.g,
            b: color.b,
        }
    }
}

impl UiNodeJson {
    fn from_node(style_id_to_index: &HashMap<StyleId, usize>, node: &UiNode) -> Self {
        let mut me = Self {
            visible: node.visible,
            style_ids: Vec::new(),
            widget: WidgetJson::from_widget(&node.widget),
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

impl WidgetJson {
    fn from_widget(widget: &Widget) -> Self {
        match widget {
            Widget::Panel(panel) => {
                Self::Panel(PanelJson::from_panel(panel))
            }
            Widget::Text(text) => {
                Self::Text(TextJson::from_text(text))
            }
        }
    }
}

impl PanelJson {
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

impl TextJson {
    fn from_text(text: &Text) -> Self {
        Self {
            text: text.inner_text().to_string(),
        }
    }
}
