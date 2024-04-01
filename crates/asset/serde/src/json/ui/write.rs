use std::collections::HashMap;

use render_api::base::Color;

use ui_layout::{Alignment, LayoutType, MarginUnits, PositionType, SizeUnits, Solid};
use ui_types::{Button, Navigation, ButtonStyle, NodeStyle, Panel, PanelStyle, StyleId, Text, TextStyle, UiConfig, UiNode, Widget, WidgetStyle, TextboxStyle, Textbox};

use crate::json::{ButtonJson, NavigationJson, ButtonStyleJson, TextboxStyleJson, TextboxJson};
use super::{
    AlignmentJson, ColorJson, LayoutTypeJson, MarginUnitsJson, PanelJson, PanelStyleJson,
    PositionTypeJson, SizeUnitsJson, SolidJson, TextJson, TextStyleJson, UiConfigJson, UiNodeJson,
    UiStyleJson, WidgetJson, WidgetStyleJson,
};

// conversion

impl From<&UiConfig> for UiConfigJson {
    fn from(ui_config: &UiConfig) -> Self {
        let mut style_id_to_index = HashMap::new();

        let text_color = From::from(ui_config.get_text_color());
        let text_icon_asset_id = ui_config.get_text_icon_asset_id().to_string();

        let mut me = Self {
            text_color,
            text_icon_asset_id,
            first_input: ui_config.get_first_input().map(|id| id.as_usize()),
            styles: Vec::new(),
            nodes: Vec::new(),
        };

        // styles
        for (style_id, style) in ui_config.store.styles_iter().enumerate() {
            let style_id = StyleId::new(style_id as u32);
            let next_index = me.styles.len();
            style_id_to_index.insert(style_id, next_index);
            me.styles.push(From::from(style));
        }

        // nodes
        for node in ui_config.store.nodes_iter() {
            me.nodes
                .push(UiNodeJson::from_node(&style_id_to_index, node));
        }

        me
    }
}

impl From<&NodeStyle> for UiStyleJson {
    fn from(style: &NodeStyle) -> Self {
        Self {
            parent_style:   style.parent_style.map(|id| id.as_usize()),
            widget_style:   From::from(&style.base.widget_style),

            position_type:  style.base.position_type.map(From::from),

            width:          style.base.width.map(From::from),
            height:         style.base.height.map(From::from),
            width_min:      style.base.width_min.map(From::from),
            width_max:      style.base.width_max.map(From::from),
            height_min:     style.base.height_min.map(From::from),
            height_max:     style.base.height_max.map(From::from),

            margin_left:    style.base.margin_left.map(From::from),
            margin_right:   style.base.margin_right.map(From::from),
            margin_top:     style.base.margin_top.map(From::from),
            margin_bottom:  style.base.margin_bottom.map(From::from),

            solid_override: style.base.solid_override.map(From::from),
            aspect_ratio:   style.aspect_ratio(),

            self_halign:    style.base.self_halign.map(From::from),
            self_valign:    style.base.self_valign.map(From::from),
        }
    }
}

impl From<&WidgetStyle> for WidgetStyleJson {
    fn from(style: &WidgetStyle) -> Self {
        match style {
            WidgetStyle::Panel(panel) =>        Self::Panel(From::from(panel)),
            WidgetStyle::Text(text) =>           Self::Text(From::from(text)),
            WidgetStyle::Button(button) =>      Self::Button(From::from(button)),
            WidgetStyle::Textbox(textbox) =>   Self::Textbox(From::from(textbox)),
        }
    }
}

impl From<&PanelStyle> for PanelStyleJson {
    fn from(style: &PanelStyle) -> Self {
        Self {
            background_color:   style.background_color.map(From::from),
            background_alpha:   style.background_alpha(),

            layout_type:        style.layout_type.map(From::from),

            padding_left:       style.padding_left.map(From::from),
            padding_right:      style.padding_right.map(From::from),
            padding_top:        style.padding_top.map(From::from),
            padding_bottom:     style.padding_bottom.map(From::from),

            row_between:        style.row_between.map(From::from),
            col_between:        style.col_between.map(From::from),
            children_halign:    style.children_halign.map(From::from),
            children_valign:    style.children_valign.map(From::from),
        }
    }
}

impl From<&TextStyle> for TextStyleJson {
    fn from(style: &TextStyle) -> Self {
        Self {
            background_color:   style.background_color.map(From::from),
            background_alpha:   style.background_alpha(),
        }
    }
}

impl From<&ButtonStyle> for ButtonStyleJson {
    fn from(style: &ButtonStyle) -> Self {
        Self {
            panel:          From::from(&style.panel),
            hover_color:    style.hover_color.map(From::from),
            down_color:     style.down_color.map(From::from),
        }
    }
}

impl From<&TextboxStyle> for TextboxStyleJson {
    fn from(style: &TextboxStyle) -> Self {
        Self {
            background_color:   style.background_color.map(From::from),
            background_alpha:   style.background_alpha(),
            hover_color:    style.hover_color.map(From::from),
            active_color:   style.active_color.map(From::from),
            select_color:   style.select_color.map(From::from),
        }
    }
}

impl From<PositionType> for PositionTypeJson {
    fn from(position_type: PositionType) -> Self {
        match position_type {
            PositionType::Absolute => Self::Absolute,
            PositionType::Relative => Self::Relative,
        }
    }
}

impl From<SizeUnits> for SizeUnitsJson {
    fn from(size_units: SizeUnits) -> Self {
        match size_units {
            SizeUnits::Pixels(pixels) =>         Self::Pixels(pixels),
            SizeUnits::Percentage(percentage) => Self::Percentage(percentage),
            SizeUnits::Viewport(percentage) =>   Self::Viewport(percentage),
            SizeUnits::Auto =>                        Self::Auto,
        }
    }
}

impl From<MarginUnits> for MarginUnitsJson {
    fn from(margin_units: MarginUnits) -> Self {
        match margin_units {
            MarginUnits::Pixels(pixels) =>          Self::Pixels(pixels),
            MarginUnits::Percentage(percentage) =>  Self::Percentage(percentage),
            MarginUnits::Viewport(percentage) =>    Self::Viewport(percentage),
        }
    }
}

impl From<Solid> for SolidJson {
    fn from(solid: Solid) -> Self {
        match solid {
            Solid::Fit =>  Self::Fit,
            Solid::Fill => Self::Fill,
        }
    }
}

impl From<Alignment> for AlignmentJson {
    fn from(alignment: Alignment) -> Self {
        match alignment {
            Alignment::Start =>  Self::Start,
            Alignment::Center => Self::Center,
            Alignment::End =>    Self::End,
        }
    }
}

impl From<LayoutType> for LayoutTypeJson {
    fn from(layout_type: LayoutType) -> Self {
        match layout_type {
            LayoutType::Row =>    Self::Row,
            LayoutType::Column => Self::Column,
        }
    }
}

impl From<Color> for ColorJson {
    fn from(color: Color) -> Self {
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
            style_id: None,
            widget: From::from(&node.widget),
        };

        if let Some(style_id) = node.style_id() {
            let style_index: usize = *style_id_to_index.get(&style_id).unwrap();
            me.style_id = Some(style_index);
        }

        me
    }
}

impl From<&Widget> for WidgetJson {
    fn from(widget: &Widget) -> Self {
        match widget {
            Widget::Panel(panel) =>        Self::Panel(From::from(panel)),
            Widget::Text(text) =>           Self::Text(From::from(text)),
            Widget::Button(button) =>     Self::Button(From::from(button)),
            Widget::Textbox(textbox) =>  Self::Textbox(From::from(textbox)),
        }
    }
}

impl From<&Panel> for PanelJson {
    fn from(panel: &Panel) -> Self {
        let mut me = Self {
            children: Vec::new(),
        };
        for child_id in panel.children.iter() {
            me.children.push(child_id.as_usize());
        }
        me
    }
}

impl From<&Text> for TextJson {
    fn from(text: &Text) -> Self {
        Self {
            text: text.inner_text().to_string(),
        }
    }
}

impl From<&Button> for ButtonJson {
    fn from(button: &Button) -> Self {
        let panel_json = From::from(&button.panel);

        Self {
            panel:      panel_json,
            id_str:     button.id_str.to_string(),
            navigation: From::from(&button.navigation),
        }
    }
}

impl From<&Textbox> for TextboxJson {
    fn from(textbox: &Textbox) -> Self {
        Self {
            id_str:     textbox.id_str.to_string(),
            navigation: From::from(&textbox.navigation),
        }
    }
}

impl From<&Navigation> for NavigationJson {
    fn from(navigation: &Navigation) -> Self {
        Self {
            up: navigation.up_goes_to.clone(),
            down: navigation.down_goes_to.clone(),
            left: navigation.left_goes_to.clone(),
            right: navigation.right_goes_to.clone(),
            tab: navigation.tab_goes_to.clone(),
        }
    }
}