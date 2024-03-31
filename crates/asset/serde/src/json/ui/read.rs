use std::collections::HashMap;

use asset_id::AssetId;
use render_api::base::Color;
use ui_layout::{Alignment, LayoutType, MarginUnits, PositionType, SizeUnits, Solid};
use ui_types::{Button, ButtonStyle, NodeId, NodeStyle, Panel, PanelStyle, StyleId, Text, Textbox, TextboxStyle, TextStyle, UiConfig, Widget, WidgetKind, WidgetStyle};

use crate::json::{ButtonJson, PanelStyleJson, TextboxJson};
use super::{
    AlignmentJson, ColorJson, LayoutTypeJson, MarginUnitsJson, PanelJson, PositionTypeJson,
    SizeUnitsJson, SolidJson, UiConfigJson, UiNodeJson, UiStyleJson, WidgetJson, WidgetStyleJson,
};

impl UiConfigJson {
    pub fn to_ui(self) -> UiConfig {
        let mut ui_config = UiConfig::new();

        // ui_serde -> ui
        let UiConfigJson {
            text_color,
            text_icon_asset_id,
            first_input,
            styles,
            nodes,
        } = self;

        // text color
        ui_config.set_text_color(text_color.to_color());

        // text icon
        let text_icon_asset_id = AssetId::from_str(&text_icon_asset_id).unwrap();
        ui_config.set_text_icon_asset_id(&text_icon_asset_id);

        // first input
        if let Some(first_input_id) = first_input {
            ui_config.set_first_input(NodeId::from_usize(first_input_id));
        }

        // styles
        let mut style_index_to_id = HashMap::new();

        for (style_index, style_serde) in styles.iter().enumerate() {
            //info!("style_serde: {}, {:?}", style_index, style_serde);

            let style_widget_kind = style_serde.widget_kind();
            let new_style = match style_widget_kind {
                WidgetKind::Panel => {
                    let mut panel_style = PanelStyle::empty();
                    style_serde.to_panel_style(&mut panel_style);
                    let mut node_style = NodeStyle::empty(WidgetStyle::Panel(panel_style));
                    style_serde.to_node_style(&mut node_style);
                    node_style
                },
                WidgetKind::Text => {
                    let mut text_style = TextStyle::empty();
                    style_serde.to_text_style(&mut text_style);
                    let mut node_style = NodeStyle::empty(WidgetStyle::Text(text_style));
                    style_serde.to_node_style(&mut node_style);
                    node_style
                },
                WidgetKind::Button => {
                    let mut button_style = ButtonStyle::empty();
                    style_serde.to_button_style(&mut button_style);
                    let mut node_style = NodeStyle::empty(WidgetStyle::Button(button_style));
                    style_serde.to_node_style(&mut node_style);
                    node_style
                },
                WidgetKind::Textbox => {
                    let mut textbox_style = TextboxStyle::empty();
                    style_serde.to_textbox_style(&mut textbox_style);
                    let mut node_style = NodeStyle::empty(WidgetStyle::Textbox(textbox_style));
                    style_serde.to_node_style(&mut node_style);
                    node_style
                },
            };
            let style_id = ui_config.insert_style(new_style);
            style_index_to_id.insert(style_index, style_id);
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
        convert_nodes_recurse_panel(&style_index_to_id, &nodes, panel_serde, &mut ui_config, &UiConfig::ROOT_NODE_ID);

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
                let child_text_id = ui_config.create_node(Widget::Text(Text::new(child_text_serde.text.as_str())));
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
                let child_button_id = ui_config.create_node(Widget::Button(Button::new(child_button_serde.id_str.as_str())));
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
                set_button_navigation(
                    child_button_serde,
                    ui_config,
                    &child_button_id,
                );

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
                let child_textbox_id = ui_config.create_node(Widget::Textbox(Textbox::new(child_textbox_serde.id_str.as_str())));
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
                set_textbox_navigation(
                    child_textbox_serde,
                    ui_config,
                    &child_textbox_id,
                );
            }
        }
    }
}

fn set_button_navigation(
    button_serde: &ButtonJson,
    ui_config: &mut UiConfig,
    button_id: &NodeId,
) {
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
                let Widget::Button(button) = &mut ui_config.node_mut(button_id).unwrap().widget else {
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
                let child_text_id = ui_config.create_node(Widget::Text(Text::new(child_text_serde.text.as_str())));
                let Widget::Button(button) = &mut ui_config.node_mut(button_id).unwrap().widget else {
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
            Self::Viewport(viewport) => SizeUnits::Viewport(*viewport),
            Self::Auto => SizeUnits::Auto,
        }
    }
}

impl MarginUnitsJson {
    fn to_margin_units(&self) -> MarginUnits {
        match self {
            Self::Pixels(pixels) => MarginUnits::Pixels(*pixels),
            Self::Percentage(percentage) => MarginUnits::Percentage(*percentage),
            Self::Viewport(viewport) => MarginUnits::Viewport(*viewport),
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

    fn to_node_style(&self, style: &mut NodeStyle) {
        style.position_type = self.position_type.as_ref().map(|val| val.to_position_type());
        style.width = self.width.as_ref().map(|val| val.to_size_units());
        style.height = self.height.as_ref().map(|val| val.to_size_units());
        style.width_min = self.width_min.as_ref().map(|val| val.to_size_units());
        style.width_max = self.width_max.as_ref().map(|val| val.to_size_units());
        style.height_min = self.height_min.as_ref().map(|val| val.to_size_units());
        style.height_max = self.height_max.as_ref().map(|val| val.to_size_units());
        style.margin_left = self.margin_left.as_ref().map(|val| val.to_margin_units());
        style.margin_right = self.margin_right.as_ref().map(|val| val.to_margin_units());
        style.margin_top = self.margin_top.as_ref().map(|val| val.to_margin_units());
        style.margin_bottom = self.margin_bottom.as_ref().map(|val| val.to_margin_units());
        style.solid_override = self.solid_override.as_ref().map(|val| val.to_solid());
        if let Some((w, h)) = self.aspect_ratio {
            style.set_aspect_ratio(w as f32, h as f32);
        }
        style.self_halign = self.self_halign.as_ref().map(|val| val.to_alignment());
        style.self_valign = self.self_valign.as_ref().map(|val| val.to_alignment());
    }

    fn to_panel_style(&self, panel_style: &mut PanelStyle) {
        // panel-specific
        let WidgetStyleJson::Panel(panel_style_serde) = &self.widget_style else {
            panic!("Expected panel style");
        };

        Self::to_panel_style_impl(panel_style, panel_style_serde);
    }

    fn to_panel_style_impl(panel_style: &mut PanelStyle, panel_style_serde: &PanelStyleJson) {
        panel_style.background_color = panel_style_serde.background_color.as_ref().map(|val| val.to_color());
        if let Some(background_alpha) = panel_style_serde.background_alpha {
            panel_style.set_background_alpha(background_alpha);
        }
        panel_style.layout_type = panel_style_serde.layout_type.as_ref().map(|val| val.to_layout_type());
        panel_style.padding_left = panel_style_serde.padding_left.as_ref().map(|val| val.to_size_units());
        panel_style.padding_right = panel_style_serde.padding_right.as_ref().map(|val| val.to_size_units());
        panel_style.padding_top = panel_style_serde.padding_top.as_ref().map(|val| val.to_size_units());
        panel_style.padding_bottom = panel_style_serde.padding_bottom.as_ref().map(|val| val.to_size_units());
        panel_style.row_between = panel_style_serde.row_between.as_ref().map(|val| val.to_size_units());
        panel_style.col_between = panel_style_serde.col_between.as_ref().map(|val| val.to_size_units());
        panel_style.children_halign = panel_style_serde.children_halign.as_ref().map(|val| val.to_alignment());
        panel_style.children_valign = panel_style_serde.children_valign.as_ref().map(|val| val.to_alignment());
    }

    fn to_text_style(&self, text_style: &mut TextStyle) {
        // text-specific
        let WidgetStyleJson::Text(text_style_serde) = &self.widget_style else {
            panic!("Expected text style");
        };
        text_style.background_color = text_style_serde.background_color.as_ref().map(|val| val.to_color());
        if let Some(background_alpha) = text_style_serde.background_alpha {
            text_style.set_background_alpha(background_alpha);
        }
    }

    fn to_button_style(&self, button_style: &mut ButtonStyle) {

        // button-specific
        let WidgetStyleJson::Button(button_style_serde) = &self.widget_style else {
            panic!("Expected panel style");
        };
        button_style.hover_color = button_style_serde.hover_color.as_ref().map(|val| val.to_color());
        button_style.down_color = button_style_serde.down_color.as_ref().map(|val| val.to_color());

        // panel-specific
        Self::to_panel_style_impl(&mut button_style.panel, &button_style_serde.panel);
    }

    fn to_textbox_style(&self, textbox_style: &mut TextboxStyle) {

        // textbox specific
        let WidgetStyleJson::Textbox(textbox_style_serde) = &self.widget_style else {
            panic!("Expected textbox style");
        };

        textbox_style.hover_color = textbox_style_serde.hover_color.as_ref().map(|val| val.to_color());
        textbox_style.active_color = textbox_style_serde.active_color.as_ref().map(|val| val.to_color());
        textbox_style.select_color = textbox_style_serde.select_color.as_ref().map(|val| val.to_color());

        // panel-specific
        let panel_style_serde = &textbox_style_serde.panel;
        Self::to_panel_style_impl(&mut textbox_style.panel, panel_style_serde);
    }
}
