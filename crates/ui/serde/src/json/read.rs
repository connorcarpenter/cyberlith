use std::collections::HashMap;

use asset_id::AssetId;
use render_api::base::Color;
use ui_builder_config::{BaseNodeStyle, Button, ButtonStyle, NodeId, NodeStyle, Panel, PanelStyle, StyleId, Text, TextStyle, Textbox, TextboxStyle, UiConfig, Widget, WidgetKind, WidgetStyle, CharacterWhitelist};
use ui_layout::{Alignment, LayoutType, MarginUnits, PositionType, SizeUnits, Solid};

use super::{AlignmentJson, ColorJson, LayoutTypeJson, MarginUnitsJson, PanelJson, PositionTypeJson, SizeUnitsJson, SolidJson, TextboxCharWhitelistJson, UiConfigJson, UiNodeJson, UiStyleJson, WidgetJson, WidgetStyleJson};
use crate::json::{
    ButtonJson, ButtonStyleJson, PanelStyleJson, TextStyleJson, TextboxJson, TextboxStyleJson,
};

impl Into<UiConfig> for UiConfigJson {
    fn into(self) -> UiConfig {
        let mut ui_config = UiConfig::new();

        // ui_serde -> ui
        let UiConfigJson {
            text_color,
            text_icon_asset_id,
            eye_icon_asset_id,
            first_input,
            styles,
            nodes,
        } = self;

        // text color
        ui_config.set_text_color(text_color.into());

        // text icon
        let text_icon_asset_id = AssetId::from_str(&text_icon_asset_id).unwrap();
        ui_config.set_text_icon_asset_id(&text_icon_asset_id);

        // eye icon
        let eye_icon_asset_id = AssetId::from_str(&eye_icon_asset_id).unwrap();
        ui_config.set_eye_icon_asset_id(&eye_icon_asset_id);

        // first input
        if let Some(first_input_id) = first_input {
            ui_config.set_first_input(NodeId::from_usize(first_input_id));
        }

        // styles
        let mut style_index_to_id = HashMap::new();

        let mut style_index = 0;
        for style_serde in styles {
            //info!("style_serde: {}, {:?}", style_index, style_serde);

            let new_style = style_serde.into();
            let style_id = ui_config.insert_style(new_style);
            style_index_to_id.insert(style_index, style_id);
            style_index += 1;
        }

        // nodes
        let Some(root_node_serde) = nodes.first() else {
            panic!("Reading Ui with no nodes");
        };
        //info!("0 - root_node_serde: {:?}", root_node_serde);

        let root_mut = ui_config.node_mut(&UiConfig::ROOT_NODE_ID).unwrap();
        if let Some(style_index) = &root_node_serde.style_id {
            let style_id = *style_index_to_id.get(style_index).unwrap();
            root_mut.set_style_id(style_id);
        }
        let WidgetJson::Panel(panel_serde) = &root_node_serde.widget else {
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
}

fn convert_nodes_recurse_panel(
    style_index_to_id: &HashMap<usize, StyleId>,
    nodes: &Vec<UiNodeJson>,
    panel_serde: &PanelJson,
    ui_config: &mut UiConfig,
    panel_id: &NodeId,
) {
    for child_index in &panel_serde.children {
        let child_index = *child_index as usize;
        let child_node_serde = &nodes[child_index];

        //info!("{} - child_node_serde: {:?}", child_index, child_node_serde);

        match child_node_serde.widget_kind() {
            WidgetKind::Panel => {
                // creates a new panel
                let child_panel_id = ui_config.create_node(Widget::Panel(Panel::new()));
                let Widget::Panel(panel) = &mut ui_config.node_mut(panel_id).unwrap().widget else {
                    panic!("Expected panel widget");
                };
                panel.add_child(child_panel_id);

                // add style
                if let Some(style_index) = &child_node_serde.style_id {
                    let style_id = *style_index_to_id.get(style_index).unwrap();
                    let child_node = ui_config.node_mut(&child_panel_id).unwrap();
                    child_node.set_style_id(style_id);
                }

                // recurse
                let WidgetJson::Panel(child_panel_serde) = &child_node_serde.widget else {
                    panic!("Expected panel widget");
                };
                convert_nodes_recurse_panel(
                    style_index_to_id,
                    nodes,
                    child_panel_serde,
                    ui_config,
                    &child_panel_id,
                );
            }
            WidgetKind::Text => {
                let WidgetJson::Text(child_text_serde) = &child_node_serde.widget else {
                    panic!("Expected text widget");
                };

                // creates a new text
                let child_text_id =
                    ui_config.create_node(Widget::Text(Text::new(child_text_serde.text.as_str())));
                let Widget::Panel(panel) = &mut ui_config.node_mut(panel_id).unwrap().widget else {
                    panic!("Expected panel widget");
                };
                panel.add_child(child_text_id);

                // add style
                if let Some(style_index) = &child_node_serde.style_id {
                    let style_id = *style_index_to_id.get(style_index).unwrap();
                    let child_node = ui_config.node_mut(&child_text_id).unwrap();
                    child_node.set_style_id(style_id);
                }
            }
            WidgetKind::Button => {
                let WidgetJson::Button(child_button_serde) = &child_node_serde.widget else {
                    panic!("Expected button widget");
                };

                // creates a new button
                let child_button_id = ui_config.create_node(Widget::Button(Button::new(
                    child_button_serde.id_str.as_str(),
                )));
                let Widget::Panel(panel) = &mut ui_config.node_mut(panel_id).unwrap().widget else {
                    panic!("Expected panel widget");
                };
                panel.add_child(child_button_id);

                // add style
                if let Some(style_index) = &child_node_serde.style_id {
                    let style_id = *style_index_to_id.get(style_index).unwrap();
                    let child_node = ui_config.node_mut(&child_button_id).unwrap();
                    child_node.set_style_id(style_id);
                }

                // add navigation
                set_button_navigation(child_button_serde, ui_config, &child_button_id);

                // recurse
                convert_nodes_recurse_button(
                    style_index_to_id,
                    nodes,
                    child_button_serde,
                    ui_config,
                    &child_button_id,
                );
            }
            WidgetKind::Textbox => {
                let WidgetJson::Textbox(child_textbox_serde) = &child_node_serde.widget else {
                    panic!("Expected textbox widget");
                };

                // creates a new textbox
                let mut textbox = Textbox::new(
                    child_textbox_serde.id_str.as_str(),
                );
                textbox.is_password = child_textbox_serde.is_password;
                textbox.char_whitelist = child_textbox_serde.char_whitelist.map(|v| v.into());
                let child_textbox_id = ui_config.create_node(Widget::Textbox(textbox));
                let Widget::Panel(panel) = &mut ui_config.node_mut(panel_id).unwrap().widget else {
                    panic!("Expected panel widget");
                };
                panel.add_child(child_textbox_id);

                // add style
                if let Some(style_index) = &child_node_serde.style_id {
                    let style_id = *style_index_to_id.get(style_index).unwrap();
                    let child_node = ui_config.node_mut(&child_textbox_id).unwrap();
                    child_node.set_style_id(style_id);
                }

                // add navigation
                set_textbox_navigation(child_textbox_serde, ui_config, &child_textbox_id);
            }
        }
    }
}

fn set_button_navigation(button_serde: &ButtonJson, ui_config: &mut UiConfig, button_id: &NodeId) {
    let button_nav_serde = &button_serde.navigation;

    let node = ui_config.node_mut(button_id).unwrap();
    let Widget::Button(button) = &mut node.widget else {
        panic!("Expected button widget");
    };
    let nav = &mut button.navigation;

    if let Some(nav_str) = button_nav_serde.up.as_ref() {
        nav.up_goes_to = Some(nav_str.to_string());
    }
    if let Some(nav_str) = button_nav_serde.down.as_ref() {
        nav.down_goes_to = Some(nav_str.to_string());
    }
    if let Some(nav_str) = button_nav_serde.left.as_ref() {
        nav.left_goes_to = Some(nav_str.to_string());
    }
    if let Some(nav_str) = button_nav_serde.right.as_ref() {
        nav.right_goes_to = Some(nav_str.to_string());
    }
    if let Some(nav_str) = button_nav_serde.tab.as_ref() {
        nav.tab_goes_to = Some(nav_str.to_string());
    }
}

fn convert_nodes_recurse_button(
    style_index_to_id: &HashMap<usize, StyleId>,
    nodes: &Vec<UiNodeJson>,
    button_serde: &ButtonJson,
    ui_config: &mut UiConfig,
    button_id: &NodeId,
) {
    for child_index in &button_serde.panel.children {
        let child_index = *child_index as usize;
        let child_node_serde = &nodes[child_index];

        //info!("{} - child_node_serde: {:?}", child_index, child_node_serde);

        match child_node_serde.widget_kind() {
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
                    let style_id = *style_index_to_id.get(style_index).unwrap();
                    let child_node = ui_config.node_mut(&child_panel_id).unwrap();
                    child_node.set_style_id(style_id);
                }
                let WidgetJson::Panel(child_panel_serde) = &child_node_serde.widget else {
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
            }
            WidgetKind::Text => {
                let WidgetJson::Text(child_text_serde) = &child_node_serde.widget else {
                    panic!("Expected text widget");
                };

                // creates a new text
                let child_text_id =
                    ui_config.create_node(Widget::Text(Text::new(child_text_serde.text.as_str())));
                let Widget::Button(button) = &mut ui_config.node_mut(button_id).unwrap().widget
                else {
                    panic!("Expected button widget");
                };
                button.add_child(child_text_id);

                // add style
                for style_index in &child_node_serde.style_id {
                    let style_id = *style_index_to_id.get(style_index).unwrap();
                    let child_node = ui_config.node_mut(&child_text_id).unwrap();
                    child_node.set_style_id(style_id);
                }
            }
            _ => {
                panic!("Button can only contain Panel or Text");
            }
        }
    }
}

fn set_textbox_navigation(
    textbox_serde: &TextboxJson,
    ui_config: &mut UiConfig,
    textbox_id: &NodeId,
) {
    let textbox_nav_serde = &textbox_serde.navigation;

    let node = ui_config.node_mut(textbox_id).unwrap();
    let Widget::Textbox(textbox) = &mut node.widget else {
        panic!("Expected textbox widget");
    };
    let nav = &mut textbox.navigation;

    if let Some(nav_str) = textbox_nav_serde.up.as_ref() {
        nav.up_goes_to = Some(nav_str.to_string());
    }
    if let Some(nav_str) = textbox_nav_serde.down.as_ref() {
        nav.down_goes_to = Some(nav_str.to_string());
    }
    if let Some(nav_str) = textbox_nav_serde.left.as_ref() {
        nav.left_goes_to = Some(nav_str.to_string());
    }
    if let Some(nav_str) = textbox_nav_serde.right.as_ref() {
        nav.right_goes_to = Some(nav_str.to_string());
    }
    if let Some(nav_str) = textbox_nav_serde.tab.as_ref() {
        nav.tab_goes_to = Some(nav_str.to_string());
    }
}

impl Into<PositionType> for PositionTypeJson {
    fn into(self) -> PositionType {
        match self {
            Self::Absolute => PositionType::Absolute,
            Self::Relative => PositionType::Relative,
        }
    }
}

impl Into<SizeUnits> for SizeUnitsJson {
    fn into(self) -> SizeUnits {
        match self {
            Self::Percentage(percentage) => SizeUnits::Percentage(percentage),
            Self::Viewport(viewport) => SizeUnits::Viewport(viewport),
            Self::Auto => SizeUnits::Auto,
        }
    }
}

impl Into<MarginUnits> for MarginUnitsJson {
    fn into(self) -> MarginUnits {
        match self {
            Self::Percentage(percentage) => MarginUnits::Percentage(percentage),
            Self::Viewport(viewport) => MarginUnits::Viewport(viewport),
        }
    }
}

impl Into<Solid> for SolidJson {
    fn into(self) -> Solid {
        match self {
            Self::Fit => Solid::Fit,
            Self::Fill => Solid::Fill,
        }
    }
}

impl Into<Alignment> for AlignmentJson {
    fn into(self) -> Alignment {
        match self {
            Self::Start => Alignment::Start,
            Self::Center => Alignment::Center,
            Self::End => Alignment::End,
        }
    }
}

impl Into<LayoutType> for LayoutTypeJson {
    fn into(self) -> LayoutType {
        match self {
            Self::Row => LayoutType::Row,
            Self::Column => LayoutType::Column,
        }
    }
}

impl Into<Color> for ColorJson {
    fn into(self) -> Color {
        Color::new(self.r, self.g, self.b)
    }
}

impl Into<NodeStyle> for UiStyleJson {
    fn into(self) -> NodeStyle {
        NodeStyle {
            parent_style: self.parent_style.map(|id| StyleId::new(id as u32)),
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
                aspect_ratio: self.aspect_ratio,
                self_halign: self.self_halign.map(Into::into),
                self_valign: self.self_valign.map(Into::into),
            },
        }
    }
}

impl Into<WidgetStyle> for WidgetStyleJson {
    fn into(self) -> WidgetStyle {
        match self {
            Self::Panel(panel) => WidgetStyle::Panel(panel.into()),
            Self::Text(text) => WidgetStyle::Text(text.into()),
            Self::Button(button) => WidgetStyle::Button(button.into()),
            Self::Textbox(textbox) => WidgetStyle::Textbox(textbox.into()),
        }
    }
}

impl Into<PanelStyle> for PanelStyleJson {
    fn into(self) -> PanelStyle {
        PanelStyle {
            is_viewport: self.is_viewport,
            background_color: self.background_color.map(Into::into),
            background_alpha: self.background_alpha,
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

impl Into<TextStyle> for TextStyleJson {
    fn into(self) -> TextStyle {
        TextStyle {
            background_color: self.background_color.map(Into::into),
            background_alpha: self.background_alpha,
        }
    }
}

impl Into<ButtonStyle> for ButtonStyleJson {
    fn into(self) -> ButtonStyle {
        ButtonStyle {
            panel: self.panel.into(),
            hover_color: self.hover_color.map(Into::into),
            down_color: self.down_color.map(Into::into),
        }
    }
}

impl Into<TextboxStyle> for TextboxStyleJson {
    fn into(self) -> TextboxStyle {
        TextboxStyle {
            background_color: self.background_color.map(Into::into),
            background_alpha: self.background_alpha,
            hover_color: self.hover_color.map(Into::into),
            active_color: self.active_color.map(Into::into),
            select_color: self.select_color.map(Into::into),
        }
    }
}

impl Into<CharacterWhitelist> for TextboxCharWhitelistJson {
    fn into(self) -> CharacterWhitelist {
        match self {
            Self::Alphanumeric => CharacterWhitelist::Alphanumeric,
            Self::Password => CharacterWhitelist::Password,
            Self::Email => CharacterWhitelist::Email,
        }
    }
}