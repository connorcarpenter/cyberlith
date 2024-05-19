use std::collections::HashMap;

use naia_serde::{
    BitReader, SerdeErr, SerdeInternal as Serde, UnsignedInteger, UnsignedVariableInteger,
};

use asset_id::AssetId;
use render_api::base::Color;
use ui_builder_config::{
    BaseNodeStyle, Button, ButtonStyle, NodeId, NodeStyle, Panel, PanelStyle, Spinner,
    SpinnerStyle, StyleId, Text, TextStyle, Textbox, TextboxStyle, UiConfig, UiContainer,
    ValidationType, Widget, WidgetKind, WidgetStyle,
};
use ui_layout::{Alignment, LayoutType, MarginUnits, PositionType, SizeUnits, Solid};

use crate::bits::{
    AlignmentBits, ButtonBits, ButtonStyleBits, ColorBits, LayoutTypeBits, MarginUnitsBits,
    PanelBits, PanelStyleBits, PositionTypeBits, SizeUnitsBits, SolidBits, SpinnerStyleBits,
    TextStyleBits, TextboxBits, TextboxStyleBits, UiAction, UiActionType, UiNodeBits, UiStyleBits,
    ValidationBits, WidgetBits, WidgetStyleBits,
};

pub fn read_bits(data: &[u8]) -> Result<UiConfig, SerdeErr> {
    let actions = bytes_to_actions(data)?;
    Ok(convert_actions_to_ui_config(actions))
}

fn convert_actions_to_ui_config(actions: Vec<UiAction>) -> UiConfig {
    let mut ui_config = UiConfig::new();

    let mut style_index_to_id: HashMap<usize, StyleId> = HashMap::new();
    let mut style_count = 0;

    let mut nodes = Vec::new();

    for action in actions {
        match action {
            UiAction::TextIconAssetId(asset_id) => {
                ui_config.set_text_icon_asset_id(&asset_id);
            }
            UiAction::EyeIconAssetId(asset_id) => {
                ui_config.set_eye_icon_asset_id(&asset_id);
            }
            UiAction::FirstInput(node_id_opt) => {
                if let Some(node_id) = node_id_opt {
                    ui_config.set_first_input(node_id)
                }
            }
            UiAction::Style(style_serde) => {
                let new_style: NodeStyle = style_serde.into();
                let style_id = ui_config.insert_style(new_style);
                style_index_to_id.insert(style_count, style_id);
                style_count += 1;
            }
            UiAction::Node(node) => {
                nodes.push(node);
            }
        }
    }

    // process nodes recursively
    let Some(root_node_serde) = nodes.first() else {
        panic!("Reading Ui with no nodes");
    };
    //info!("0 - root_node_serde: {:?}", root_node_serde);

    let root_mut = ui_config.node_mut(&UiConfig::ROOT_NODE_ID).unwrap();
    if let Some(style_index) = &root_node_serde.style_id {
        let style_index = *style_index as usize;
        let style_id = *style_index_to_id.get(&style_index).unwrap();
        root_mut.set_style_id(style_id);
    }
    let WidgetBits::Panel(panel_serde) = &root_node_serde.widget else {
        panic!("Expected panel widget");
    };
    convert_nodes_recurse_panel(
        &style_index_to_id,
        &nodes,
        panel_serde,
        &mut ui_config,
        &UiConfig::ROOT_NODE_ID,
    );

    ui_config
}

fn bytes_to_actions(data: &[u8]) -> Result<Vec<UiAction>, SerdeErr> {
    let mut bit_reader = BitReader::new(data);
    let bit_reader = &mut bit_reader;
    let mut actions = Vec::new();

    loop {
        let action_type = UiActionType::de(bit_reader)?;

        match action_type {
            UiActionType::TextIconAssetId => {
                let val = u32::de(bit_reader)?;
                let asset_id = AssetId::from_u32(val)?;
                actions.push(UiAction::TextIconAssetId(asset_id));
            }
            UiActionType::EyeIconAssetId => {
                let val = u32::de(bit_reader)?;
                let asset_id = AssetId::from_u32(val)?;
                actions.push(UiAction::EyeIconAssetId(asset_id));
            }
            UiActionType::DefaultButton => {
                let val = Option::<UnsignedVariableInteger<7>>::de(bit_reader)?;
                let val: Option<u64> = val.map(|v| v.to());
                let node_id_opt = val.map(|v| NodeId::from_usize(v as usize));
                actions.push(UiAction::FirstInput(node_id_opt));
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

fn convert_nodes_recurse_panel(
    style_index_to_id: &HashMap<usize, StyleId>,
    nodes: &Vec<UiNodeBits>,
    panel_serde: &PanelBits,
    ui_config: &mut UiConfig,
    panel_id: &NodeId,
) {
    for child_index in &panel_serde.children {
        let child_index = *child_index as usize;
        let child_node_serde = &nodes[child_index];

        //info!("{} - child_node_serde: {:?}", child_index, child_node_serde);

        let child_node_id = match child_node_serde.widget_kind() {
            WidgetKind::Panel => {
                // creates a new panel
                let child_node_id = ui_config.create_node(Widget::Panel(Panel::new()));
                let Widget::Panel(panel) = &mut ui_config.node_mut(panel_id).unwrap().widget else {
                    panic!("Expected panel widget");
                };
                panel.add_child(child_node_id);

                // add style
                if let Some(style_index) = &child_node_serde.style_id {
                    let style_index = *style_index as usize;
                    let style_id = *style_index_to_id.get(&style_index).unwrap();
                    let child_node = ui_config.node_mut(&child_node_id).unwrap();
                    child_node.set_style_id(style_id);
                }

                // recurse
                let WidgetBits::Panel(child_widget_serde) = &child_node_serde.widget else {
                    panic!("Expected panel widget");
                };
                convert_nodes_recurse_panel(
                    style_index_to_id,
                    nodes,
                    child_widget_serde,
                    ui_config,
                    &child_node_id,
                );

                child_node_id
            }
            WidgetKind::Button => {
                let WidgetBits::Button(child_widget_serde) = &child_node_serde.widget else {
                    panic!("Expected button widget");
                };

                // creates a new button
                let child_node_id = ui_config.create_node(Widget::Button(Button::new(
                    child_widget_serde.id_str.as_str(),
                )));
                let Widget::Panel(panel) = &mut ui_config.node_mut(panel_id).unwrap().widget else {
                    panic!("Expected panel widget");
                };
                panel.add_child(child_node_id);

                // add style
                if let Some(style_index) = &child_node_serde.style_id {
                    let style_index = *style_index as usize;
                    let style_id = *style_index_to_id.get(&style_index).unwrap();
                    let child_node = ui_config.node_mut(&child_node_id).unwrap();
                    child_node.set_style_id(style_id);
                }

                // add navigation
                set_button_navigation(nodes, child_widget_serde, ui_config, &child_node_id);

                // recurse
                convert_nodes_recurse_button(
                    style_index_to_id,
                    nodes,
                    child_widget_serde,
                    ui_config,
                    &child_node_id,
                );

                child_node_id
            }
            WidgetKind::Text => {
                let WidgetBits::Text(child_widget_serde) = &child_node_serde.widget else {
                    panic!("Expected text widget");
                };

                // creates a new text
                let text = Text::new(
                    child_widget_serde.id_str.as_ref().map(|v| v.as_str()),
                    &child_widget_serde.init_text,
                );
                let child_node_id = ui_config.create_node(Widget::Text(text));
                let Widget::Panel(panel) = &mut ui_config.node_mut(panel_id).unwrap().widget else {
                    panic!("Expected panel widget");
                };
                panel.add_child(child_node_id);

                // add style
                if let Some(style_index) = &child_node_serde.style_id {
                    let style_index = *style_index as usize;
                    let style_id = *style_index_to_id.get(&style_index).unwrap();
                    let child_node = ui_config.node_mut(&child_node_id).unwrap();
                    child_node.set_style_id(style_id);
                }

                child_node_id
            }
            WidgetKind::Textbox => {
                let WidgetBits::Textbox(child_widget_serde) = &child_node_serde.widget else {
                    panic!("Expected textbox widget");
                };

                // creates a new textbox
                let mut textbox = Textbox::new(child_widget_serde.id_str.as_str());
                textbox.is_password = child_widget_serde.is_password;
                textbox.validation = child_widget_serde.validation.map(|v| v.into());
                let child_node_id = ui_config.create_node(Widget::Textbox(textbox));
                let Widget::Panel(panel) = &mut ui_config.node_mut(panel_id).unwrap().widget else {
                    panic!("Expected panel widget");
                };
                panel.add_child(child_node_id);

                // add style
                for style_index in &child_node_serde.style_id {
                    let style_index = *style_index as usize;
                    let style_id = *style_index_to_id.get(&style_index).unwrap();
                    let child_node = ui_config.node_mut(&child_node_id).unwrap();
                    child_node.set_style_id(style_id);
                }

                // add navigation
                set_textbox_navigation(nodes, child_widget_serde, ui_config, &child_node_id);

                child_node_id
            }
            WidgetKind::Spinner => {
                let WidgetBits::Spinner(child_widget_serde) = &child_node_serde.widget else {
                    panic!("Expected spinner widget");
                };

                // creates a new spinner
                let spinner = Spinner::new(child_widget_serde.id_str.as_str());
                let child_node_id = ui_config.create_node(Widget::Spinner(spinner));
                let Widget::Panel(panel) = &mut ui_config.node_mut(panel_id).unwrap().widget else {
                    panic!("Expected panel widget");
                };
                panel.add_child(child_node_id);

                // add style
                for style_index in &child_node_serde.style_id {
                    let style_index = *style_index as usize;
                    let style_id = *style_index_to_id.get(&style_index).unwrap();
                    let child_node = ui_config.node_mut(&child_node_id).unwrap();
                    child_node.set_style_id(style_id);
                }

                child_node_id
            }
            WidgetKind::UiContainer => {
                let WidgetBits::UiContainer(child_widget_serde) = &child_node_serde.widget else {
                    panic!("Expected spinner widget");
                };

                // creates a new uicontainer
                let ui_container = UiContainer::new(child_widget_serde.id_str.as_str());
                let child_node_id = ui_config.create_node(Widget::UiContainer(ui_container));
                let Widget::Panel(panel) = &mut ui_config.node_mut(panel_id).unwrap().widget else {
                    panic!("Expected panel widget");
                };
                panel.add_child(child_node_id);

                // add style
                for style_index in &child_node_serde.style_id {
                    let style_index = *style_index as usize;
                    let style_id = *style_index_to_id.get(&style_index).unwrap();
                    let child_node = ui_config.node_mut(&child_node_id).unwrap();
                    child_node.set_style_id(style_id);
                }

                child_node_id
            }
        };

        let child_node = ui_config.node_mut(&child_node_id).unwrap();
        child_node.set_visible(child_node_serde.init_visible);
    }
}

fn get_nav_thang<'a>(
    nodes: &'a Vec<UiNodeBits>,
    input_int: Option<&UnsignedVariableInteger<4>>,
) -> Option<&'a str> {
    let input_int = input_int?;
    let nav_index = input_int.to::<u32>() as usize;
    let nav_node_serde = &nodes[nav_index];
    match &nav_node_serde.widget {
        WidgetBits::Button(button_serde) => {
            let nav_str = button_serde.id_str.as_str();
            Some(nav_str)
        }
        WidgetBits::Textbox(textbox_serde) => {
            let nav_str = textbox_serde.id_str.as_str();
            Some(nav_str)
        }
        _ => None,
    }
}

fn set_button_navigation(
    nodes: &Vec<UiNodeBits>,
    button_serde: &ButtonBits,
    ui_config: &mut UiConfig,
    button_id: &NodeId,
) {
    let button_nav_serde = &button_serde.navigation;

    let node = ui_config.node_mut(button_id).unwrap();
    let Widget::Button(button) = &mut node.widget else {
        panic!("Expected button widget");
    };
    let nav = &mut button.navigation;

    if let Some(nav_str) = get_nav_thang(nodes, button_nav_serde.up.as_ref()) {
        nav.up_goes_to = Some(nav_str.to_string());
    }
    if let Some(nav_str) = get_nav_thang(nodes, button_nav_serde.down.as_ref()) {
        nav.down_goes_to = Some(nav_str.to_string());
    }
    if let Some(nav_str) = get_nav_thang(nodes, button_nav_serde.left.as_ref()) {
        nav.left_goes_to = Some(nav_str.to_string());
    }
    if let Some(nav_str) = get_nav_thang(nodes, button_nav_serde.right.as_ref()) {
        nav.right_goes_to = Some(nav_str.to_string());
    }
    if let Some(nav_str) = get_nav_thang(nodes, button_nav_serde.tab.as_ref()) {
        nav.tab_goes_to = Some(nav_str.to_string());
    }
}

fn convert_nodes_recurse_button(
    style_index_to_id: &HashMap<usize, StyleId>,
    nodes: &Vec<UiNodeBits>,
    button_serde: &ButtonBits,
    ui_config: &mut UiConfig,
    button_id: &NodeId,
) {
    for child_index in &button_serde.panel.children {
        let child_index = *child_index as usize;
        let child_node_serde = &nodes[child_index];

        //info!("{} - child_node_serde: {:?}", child_index, child_node_serde);

        let child_node_id = match child_node_serde.widget_kind() {
            WidgetKind::Panel => {
                // creates a new panel
                let child_panel_id = ui_config.create_node(Widget::Panel(Panel::new()));
                let Widget::Button(button) = &mut ui_config.node_mut(button_id).unwrap().widget
                else {
                    panic!("Expected button widget");
                };
                button.add_child(child_panel_id);

                // add style
                if let Some(style_index) = &child_node_serde.style_id {
                    let style_index = *style_index as usize;
                    let style_id = *style_index_to_id.get(&style_index).unwrap();
                    let child_node = ui_config.node_mut(&child_panel_id).unwrap();
                    child_node.set_style_id(style_id);
                }
                let WidgetBits::Panel(child_panel_serde) = &child_node_serde.widget else {
                    panic!("Expected panel widget");
                };

                // recurse
                convert_nodes_recurse_panel(
                    style_index_to_id,
                    nodes,
                    child_panel_serde,
                    ui_config,
                    &child_panel_id,
                );

                child_panel_id
            }
            WidgetKind::Text => {
                let WidgetBits::Text(child_text_serde) = &child_node_serde.widget else {
                    panic!("Expected text widget");
                };

                // creates a new text
                let text = Text::new(
                    child_text_serde.id_str.as_ref().map(|s| s.as_str()),
                    &child_text_serde.init_text,
                );
                let child_text_id = ui_config.create_node(Widget::Text(text));
                let Widget::Button(button) = &mut ui_config.node_mut(button_id).unwrap().widget
                else {
                    panic!("Expected button widget");
                };
                button.add_child(child_text_id);

                // add style
                if let Some(style_index) = &child_node_serde.style_id {
                    let style_index = *style_index as usize;
                    let style_id = *style_index_to_id.get(&style_index).unwrap();
                    let child_node = ui_config.node_mut(&child_text_id).unwrap();
                    child_node.set_style_id(style_id);
                }

                child_text_id
            }
            _ => {
                panic!("Button can only contain Panel or Text");
            }
        };

        let child_node = ui_config.node_mut(&child_node_id).unwrap();
        child_node.set_visible(child_node_serde.init_visible);
    }
}

fn set_textbox_navigation(
    nodes: &Vec<UiNodeBits>,
    textbox_serde: &TextboxBits,
    ui_config: &mut UiConfig,
    textbox_id: &NodeId,
) {
    let textbox_nav_serde = &textbox_serde.navigation;

    let node = ui_config.node_mut(textbox_id).unwrap();
    let Widget::Textbox(textbox) = &mut node.widget else {
        panic!("Expected textbox widget");
    };
    let nav = &mut textbox.navigation;

    if let Some(nav_str) = get_nav_thang(nodes, textbox_nav_serde.up.as_ref()) {
        nav.up_goes_to = Some(nav_str.to_string());
    }
    if let Some(nav_str) = get_nav_thang(nodes, textbox_nav_serde.down.as_ref()) {
        nav.down_goes_to = Some(nav_str.to_string());
    }
    if let Some(nav_str) = get_nav_thang(nodes, textbox_nav_serde.left.as_ref()) {
        nav.left_goes_to = Some(nav_str.to_string());
    }
    if let Some(nav_str) = get_nav_thang(nodes, textbox_nav_serde.right.as_ref()) {
        nav.right_goes_to = Some(nav_str.to_string());
    }
    if let Some(nav_str) = get_nav_thang(nodes, textbox_nav_serde.tab.as_ref()) {
        nav.tab_goes_to = Some(nav_str.to_string());
    }
}

impl Into<PositionType> for PositionTypeBits {
    fn into(self) -> PositionType {
        match self {
            Self::Absolute => PositionType::Absolute,
            Self::Relative => PositionType::Relative,
        }
    }
}

impl Into<SizeUnits> for SizeUnitsBits {
    fn into(self) -> SizeUnits {
        match self {
            Self::Percent(val) => {
                let val: u64 = val.to();
                let val: f32 = val as f32;
                SizeUnits::Percentage(val)
            }
            Self::Viewport(val) => {
                let val: u64 = val.to();
                let val: f32 = (val as f32) / 10.0;
                SizeUnits::Viewport(val)
            }
            Self::Auto => SizeUnits::Auto,
        }
    }
}

impl Into<MarginUnits> for MarginUnitsBits {
    fn into(self) -> MarginUnits {
        match self {
            Self::Percent(val) => {
                let val: u64 = val.to();
                let val: f32 = val as f32;
                MarginUnits::Percentage(val)
            }
            Self::Viewport(val) => {
                let val: u64 = val.to();
                let val: f32 = (val as f32) / 10.0;
                MarginUnits::Viewport(val)
            }
        }
    }
}

impl Into<Solid> for SolidBits {
    fn into(self) -> Solid {
        match self {
            Self::Fit => Solid::Fit,
            Self::Fill => Solid::Fill,
        }
    }
}

impl Into<Alignment> for AlignmentBits {
    fn into(self) -> Alignment {
        match self {
            Self::Start => Alignment::Start,
            Self::Center => Alignment::Center,
            Self::End => Alignment::End,
        }
    }
}

impl Into<LayoutType> for LayoutTypeBits {
    fn into(self) -> LayoutType {
        match self {
            Self::Row => LayoutType::Row,
            Self::Column => LayoutType::Column,
        }
    }
}

impl Into<ValidationType> for ValidationBits {
    fn into(self) -> ValidationType {
        match self {
            Self::Alphanumeric => ValidationType::Username,
            Self::Password => ValidationType::Password,
            Self::Email => ValidationType::Email,
        }
    }
}

impl Into<Color> for ColorBits {
    fn into(self) -> Color {
        Color::new(self.r, self.g, self.b)
    }
}

impl Into<NodeStyle> for UiStyleBits {
    fn into(self) -> NodeStyle {
        NodeStyle {
            parent_style: self.parent_style.map(|val| StyleId::new(val as u32)),
            base: BaseNodeStyle {
                widget_style: self.widget_style.into(),
                position_type: self.position_type.map(Into::into),
                width: self.width.map(Into::into),
                height: self.height.map(Into::into),
                width_min: self.width_min.map(Into::into),
                width_max: self.width_max.map(Into::into),
                height_min: self.height_min.map(Into::into),
                height_max: self.height_max.map(Into::into),
                margin_left: self.margin_left.map(Into::into),
                margin_right: self.margin_right.map(Into::into),
                margin_top: self.margin_top.map(Into::into),
                margin_bottom: self.margin_bottom.map(Into::into),
                solid_override: self.solid_override.map(Into::into),
                aspect_ratio: self.aspect_ratio.map(|(w, h)| (w as f32, h as f32)),
                self_halign: self.self_halign.map(Into::into),
                self_valign: self.self_valign.map(Into::into),
            },
        }
    }
}

impl Into<WidgetStyle> for WidgetStyleBits {
    fn into(self) -> WidgetStyle {
        match self {
            WidgetStyleBits::Panel(panel_style) => WidgetStyle::Panel(panel_style.into()),
            WidgetStyleBits::Text(text_style) => WidgetStyle::Text(text_style.into()),
            WidgetStyleBits::Button(button_style) => WidgetStyle::Button(button_style.into()),
            WidgetStyleBits::Textbox(textbox_style) => WidgetStyle::Textbox(textbox_style.into()),
            WidgetStyleBits::Spinner(spinner_style) => WidgetStyle::Spinner(spinner_style.into()),
            WidgetStyleBits::UiContainer => WidgetStyle::UiContainer,
        }
    }
}

impl Into<PanelStyle> for PanelStyleBits {
    fn into(self) -> PanelStyle {
        PanelStyle {
            is_viewport: self.is_viewport,
            background_color: self.background_color.map(Into::into),
            background_alpha: self.background_alpha.map(bits_into_alpha),
            layout_type: self.layout_type.map(Into::into),
            padding_left: self.padding_left.map(Into::into),
            padding_right: self.padding_right.map(Into::into),
            padding_top: self.padding_top.map(Into::into),
            padding_bottom: self.padding_bottom.map(Into::into),
            row_between: self.row_between.map(Into::into),
            col_between: self.col_between.map(Into::into),
            children_halign: self.children_halign.map(Into::into),
            children_valign: self.children_valign.map(Into::into),
        }
    }
}

impl Into<TextStyle> for TextStyleBits {
    fn into(self) -> TextStyle {
        TextStyle {
            background_color: self.background_color.map(Into::into),
            background_alpha: self.background_alpha.map(bits_into_alpha),
            text_color: self.text_color.map(Into::into),
        }
    }
}

impl Into<ButtonStyle> for ButtonStyleBits {
    fn into(self) -> ButtonStyle {
        ButtonStyle {
            panel: self.panel.into(),
            hover_color: self.hover_color.map(|val| val.into()),
            down_color: self.down_color.map(|val| val.into()),
        }
    }
}

impl Into<TextboxStyle> for TextboxStyleBits {
    fn into(self) -> TextboxStyle {
        TextboxStyle {
            background_color: self.background_color.map(Into::into),
            background_alpha: self.background_alpha.map(bits_into_alpha),
            text_color: self.text_color.map(Into::into),
            hover_color: self.hover_color.map(|val| val.into()),
            active_color: self.active_color.map(|val| val.into()),
            select_color: self.select_color.map(|val| val.into()),
        }
    }
}

impl Into<SpinnerStyle> for SpinnerStyleBits {
    fn into(self) -> SpinnerStyle {
        SpinnerStyle {
            background_color: self.background_color.map(Into::into),
            background_alpha: self.background_alpha.map(bits_into_alpha),
            spinner_color: self.spinner_color.map(Into::into),
        }
    }
}

fn bits_into_alpha(bits_val: UnsignedInteger<4>) -> f32 {
    let val: u8 = bits_val.to();
    let val: f32 = val as f32;
    val / 10.0
}
