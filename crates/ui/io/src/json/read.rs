use std::collections::HashMap;

use log::info;

use asset_id::AssetId;
use asset_render::{AssetHandle, IconData};
use ui_layout::{Alignment, LayoutType, MarginUnits, PositionType, SizeUnits, Solid};
use render_api::base::Color;
use ui::{
    WidgetKind, PanelMut, PanelStyleMut, StyleId,
    TextStyleMut, Ui,
};

use super::{
    UiNodeJson, PanelJson, WidgetJson, UiJson, ColorJson, UiStyleJson, SizeUnitsJson,
    PositionTypeJson, WidgetStyleJson, MarginUnitsJson, SolidJson, AlignmentJson,
    LayoutTypeJson,
};

fn convert_nodes_recurse<'a>(
    style_index_to_id: &HashMap<usize, StyleId>,
    nodes: &Vec<UiNodeJson>,
    panel_serde: &PanelJson,
    panel_mut: &'a mut PanelMut<'a>
) {
    panel_mut.contents(|c| {
        for child_index in &panel_serde.children {
            let child_index = *child_index;
            let child_node_serde = &nodes[child_index];

            info!("{} - child_node_serde: {:?}", child_index, child_node_serde);

            match child_node_serde.widget_kind() {
                WidgetKind::Panel => {
                    let mut child_panel_mut = c.add_panel();
                    child_panel_mut.set_visible(child_node_serde.visible);
                    for style_index in &child_node_serde.style_ids {
                        let style_id = *style_index_to_id.get(style_index).unwrap();
                        child_panel_mut.add_style(style_id);
                    }
                    let WidgetJson::Panel(child_panel_serde) = &child_node_serde.widget else {
                        panic!("Expected panel widget");
                    };
                    convert_nodes_recurse(style_index_to_id, nodes, child_panel_serde, &mut child_panel_mut);
                }
                WidgetKind::Text => {
                    let WidgetJson::Text(child_text_serde) = &child_node_serde.widget else {
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

impl UiJson {

    pub(crate) fn to_ui(self) -> Ui {
        let mut ui = Ui::new();

        // ui_serde -> ui
        let UiJson {
            text_color,
            text_icon_asset_id,
            styles,
            nodes,
        } = self;

        // text color
        ui.set_text_color(text_color.to_color());

        // text icon
        let text_icon_asset_id = AssetId::from_str(&text_icon_asset_id).unwrap();
        let text_icon_asset_handle = AssetHandle::<IconData>::new(text_icon_asset_id);
        ui.set_text_icon_handle(&text_icon_asset_handle);

        let mut style_index_to_id = HashMap::new();

        // styles
        for (style_index, style_serde) in styles.iter().enumerate() {

            info!("style_serde: {}, {:?}", style_index, style_serde);

            let style_id = match style_serde.widget_kind() {
                WidgetKind::Panel => ui.create_panel_style(|style| {
                    style_serde.to_panel_style(style);
                }),
                WidgetKind::Text => ui.create_text_style(|style| {
                    style_serde.to_text_style(style);
                }),
            };
            style_index_to_id.insert(style_index, style_id);
        }

        // nodes
        let root_node_serde = nodes.first().unwrap();
        info!("0 - root_node_serde: {:?}", root_node_serde);

        let mut root_mut = ui.root_mut();
        root_mut.set_visible(root_node_serde.visible);
        for style_index in &root_node_serde.style_ids {
            let style_id = *style_index_to_id.get(style_index).unwrap();
            root_mut.add_style(style_id);
        }
        let WidgetJson::Panel(panel_serde) = &root_node_serde.widget else {
            panic!("Expected panel widget");
        };
        convert_nodes_recurse(&style_index_to_id, &nodes, panel_serde, &mut root_mut);

        ui
    }
}

impl PositionTypeJson {
    fn to_position_type(&self) -> PositionType {
        match self {
            Self::Absolute => PositionType::Absolute,
            Self::Relative => PositionType::Relative,
        }
    }
}

impl SizeUnitsJson {
    fn to_size_units(&self) -> SizeUnits {
        match self {
            Self::Pixels(pixels) => SizeUnits::Pixels(*pixels),
            Self::Percentage(percentage) => SizeUnits::Percentage(*percentage),
            Self::Auto => SizeUnits::Auto,
        }
    }
}

impl MarginUnitsJson {
    fn to_margin_units(&self) -> MarginUnits {
        match self {
            Self::Pixels(pixels) => MarginUnits::Pixels(*pixels),
            Self::Percentage(percentage) => MarginUnits::Percentage(*percentage),
        }
    }
}

impl SolidJson {
    fn to_solid(&self) -> Solid {
        match self {
            Self::Fit => Solid::Fit,
            Self::Fill => Solid::Fill,
        }
    }
}

impl AlignmentJson {
    fn to_alignment(&self) -> Alignment {
        match self {
            Self::Start => Alignment::Start,
            Self::Center => Alignment::Center,
            Self::End => Alignment::End,
        }
    }
}

impl LayoutTypeJson {
    fn to_layout_type(&self) -> LayoutType {
        match self {
            Self::Row => LayoutType::Row,
            Self::Column => LayoutType::Column,
        }
    }
}

impl ColorJson {
    fn to_color(&self) -> Color {
        Color::new(self.r, self.g, self.b)
    }
}

impl UiStyleJson {
    fn to_panel_style(&self, style: &mut PanelStyleMut) {
        // node-specific
        if let Some(position_type_serde) = &self.position_type {
            let position_type = position_type_serde.to_position_type();
            match position_type {
                PositionType::Absolute => style.set_absolute(),
                PositionType::Relative => style.set_relative(),
            };
        }
        if let Some(val_serde) = &self.width {
            let val = val_serde.to_size_units();
            match val {
                SizeUnits::Pixels(pixels) => style.set_width_px(pixels),
                SizeUnits::Percentage(percentage) => style.set_width_pc(percentage),
                SizeUnits::Auto => style.set_width_auto(),
            };
        }
        if let Some(val_serde) = &self.height {
            let val = val_serde.to_size_units();
            match val {
                SizeUnits::Pixels(pixels) => style.set_height_px(pixels),
                SizeUnits::Percentage(percentage) => style.set_height_pc(percentage),
                SizeUnits::Auto => style.set_height_auto(),
            };
        }
        if let Some(val_serde) = &self.width_min {
            let val = val_serde.to_size_units();
            match val {
                SizeUnits::Pixels(pixels) => style.set_width_min_px(pixels),
                SizeUnits::Percentage(percentage) => style.set_width_min_pc(percentage),
                SizeUnits::Auto => style.set_width_min_auto(),
            };
        }
        if let Some(val_serde) = &self.width_max {
            let val = val_serde.to_size_units();
            match val {
                SizeUnits::Pixels(pixels) => style.set_width_max_px(pixels),
                SizeUnits::Percentage(percentage) => style.set_width_max_pc(percentage),
                SizeUnits::Auto => style.set_width_max_auto(),
            };
        }
        if let Some(val_serde) = &self.height_min {
            let val = val_serde.to_size_units();
            match val {
                SizeUnits::Pixels(pixels) => style.set_height_min_px(pixels),
                SizeUnits::Percentage(percentage) => style.set_height_min_pc(percentage),
                SizeUnits::Auto => style.set_height_min_auto(),
            };
        }
        if let Some(val_serde) = &self.height_max {
            let val = val_serde.to_size_units();
            match val {
                SizeUnits::Pixels(pixels) => style.set_height_max_px(pixels),
                SizeUnits::Percentage(percentage) => style.set_height_max_pc(percentage),
                SizeUnits::Auto => style.set_height_max_auto(),
            };
        }
        if let Some(val_serde) = &self.margin_left {
            let val = val_serde.to_margin_units();
            match val {
                MarginUnits::Pixels(pixels) => style.set_margin_left_px(pixels),
                MarginUnits::Percentage(percentage) => style.set_margin_left_pc(percentage),
            };
        }
        if let Some(val_serde) = &self.margin_right {
            let val = val_serde.to_margin_units();
            match val {
                MarginUnits::Pixels(pixels) => style.set_margin_right_px(pixels),
                MarginUnits::Percentage(percentage) => style.set_margin_right_pc(percentage),
            };
        }
        if let Some(val_serde) = &self.margin_top {
            let val = val_serde.to_margin_units();
            match val {
                MarginUnits::Pixels(pixels) => style.set_margin_top_px(pixels),
                MarginUnits::Percentage(percentage) => style.set_margin_top_pc(percentage),
            };
        }
        if let Some(val_serde) = &self.margin_bottom {
            let val = val_serde.to_margin_units();
            match val {
                MarginUnits::Pixels(pixels) => style.set_margin_bottom_px(pixels),
                MarginUnits::Percentage(percentage) => style.set_margin_bottom_pc(percentage),
            };
        }
        if let Some(solid_override_serde) = &self.solid_override {
            let solid_override = solid_override_serde.to_solid();
            match solid_override {
                Solid::Fit => style.set_solid_fit(),
                Solid::Fill => style.set_solid_fill(),
            };
        }
        if let Some(aspect_ratio_w_over_h) = self.aspect_ratio_w_over_h {
            style.set_aspect_ratio(aspect_ratio_w_over_h, 1.0);
        }
        if let Some(val_serde) = &self.self_halign {
            let val = val_serde.to_alignment();
            style.set_self_halign(val);
        }
        if let Some(val_serde) = &self.self_valign {
            let val = val_serde.to_alignment();
            style.set_self_valign(val);
        }

        // panel-specific
        let WidgetStyleJson::Panel(panel_style_serde) = &self.widget_style else {
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

    fn to_text_style(&self, style: &mut TextStyleMut) {
        // node-specific
        if let Some(position_type_serde) = &self.position_type {
            let position_type = position_type_serde.to_position_type();
            match position_type {
                PositionType::Absolute => style.set_absolute(),
                PositionType::Relative => style.set_relative(),
            };
        }
        if let Some(val_serde) = &self.width {
            let val = val_serde.to_size_units();
            match val {
                SizeUnits::Pixels(pixels) => style.set_width_px(pixels),
                SizeUnits::Percentage(percentage) => style.set_width_pc(percentage),
                SizeUnits::Auto => style.set_width_auto(),
            };
        }
        if let Some(val_serde) = &self.height {
            let val = val_serde.to_size_units();
            match val {
                SizeUnits::Pixels(pixels) => style.set_height_px(pixels),
                SizeUnits::Percentage(percentage) => style.set_height_pc(percentage),
                SizeUnits::Auto => style.set_height_auto(),
            };
        }
        if let Some(val_serde) = &self.width_min {
            let val = val_serde.to_size_units();
            match val {
                SizeUnits::Pixels(pixels) => style.set_width_min_px(pixels),
                SizeUnits::Percentage(percentage) => style.set_width_min_pc(percentage),
                SizeUnits::Auto => style.set_width_min_auto(),
            };
        }
        if let Some(val_serde) = &self.width_max {
            let val = val_serde.to_size_units();
            match val {
                SizeUnits::Pixels(pixels) => style.set_width_max_px(pixels),
                SizeUnits::Percentage(percentage) => style.set_width_max_pc(percentage),
                SizeUnits::Auto => style.set_width_max_auto(),
            };
        }
        if let Some(val_serde) = &self.height_min {
            let val = val_serde.to_size_units();
            match val {
                SizeUnits::Pixels(pixels) => style.set_height_min_px(pixels),
                SizeUnits::Percentage(percentage) => style.set_height_min_pc(percentage),
                SizeUnits::Auto => style.set_height_min_auto(),
            };
        }
        if let Some(val_serde) = &self.height_max {
            let val = val_serde.to_size_units();
            match val {
                SizeUnits::Pixels(pixels) => style.set_height_max_px(pixels),
                SizeUnits::Percentage(percentage) => style.set_height_max_pc(percentage),
                SizeUnits::Auto => style.set_height_max_auto(),
            };
        }
        if let Some(val_serde) = &self.margin_left {
            let val = val_serde.to_margin_units();
            match val {
                MarginUnits::Pixels(pixels) => style.set_margin_left_px(pixels),
                MarginUnits::Percentage(percentage) => style.set_margin_left_pc(percentage),
            };
        }
        if let Some(val_serde) = &self.margin_right {
            let val = val_serde.to_margin_units();
            match val {
                MarginUnits::Pixels(pixels) => style.set_margin_right_px(pixels),
                MarginUnits::Percentage(percentage) => style.set_margin_right_pc(percentage),
            };
        }
        if let Some(val_serde) = &self.margin_top {
            let val = val_serde.to_margin_units();
            match val {
                MarginUnits::Pixels(pixels) => style.set_margin_top_px(pixels),
                MarginUnits::Percentage(percentage) => style.set_margin_top_pc(percentage),
            };
        }
        if let Some(val_serde) = &self.margin_bottom {
            let val = val_serde.to_margin_units();
            match val {
                MarginUnits::Pixels(pixels) => style.set_margin_bottom_px(pixels),
                MarginUnits::Percentage(percentage) => style.set_margin_bottom_pc(percentage),
            };
        }
        if let Some(solid_override_serde) = &self.solid_override {
            let solid_override = solid_override_serde.to_solid();
            match solid_override {
                Solid::Fit => style.set_solid_fit(),
                Solid::Fill => style.set_solid_fill(),
            };
        }
        if let Some(aspect_ratio_w_over_h) = self.aspect_ratio_w_over_h {
            style.set_aspect_ratio(aspect_ratio_w_over_h, 1.0);
        }
        if let Some(val_serde) = &self.self_halign {
            let val = val_serde.to_alignment();
            style.set_self_halign(val);
        }
        if let Some(val_serde) = &self.self_valign {
            let val = val_serde.to_alignment();
            style.set_self_valign(val);
        }
    }
}