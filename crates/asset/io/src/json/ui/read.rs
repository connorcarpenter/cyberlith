use std::collections::HashMap;

use crate::json::ButtonJson;
use asset_id::AssetId;
use render_api::base::Color;
use ui::{
    Alignment, ButtonMut, ButtonStyleMut, LayoutType, MarginUnits, PanelMut, PanelStyleMut,
    PositionType, SizeUnits, Solid, StyleId, TextStyleMut, Ui, WidgetKind,
};

use super::{
    AlignmentJson, ColorJson, LayoutTypeJson, MarginUnitsJson, PanelJson, PositionTypeJson,
    SizeUnitsJson, SolidJson, UiJson, UiNodeJson, UiStyleJson, WidgetJson, WidgetStyleJson,
};

fn convert_nodes_recurse_panel<'a>(
    style_index_to_id: &HashMap<usize, StyleId>,
    nodes: &Vec<UiNodeJson>,
    panel_serde: &PanelJson,
    panel_mut: &'a mut PanelMut<'a>,
) {
    panel_mut.contents(|c| {
        for child_index in &panel_serde.children {
            let child_index = *child_index;
            let child_node_serde = &nodes[child_index];

            //info!("{} - child_node_serde: {:?}", child_index, child_node_serde);

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
                    convert_nodes_recurse_panel(
                        style_index_to_id,
                        nodes,
                        child_panel_serde,
                        &mut child_panel_mut,
                    );
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
                WidgetKind::Button => {
                    let WidgetJson::Button(child_button_serde) = &child_node_serde.widget else {
                        panic!("Expected button widget");
                    };
                    let mut child_button_mut = c.add_button(child_button_serde.id_str.as_str());
                    child_button_mut.set_visible(child_node_serde.visible);
                    for style_index in &child_node_serde.style_ids {
                        let style_id = *style_index_to_id.get(style_index).unwrap();
                        child_button_mut.add_style(style_id);
                    }
                    let WidgetJson::Button(child_button_serde) = &child_node_serde.widget else {
                        panic!("Expected button widget");
                    };
                    convert_nodes_recurse_button(
                        style_index_to_id,
                        nodes,
                        child_button_serde,
                        &mut child_button_mut,
                    );
                }
            }
        }
    });
}

fn convert_nodes_recurse_button<'a>(
    style_index_to_id: &HashMap<usize, StyleId>,
    nodes: &Vec<UiNodeJson>,
    button_serde: &ButtonJson,
    button_mut: &'a mut ButtonMut<'a>,
) {
    button_mut.contents(|c| {
        for child_index in &button_serde.panel.children {
            let child_index = *child_index;
            let child_node_serde = &nodes[child_index];

            //info!("{} - child_node_serde: {:?}", child_index, child_node_serde);

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
                    convert_nodes_recurse_panel(
                        style_index_to_id,
                        nodes,
                        child_panel_serde,
                        &mut child_panel_mut,
                    );
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
                _ => {
                    panic!("Button children can only be panels or text");
                }
            }
        }
    });
}

impl UiJson {
    pub fn to_ui(self) -> Ui {
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
        ui.set_text_icon_asset_id(&text_icon_asset_id);

        // styles
        let mut style_index_to_id = HashMap::new();

        for (style_index, style_serde) in styles.iter().enumerate() {
            //info!("style_serde: {}, {:?}", style_index, style_serde);

            let style_id = match style_serde.widget_kind() {
                WidgetKind::Panel => ui.create_panel_style(|style| {
                    style_serde.to_panel_style(style);
                }),
                WidgetKind::Text => ui.create_text_style(|style| {
                    style_serde.to_text_style(style);
                }),
                WidgetKind::Button => ui.create_button_style(|style| {
                    style_serde.to_button_style(style);
                }),
            };
            style_index_to_id.insert(style_index, style_id);
        }

        // nodes
        let root_node_serde = nodes.first().unwrap();
        //info!("0 - root_node_serde: {:?}", root_node_serde);

        let mut root_mut = ui.root_mut();
        root_mut.set_visible(root_node_serde.visible);
        for style_index in &root_node_serde.style_ids {
            let style_id = *style_index_to_id.get(style_index).unwrap();
            root_mut.add_style(style_id);
        }
        let WidgetJson::Panel(panel_serde) = &root_node_serde.widget else {
            panic!("Expected panel widget");
        };
        convert_nodes_recurse_panel(&style_index_to_id, &nodes, panel_serde, &mut root_mut);

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
        if let Some((w, h)) = self.aspect_ratio {
            style.set_aspect_ratio(w, h);
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
        if let Some((w, h)) = self.aspect_ratio {
            style.set_aspect_ratio(w, h);
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

    fn to_button_style(&self, style: &mut ButtonStyleMut) {
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
        if let Some((w, h)) = self.aspect_ratio {
            style.set_aspect_ratio(w, h);
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
        let WidgetStyleJson::Button(button_style_serde) = &self.widget_style else {
            panic!("Expected button style");
        };
        let panel_style_serde = &button_style_serde.panel;

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

        // button-specific
        if let Some(val_serde) = &button_style_serde.hover_color {
            let val = val_serde.to_color();
            style.set_hover_color(val);
        }

        if let Some(val_serde) = &button_style_serde.down_color {
            let val = val_serde.to_color();
            style.set_down_color(val);
        }
    }
}