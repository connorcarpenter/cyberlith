use std::collections::HashMap;

use naia_serde::{BitReader, SerdeErr, SerdeInternal as Serde, UnsignedVariableInteger};

use asset_id::AssetId;
use render_api::base::Color;
use ui::{Alignment, ButtonMut, ButtonStyleMut, LayoutType, MarginUnits, NodeId, PanelMut, PanelStyleMut, PositionType, SizeUnits, Solid, StyleId, TextboxMut, TextboxStyleMut, TextStyleMut, Ui, WidgetKind};

use crate::bits::{AlignmentBits, ButtonBits, LayoutTypeBits, MarginUnitsBits, PanelBits, PositionTypeBits, SizeUnitsBits, SolidBits, TextboxBits, UiAction, UiActionType, UiNodeBits, UiStyleBits, WidgetBits, WidgetStyleBits};

pub fn read_bits(data: &[u8]) -> Ui {
    let actions = bytes_to_actions(data).unwrap();
    convert_actions_to_ui(actions)
}

fn convert_actions_to_ui(actions: Vec<UiAction>) -> Ui {
    let mut ui = Ui::new();

    let mut style_index_to_id: HashMap<usize, StyleId> = HashMap::new();
    let mut style_count = 0;

    let mut nodes = Vec::new();

    for action in actions {
        match action {
            UiAction::TextColor(r, g, b) => {
                let color = Color::new(r, g, b);
                ui.set_text_color(color);
            }
            UiAction::TextIconAssetId(asset_id) => {
                ui.set_text_icon_asset_id(&asset_id);
            }
            UiAction::DefaultButton(node_id_opt) => {
                if let Some(node_id) = node_id_opt {
                    ui.set_default_button(node_id)
                }
            }
            UiAction::Style(style_bits) => {
                let style_id = match style_bits.widget_kind() {
                    WidgetKind::Panel => ui.create_panel_style(|style| {
                        style_bits.to_panel_style(style);
                    }),
                    WidgetKind::Text => ui.create_text_style(|style| {
                        style_bits.to_text_style(style);
                    }),
                    WidgetKind::Button => ui.create_button_style(|style| {
                        style_bits.to_button_style(style);
                    }),
                    WidgetKind::Textbox => ui.create_textbox_style(|style| {
                        style_bits.to_textbox_style(style);
                    }),
                };
                style_index_to_id.insert(style_count, style_id);
                style_count += 1;
            }
            UiAction::Node(node) => {
                nodes.push(node);
            }
        }
    }

    // process nodes recursively
    let root_node_serde = nodes.first().unwrap();
    //info!("0 - root_node_serde: {:?}", root_node_serde);

    let mut root_mut = ui.root_mut();
    root_mut.set_visible(root_node_serde.visible);
    for style_index in &root_node_serde.style_ids {
        let style_index = *style_index as usize;
        let style_id = *style_index_to_id.get(&style_index).unwrap();
        root_mut.add_style(style_id);
    }
    let WidgetBits::Panel(panel_serde) = &root_node_serde.widget else {
        panic!("Expected panel widget");
    };
    convert_nodes_recurse_panel(&style_index_to_id, &nodes, panel_serde, &mut root_mut);

    ui
}

fn bytes_to_actions(data: &[u8]) -> Result<Vec<UiAction>, SerdeErr> {
    let mut bit_reader = BitReader::new(data);
    let bit_reader = &mut bit_reader;
    let mut actions = Vec::new();

    loop {
        let action_type = UiActionType::de(bit_reader)?;

        match action_type {
            UiActionType::TextColor => {
                let r = u8::de(bit_reader)?;
                let g = u8::de(bit_reader)?;
                let b = u8::de(bit_reader)?;
                actions.push(UiAction::TextColor(r, g, b));
            }
            UiActionType::TextIconAssetId => {
                let val = u32::de(bit_reader)?;
                let asset_id = AssetId::from_u32(val).unwrap();
                actions.push(UiAction::TextIconAssetId(asset_id));
            }
            UiActionType::DefaultButton => {
                let val = Option::<UnsignedVariableInteger<7>>::de(bit_reader)?;
                let val: Option<u64> = val.map(|v| v.to());
                let node_id_opt = val.map(|v| NodeId::from_usize(v as usize));
                actions.push(UiAction::DefaultButton(node_id_opt));
            }
            UiActionType::Style => {
                let style = UiStyleBits::de(bit_reader)?;
                actions.push(UiAction::Style(style));
            }
            UiActionType::Node => {
                let node = UiNodeBits::de(bit_reader)?;
                actions.push(UiAction::Node(node));
            }
            UiActionType::None => {
                break;
            }
        }
    }

    Ok(actions)
}

fn convert_nodes_recurse_panel<'a>(
    style_index_to_id: &HashMap<usize, StyleId>,
    nodes: &Vec<UiNodeBits>,
    panel_serde: &PanelBits,
    panel_mut: &'a mut PanelMut<'a>,
) {
    panel_mut.contents(|c| {
        for child_index in &panel_serde.children {
            let child_index = *child_index as usize;
            let child_node_serde = &nodes[child_index];

            //info!("{} - child_node_serde: {:?}", child_index, child_node_serde);

            match child_node_serde.widget_kind() {
                WidgetKind::Panel => {
                    let mut child_panel_mut = c.add_panel();
                    child_panel_mut.set_visible(child_node_serde.visible);
                    for style_index in &child_node_serde.style_ids {
                        let style_index = *style_index as usize;
                        let style_id = *style_index_to_id.get(&style_index).unwrap();
                        child_panel_mut.add_style(style_id);
                    }
                    let WidgetBits::Panel(child_panel_serde) = &child_node_serde.widget else {
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
                    let WidgetBits::Text(child_text_serde) = &child_node_serde.widget else {
                        panic!("Expected text widget");
                    };
                    let mut child_text_mut = c.add_text(child_text_serde.text.as_str());
                    child_text_mut.set_visible(child_node_serde.visible);
                    for style_index in &child_node_serde.style_ids {
                        let style_index = *style_index as usize;
                        let style_id = *style_index_to_id.get(&style_index).unwrap();
                        child_text_mut.add_style(style_id);
                    }
                }
                WidgetKind::Button => {
                    let WidgetBits::Button(child_button_serde) = &child_node_serde.widget else {
                        panic!("Expected button widget");
                    };
                    let mut child_button_mut = c.add_button(child_button_serde.id_str.as_str());
                    child_button_mut.set_visible(child_node_serde.visible);
                    for style_index in &child_node_serde.style_ids {
                        let style_index = *style_index as usize;
                        let style_id = *style_index_to_id.get(&style_index).unwrap();
                        child_button_mut.add_style(style_id);
                    }
                    convert_nodes_recurse_button(
                        style_index_to_id,
                        nodes,
                        child_button_serde,
                        &mut child_button_mut,
                    );
                }
                WidgetKind::Textbox => {
                    let WidgetBits::Textbox(child_textbox_serde) = &child_node_serde.widget else {
                        panic!("Expected textbox widget");
                    };
                    let mut child_textbox_mut = c.add_textbox(child_textbox_serde.id_str.as_str());
                    child_textbox_mut.set_visible(child_node_serde.visible);
                    for style_index in &child_node_serde.style_ids {
                        let style_index = *style_index as usize;
                        let style_id = *style_index_to_id.get(&style_index).unwrap();
                        child_textbox_mut.add_style(style_id);
                    }
                    set_textbox_navigation(
                        nodes,
                        child_textbox_serde,
                        &mut child_textbox_mut,
                    );
                }
            }
        }
    });
}

fn get_nav_thang<'a>(
    nodes: &'a Vec<UiNodeBits>,
    input_int: Option<&UnsignedVariableInteger<4>>
) -> Option<&'a str> {
    let input_int = input_int?;
    let nav_index = input_int.to::<u32>() as usize;
    let nav_node_serde = &nodes[nav_index];
    let WidgetBits::Button(button_bits) = &nav_node_serde.widget else {
        panic!("Expected button widget");
    };
    let nav_str = button_bits.id_str.as_str();
    Some(nav_str)
}

fn convert_nodes_recurse_button<'a>(
    style_index_to_id: &HashMap<usize, StyleId>,
    nodes: &Vec<UiNodeBits>,
    button_serde: &ButtonBits,
    button_mut: &'a mut ButtonMut<'a>,
) {
    let button_nav_serde = &button_serde.navigation;
    button_mut
        .navigation(|n| {
            if let Some(nav_str) = get_nav_thang(nodes, button_nav_serde.up.as_ref()) {
                n.up_goes_to(nav_str);
            }
            if let Some(nav_str) = get_nav_thang(nodes, button_nav_serde.down.as_ref()) {
                n.down_goes_to(nav_str);
            }
            if let Some(nav_str) = get_nav_thang(nodes, button_nav_serde.left.as_ref()) {
                n.left_goes_to(nav_str);
            }
            if let Some(nav_str) = get_nav_thang(nodes, button_nav_serde.right.as_ref()) {
                n.right_goes_to(nav_str);
            }
        })
        .contents(|c| {
        for child_index in &button_serde.panel.children {
            let child_index = *child_index as usize;
            let child_node_serde = &nodes[child_index];

            //info!("{} - child_node_serde: {:?}", child_index, child_node_serde);

            match child_node_serde.widget_kind() {
                WidgetKind::Panel => {
                    let mut child_panel_mut = c.add_panel();
                    child_panel_mut.set_visible(child_node_serde.visible);
                    for style_index in &child_node_serde.style_ids {
                        let style_index = *style_index as usize;
                        let style_id = *style_index_to_id.get(&style_index).unwrap();
                        child_panel_mut.add_style(style_id);
                    }
                    let WidgetBits::Panel(child_panel_serde) = &child_node_serde.widget else {
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
                    let WidgetBits::Text(child_text_serde) = &child_node_serde.widget else {
                        panic!("Expected text widget");
                    };
                    let mut child_text_mut = c.add_text(child_text_serde.text.as_str());
                    child_text_mut.set_visible(child_node_serde.visible);
                    for style_index in &child_node_serde.style_ids {
                        let style_index = *style_index as usize;
                        let style_id = *style_index_to_id.get(&style_index).unwrap();
                        child_text_mut.add_style(style_id);
                    }
                }
                _ => {
                    panic!("Button can only contain Panel or Text");
                }
            }
        }
    });
}

fn set_textbox_navigation<'a>(
    nodes: &Vec<UiNodeBits>,
    textbox_serde: &TextboxBits,
    textbox_mut: &'a mut TextboxMut<'a>,
) {
    let textbox_nav_serde = &textbox_serde.navigation;
    textbox_mut
        .navigation(|n| {
            if let Some(nav_str) = get_nav_thang(nodes, textbox_nav_serde.up.as_ref()) {
                n.up_goes_to(nav_str);
            }
            if let Some(nav_str) = get_nav_thang(nodes, textbox_nav_serde.down.as_ref()) {
                n.down_goes_to(nav_str);
            }
            if let Some(nav_str) = get_nav_thang(nodes, textbox_nav_serde.left.as_ref()) {
                n.left_goes_to(nav_str);
            }
            if let Some(nav_str) = get_nav_thang(nodes, textbox_nav_serde.right.as_ref()) {
                n.right_goes_to(nav_str);
            }
        });
}

impl PositionTypeBits {
    fn to_position_type(&self) -> PositionType {
        match self {
            Self::Absolute => PositionType::Absolute,
            Self::Relative => PositionType::Relative,
        }
    }
}

impl SizeUnitsBits {
    fn to_size_units(&self) -> SizeUnits {
        match self {
            Self::Pixels(val) => {
                let val: u64 = val.to();
                let val: f32 = val as f32;
                SizeUnits::Pixels(val)
            }
            Self::Percent(val) => {
                let val: u64 = val.to();
                let val: f32 = val as f32;
                SizeUnits::Percentage(val)
            }
            Self::Viewport(val) => {
                let val: u64 = val.to();
                let val: f32 = val as f32;
                SizeUnits::Viewport(val)
            }
            Self::Auto => SizeUnits::Auto,
        }
    }
}

impl MarginUnitsBits {
    fn to_margin_units(&self) -> MarginUnits {
        match self {
            Self::Pixels(val) => {
                let val: u64 = val.to();
                let val: f32 = val as f32;
                MarginUnits::Pixels(val)
            }
            Self::Percent(val) => {
                let val: u64 = val.to();
                let val: f32 = val as f32;
                MarginUnits::Percentage(val)
            }
            Self::Viewport(val) => {
                let val: u64 = val.to();
                let val: f32 = val as f32;
                MarginUnits::Viewport(val)
            }
        }
    }
}

impl SolidBits {
    fn to_solid(&self) -> Solid {
        match self {
            Self::Fit => Solid::Fit,
            Self::Fill => Solid::Fill,
        }
    }
}

impl AlignmentBits {
    fn to_alignment(&self) -> Alignment {
        match self {
            Self::Start => Alignment::Start,
            Self::Center => Alignment::Center,
            Self::End => Alignment::End,
        }
    }
}

impl LayoutTypeBits {
    fn to_layout_type(&self) -> LayoutType {
        match self {
            Self::Row => LayoutType::Row,
            Self::Column => LayoutType::Column,
        }
    }
}

impl UiStyleBits {
    fn to_panel_style(&self, style: &mut PanelStyleMut) {
        // panel-specific
        let WidgetStyleBits::Panel(panel_style_serde) = &self.widget_style else {
            panic!("Expected panel style");
        };

        if let Some((r, g, b)) = &panel_style_serde.background_color {
            style.set_background_color(Color::new(*r, *g, *b));
        }
        if let Some(background_alpha) = panel_style_serde.background_alpha {
            let val: u8 = background_alpha.to();
            let val: f32 = val as f32;
            let val = val / 10.0;
            style.set_background_alpha(val);
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
                SizeUnits::Viewport(percentage) => style.set_padding_left_vp(percentage),
                SizeUnits::Auto => style.set_padding_left_auto(),
            };
        }
        if let Some(val_serde) = &panel_style_serde.padding_right {
            let val = val_serde.to_size_units();
            match val {
                SizeUnits::Pixels(pixels) => style.set_padding_right_px(pixels),
                SizeUnits::Percentage(percentage) => style.set_padding_right_pc(percentage),
                SizeUnits::Viewport(percentage) => style.set_padding_right_vp(percentage),
                SizeUnits::Auto => style.set_padding_right_auto(),
            };
        }
        if let Some(val_serde) = &panel_style_serde.padding_top {
            let val = val_serde.to_size_units();
            match val {
                SizeUnits::Pixels(pixels) => style.set_padding_top_px(pixels),
                SizeUnits::Percentage(percentage) => style.set_padding_top_pc(percentage),
                SizeUnits::Viewport(percentage) => style.set_padding_top_vp(percentage),
                SizeUnits::Auto => style.set_padding_top_auto(),
            };
        }
        if let Some(val_serde) = &panel_style_serde.padding_bottom {
            let val = val_serde.to_size_units();
            match val {
                SizeUnits::Pixels(pixels) => style.set_padding_bottom_px(pixels),
                SizeUnits::Percentage(percentage) => style.set_padding_bottom_pc(percentage),
                SizeUnits::Viewport(percentage) => style.set_padding_bottom_vp(percentage),
                SizeUnits::Auto => style.set_padding_bottom_auto(),
            };
        }
        if let Some(val_serde) = &panel_style_serde.row_between {
            let val = val_serde.to_size_units();
            match val {
                SizeUnits::Pixels(pixels) => style.set_row_between_px(pixels),
                SizeUnits::Percentage(percentage) => style.set_row_between_pc(percentage),
                SizeUnits::Viewport(percentage) => style.set_row_between_vp(percentage),
                SizeUnits::Auto => style.set_row_between_auto(),
            };
        }
        if let Some(val_serde) = &panel_style_serde.col_between {
            let val = val_serde.to_size_units();
            match val {
                SizeUnits::Pixels(pixels) => style.set_col_between_px(pixels),
                SizeUnits::Percentage(percentage) => style.set_col_between_pc(percentage),
                SizeUnits::Viewport(percentage) => style.set_col_between_vp(percentage),
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
                SizeUnits::Viewport(percentage) => style.set_width_vp(percentage),
                SizeUnits::Auto => style.set_width_auto(),
            };
        }
        if let Some(val_serde) = &self.height {
            let val = val_serde.to_size_units();
            match val {
                SizeUnits::Pixels(pixels) => style.set_height_px(pixels),
                SizeUnits::Percentage(percentage) => style.set_height_pc(percentage),
                SizeUnits::Viewport(percentage) => style.set_height_vp(percentage),
                SizeUnits::Auto => style.set_height_auto(),
            };
        }
        if let Some(val_serde) = &self.width_min {
            let val = val_serde.to_size_units();
            match val {
                SizeUnits::Pixels(pixels) => style.set_width_min_px(pixels),
                SizeUnits::Percentage(percentage) => style.set_width_min_pc(percentage),
                SizeUnits::Viewport(percentage) => style.set_width_min_vp(percentage),
                SizeUnits::Auto => style.set_width_min_auto(),
            };
        }
        if let Some(val_serde) = &self.width_max {
            let val = val_serde.to_size_units();
            match val {
                SizeUnits::Pixels(pixels) => style.set_width_max_px(pixels),
                SizeUnits::Percentage(percentage) => style.set_width_max_pc(percentage),
                SizeUnits::Viewport(percentage) => style.set_width_max_vp(percentage),
                SizeUnits::Auto => style.set_width_max_auto(),
            };
        }
        if let Some(val_serde) = &self.height_min {
            let val = val_serde.to_size_units();
            match val {
                SizeUnits::Pixels(pixels) => style.set_height_min_px(pixels),
                SizeUnits::Percentage(percentage) => style.set_height_min_pc(percentage),
                SizeUnits::Viewport(percentage) => style.set_height_min_vp(percentage),
                SizeUnits::Auto => style.set_height_min_auto(),
            };
        }
        if let Some(val_serde) = &self.height_max {
            let val = val_serde.to_size_units();
            match val {
                SizeUnits::Pixels(pixels) => style.set_height_max_px(pixels),
                SizeUnits::Percentage(percentage) => style.set_height_max_pc(percentage),
                SizeUnits::Viewport(percentage) => style.set_height_max_vp(percentage),
                SizeUnits::Auto => style.set_height_max_auto(),
            };
        }
        if let Some(val_serde) = &self.margin_left {
            let val = val_serde.to_margin_units();
            match val {
                MarginUnits::Pixels(pixels) => style.set_margin_left_px(pixels),
                MarginUnits::Percentage(percentage) => style.set_margin_left_pc(percentage),
                MarginUnits::Viewport(percentage) => style.set_margin_left_vp(percentage),
            };
        }
        if let Some(val_serde) = &self.margin_right {
            let val = val_serde.to_margin_units();
            match val {
                MarginUnits::Pixels(pixels) => style.set_margin_right_px(pixels),
                MarginUnits::Percentage(percentage) => style.set_margin_right_pc(percentage),
                MarginUnits::Viewport(percentage) => style.set_margin_right_vp(percentage),
            };
        }
        if let Some(val_serde) = &self.margin_top {
            let val = val_serde.to_margin_units();
            match val {
                MarginUnits::Pixels(pixels) => style.set_margin_top_px(pixels),
                MarginUnits::Percentage(percentage) => style.set_margin_top_pc(percentage),
                MarginUnits::Viewport(percentage) => style.set_margin_top_vp(percentage),
            };
        }
        if let Some(val_serde) = &self.margin_bottom {
            let val = val_serde.to_margin_units();
            match val {
                MarginUnits::Pixels(pixels) => style.set_margin_bottom_px(pixels),
                MarginUnits::Percentage(percentage) => style.set_margin_bottom_pc(percentage),
                MarginUnits::Viewport(percentage) => style.set_margin_bottom_vp(percentage),
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
            style.set_aspect_ratio(w as f32, h as f32);
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

    fn to_text_style(&self, style: &mut TextStyleMut) {
        // text-specific
        let WidgetStyleBits::Text(text_style_serde) = &self.widget_style else {
            panic!("Expected text style");
        };

        if let Some((r, g, b)) = &text_style_serde.background_color {
            style.set_background_color(Color::new(*r, *g, *b));
        }
        if let Some(background_alpha) = text_style_serde.background_alpha {
            let val: u8 = background_alpha.to();
            let val: f32 = val as f32;
            let val = val / 10.0;
            style.set_background_alpha(val);
        }
        // node-specific
        if let Some(position_type_serde) = &self.position_type {
            let position_type = position_type_serde.to_position_type();
            match position_type {
                PositionType::Absolute => style.set_absolute(),
                PositionType::Relative => style.set_relative(),
            };
        }
        if let Some(val_serde) = &self.height {
            let val = val_serde.to_size_units();
            match val {
                SizeUnits::Pixels(pixels) => style.set_size_px(pixels),
                SizeUnits::Percentage(percentage) => style.set_size_pc(percentage),
                SizeUnits::Viewport(percentage) => style.set_size_vp(percentage),
                SizeUnits::Auto => panic!("unsupported"),
            };
        }
        if let Some(val_serde) = &self.height_min {
            let val = val_serde.to_size_units();
            match val {
                SizeUnits::Pixels(pixels) => style.set_size_min_px(pixels),
                SizeUnits::Percentage(percentage) => style.set_size_min_pc(percentage),
                SizeUnits::Viewport(percentage) => style.set_size_min_vp(percentage),
                SizeUnits::Auto => panic!("unsupported"),
            };
        }
        if let Some(val_serde) = &self.height_max {
            let val = val_serde.to_size_units();
            match val {
                SizeUnits::Pixels(pixels) => style.set_size_max_px(pixels),
                SizeUnits::Percentage(percentage) => style.set_size_max_pc(percentage),
                SizeUnits::Viewport(percentage) => style.set_size_max_vp(percentage),
                SizeUnits::Auto => panic!("unsupported"),
            };
        }
        if let Some(val_serde) = &self.margin_left {
            let val = val_serde.to_margin_units();
            match val {
                MarginUnits::Pixels(pixels) => style.set_margin_left_px(pixels),
                MarginUnits::Percentage(percentage) => style.set_margin_left_pc(percentage),
                MarginUnits::Viewport(percentage) => style.set_margin_left_vp(percentage),
            };
        }
        if let Some(val_serde) = &self.margin_right {
            let val = val_serde.to_margin_units();
            match val {
                MarginUnits::Pixels(pixels) => style.set_margin_right_px(pixels),
                MarginUnits::Percentage(percentage) => style.set_margin_right_pc(percentage),
                MarginUnits::Viewport(percentage) => style.set_margin_right_vp(percentage),
            };
        }
        if let Some(val_serde) = &self.margin_top {
            let val = val_serde.to_margin_units();
            match val {
                MarginUnits::Pixels(pixels) => style.set_margin_top_px(pixels),
                MarginUnits::Percentage(percentage) => style.set_margin_top_pc(percentage),
                MarginUnits::Viewport(percentage) => style.set_margin_top_vp(percentage),
            };
        }
        if let Some(val_serde) = &self.margin_bottom {
            let val = val_serde.to_margin_units();
            match val {
                MarginUnits::Pixels(pixels) => style.set_margin_bottom_px(pixels),
                MarginUnits::Percentage(percentage) => style.set_margin_bottom_pc(percentage),
                MarginUnits::Viewport(percentage) => style.set_margin_bottom_vp(percentage),
            };
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

        // button-specific
        let WidgetStyleBits::Button(button_style_serde) = &self.widget_style else {
            panic!("Expected panel style");
        };

        if let Some((r, g, b)) = &button_style_serde.hover_color {
            style.set_hover_color(Color::new(*r, *g, *b));
        }

        if let Some((r, g, b)) = &button_style_serde.down_color {
            style.set_down_color(Color::new(*r, *g, *b));
        }

        // panel-specific
        let panel_style_serde = &button_style_serde.panel;

        if let Some((r, g, b)) = &panel_style_serde.background_color {
            style.set_background_color(Color::new(*r, *g, *b));
        }
        if let Some(background_alpha) = panel_style_serde.background_alpha {
            let val: u8 = background_alpha.to();
            let val: f32 = val as f32;
            let val = val / 10.0;
            style.set_background_alpha(val);
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
                SizeUnits::Viewport(percentage) => style.set_padding_left_vp(percentage),
                SizeUnits::Auto => style.set_padding_left_auto(),
            };
        }
        if let Some(val_serde) = &panel_style_serde.padding_right {
            let val = val_serde.to_size_units();
            match val {
                SizeUnits::Pixels(pixels) => style.set_padding_right_px(pixels),
                SizeUnits::Percentage(percentage) => style.set_padding_right_pc(percentage),
                SizeUnits::Viewport(percentage) => style.set_padding_right_vp(percentage),
                SizeUnits::Auto => style.set_padding_right_auto(),
            };
        }
        if let Some(val_serde) = &panel_style_serde.padding_top {
            let val = val_serde.to_size_units();
            match val {
                SizeUnits::Pixels(pixels) => style.set_padding_top_px(pixels),
                SizeUnits::Percentage(percentage) => style.set_padding_top_pc(percentage),
                SizeUnits::Viewport(percentage) => style.set_padding_top_vp(percentage),
                SizeUnits::Auto => style.set_padding_top_auto(),
            };
        }
        if let Some(val_serde) = &panel_style_serde.padding_bottom {
            let val = val_serde.to_size_units();
            match val {
                SizeUnits::Pixels(pixels) => style.set_padding_bottom_px(pixels),
                SizeUnits::Percentage(percentage) => style.set_padding_bottom_pc(percentage),
                SizeUnits::Viewport(percentage) => style.set_padding_bottom_vp(percentage),
                SizeUnits::Auto => style.set_padding_bottom_auto(),
            };
        }
        if let Some(val_serde) = &panel_style_serde.row_between {
            let val = val_serde.to_size_units();
            match val {
                SizeUnits::Pixels(pixels) => style.set_row_between_px(pixels),
                SizeUnits::Percentage(percentage) => style.set_row_between_pc(percentage),
                SizeUnits::Viewport(percentage) => style.set_row_between_vp(percentage),
                SizeUnits::Auto => style.set_row_between_auto(),
            };
        }
        if let Some(val_serde) = &panel_style_serde.col_between {
            let val = val_serde.to_size_units();
            match val {
                SizeUnits::Pixels(pixels) => style.set_col_between_px(pixels),
                SizeUnits::Percentage(percentage) => style.set_col_between_pc(percentage),
                SizeUnits::Viewport(percentage) => style.set_col_between_vp(percentage),
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
                SizeUnits::Viewport(percentage) => style.set_width_vp(percentage),
                SizeUnits::Auto => style.set_width_auto(),
            };
        }
        if let Some(val_serde) = &self.height {
            let val = val_serde.to_size_units();
            match val {
                SizeUnits::Pixels(pixels) => style.set_height_px(pixels),
                SizeUnits::Percentage(percentage) => style.set_height_pc(percentage),
                SizeUnits::Viewport(percentage) => style.set_height_vp(percentage),
                SizeUnits::Auto => style.set_height_auto(),
            };
        }
        if let Some(val_serde) = &self.width_min {
            let val = val_serde.to_size_units();
            match val {
                SizeUnits::Pixels(pixels) => style.set_width_min_px(pixels),
                SizeUnits::Percentage(percentage) => style.set_width_min_pc(percentage),
                SizeUnits::Viewport(percentage) => style.set_width_min_vp(percentage),
                SizeUnits::Auto => style.set_width_min_auto(),
            };
        }
        if let Some(val_serde) = &self.width_max {
            let val = val_serde.to_size_units();
            match val {
                SizeUnits::Pixels(pixels) => style.set_width_max_px(pixels),
                SizeUnits::Percentage(percentage) => style.set_width_max_pc(percentage),
                SizeUnits::Viewport(percentage) => style.set_width_max_vp(percentage),
                SizeUnits::Auto => style.set_width_max_auto(),
            };
        }
        if let Some(val_serde) = &self.height_min {
            let val = val_serde.to_size_units();
            match val {
                SizeUnits::Pixels(pixels) => style.set_height_min_px(pixels),
                SizeUnits::Percentage(percentage) => style.set_height_min_pc(percentage),
                SizeUnits::Viewport(percentage) => style.set_height_min_vp(percentage),
                SizeUnits::Auto => style.set_height_min_auto(),
            };
        }
        if let Some(val_serde) = &self.height_max {
            let val = val_serde.to_size_units();
            match val {
                SizeUnits::Pixels(pixels) => style.set_height_max_px(pixels),
                SizeUnits::Percentage(percentage) => style.set_height_max_pc(percentage),
                SizeUnits::Viewport(percentage) => style.set_height_max_vp(percentage),
                SizeUnits::Auto => style.set_height_max_auto(),
            };
        }
        if let Some(val_serde) = &self.margin_left {
            let val = val_serde.to_margin_units();
            match val {
                MarginUnits::Pixels(pixels) => style.set_margin_left_px(pixels),
                MarginUnits::Percentage(percentage) => style.set_margin_left_pc(percentage),
                MarginUnits::Viewport(percentage) => style.set_margin_left_vp(percentage),
            };
        }
        if let Some(val_serde) = &self.margin_right {
            let val = val_serde.to_margin_units();
            match val {
                MarginUnits::Pixels(pixels) => style.set_margin_right_px(pixels),
                MarginUnits::Percentage(percentage) => style.set_margin_right_pc(percentage),
                MarginUnits::Viewport(percentage) => style.set_margin_right_vp(percentage),
            };
        }
        if let Some(val_serde) = &self.margin_top {
            let val = val_serde.to_margin_units();
            match val {
                MarginUnits::Pixels(pixels) => style.set_margin_top_px(pixels),
                MarginUnits::Percentage(percentage) => style.set_margin_top_pc(percentage),
                MarginUnits::Viewport(percentage) => style.set_margin_top_vp(percentage),
            };
        }
        if let Some(val_serde) = &self.margin_bottom {
            let val = val_serde.to_margin_units();
            match val {
                MarginUnits::Pixels(pixels) => style.set_margin_bottom_px(pixels),
                MarginUnits::Percentage(percentage) => style.set_margin_bottom_pc(percentage),
                MarginUnits::Viewport(percentage) => style.set_margin_bottom_vp(percentage),
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
            style.set_aspect_ratio(w as f32, h as f32);
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

    fn to_textbox_style(&self, style: &mut TextboxStyleMut) {

        // textbox specific
        let WidgetStyleBits::Textbox(textbox_style_serde) = &self.widget_style else {
            panic!("Expected textbox style");
        };

        if let Some((r, g, b)) = &textbox_style_serde.hover_color {
            style.set_hover_color(Color::new(*r, *g, *b));
        }

        if let Some((r, g, b)) = &textbox_style_serde.active_color {
            style.set_active_color(Color::new(*r, *g, *b));
        }

        // panel-specific
        let panel_style_serde = &textbox_style_serde.panel;

        if let Some((r, g, b)) = &panel_style_serde.background_color {
            style.set_background_color(Color::new(*r, *g, *b));
        }
        if let Some(background_alpha) = panel_style_serde.background_alpha {
            let val: u8 = background_alpha.to();
            let val: f32 = val as f32;
            let val = val / 10.0;
            style.set_background_alpha(val);
        }

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
                SizeUnits::Viewport(percentage) => style.set_width_vp(percentage),
                SizeUnits::Auto => style.set_width_auto(),
            };
        }
        if let Some(val_serde) = &self.height {
            let val = val_serde.to_size_units();
            match val {
                SizeUnits::Pixels(pixels) => style.set_height_px(pixels),
                SizeUnits::Percentage(percentage) => style.set_height_pc(percentage),
                SizeUnits::Viewport(percentage) => style.set_height_vp(percentage),
                SizeUnits::Auto => style.set_height_auto(),
            };
        }
        if let Some(val_serde) = &self.width_min {
            let val = val_serde.to_size_units();
            match val {
                SizeUnits::Pixels(pixels) => style.set_width_min_px(pixels),
                SizeUnits::Percentage(percentage) => style.set_width_min_pc(percentage),
                SizeUnits::Viewport(percentage) => style.set_width_min_vp(percentage),
                SizeUnits::Auto => style.set_width_min_auto(),
            };
        }
        if let Some(val_serde) = &self.width_max {
            let val = val_serde.to_size_units();
            match val {
                SizeUnits::Pixels(pixels) => style.set_width_max_px(pixels),
                SizeUnits::Percentage(percentage) => style.set_width_max_pc(percentage),
                SizeUnits::Viewport(percentage) => style.set_width_max_vp(percentage),
                SizeUnits::Auto => style.set_width_max_auto(),
            };
        }
        if let Some(val_serde) = &self.height_min {
            let val = val_serde.to_size_units();
            match val {
                SizeUnits::Pixels(pixels) => style.set_height_min_px(pixels),
                SizeUnits::Percentage(percentage) => style.set_height_min_pc(percentage),
                SizeUnits::Viewport(percentage) => style.set_height_min_vp(percentage),
                SizeUnits::Auto => style.set_height_min_auto(),
            };
        }
        if let Some(val_serde) = &self.height_max {
            let val = val_serde.to_size_units();
            match val {
                SizeUnits::Pixels(pixels) => style.set_height_max_px(pixels),
                SizeUnits::Percentage(percentage) => style.set_height_max_pc(percentage),
                SizeUnits::Viewport(percentage) => style.set_height_max_vp(percentage),
                SizeUnits::Auto => style.set_height_max_auto(),
            };
        }
        if let Some(val_serde) = &self.margin_left {
            let val = val_serde.to_margin_units();
            match val {
                MarginUnits::Pixels(pixels) => style.set_margin_left_px(pixels),
                MarginUnits::Percentage(percentage) => style.set_margin_left_pc(percentage),
                MarginUnits::Viewport(percentage) => style.set_margin_left_vp(percentage),
            };
        }
        if let Some(val_serde) = &self.margin_right {
            let val = val_serde.to_margin_units();
            match val {
                MarginUnits::Pixels(pixels) => style.set_margin_right_px(pixels),
                MarginUnits::Percentage(percentage) => style.set_margin_right_pc(percentage),
                MarginUnits::Viewport(percentage) => style.set_margin_right_vp(percentage),
            };
        }
        if let Some(val_serde) = &self.margin_top {
            let val = val_serde.to_margin_units();
            match val {
                MarginUnits::Pixels(pixels) => style.set_margin_top_px(pixels),
                MarginUnits::Percentage(percentage) => style.set_margin_top_pc(percentage),
                MarginUnits::Viewport(percentage) => style.set_margin_top_vp(percentage),
            };
        }
        if let Some(val_serde) = &self.margin_bottom {
            let val = val_serde.to_margin_units();
            match val {
                MarginUnits::Pixels(pixels) => style.set_margin_bottom_px(pixels),
                MarginUnits::Percentage(percentage) => style.set_margin_bottom_pc(percentage),
                MarginUnits::Viewport(percentage) => style.set_margin_bottom_vp(percentage),
            };
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
