use std::collections::HashMap;

use render_api::base::Color;
use ui_builder_config::{
    BaseNodeStyle, Button, ButtonStyle, NodeId, NodeStyle, Panel, PanelStyle, Spinner,
    SpinnerStyle, StyleId, Text, TextStyle, Textbox, TextboxStyle, UiConfig, UiContainer,
    ValidationType, Widget, WidgetKind, WidgetStyle,
};
use ui_layout::{Alignment, LayoutType, MarginUnits, PositionType, SizeUnits, Solid};

use super::{
    AlignmentJson, ColorJson, LayoutTypeJson, MarginUnitsJson, PanelJson, PositionTypeJson,
    SizeUnitsJson, SolidJson, SpinnerStyleJson, UiConfigJson, UiNodeJson, UiStyleJson,
    ValidationJson, WidgetJson, WidgetStyleJson,
};
use crate::json::{
    ButtonJson, ButtonStyleJson, PanelStyleJson, TextStyleJson, TextboxJson, TextboxStyleJson,
};

impl Into<UiConfig> for UiConfigJson {
    fn into(self) -> UiConfig {
        let mut ui_config = UiConfig::new();

        // ui_serde -> ui
        let UiConfigJson {
            first_input,
            styles,
            nodes,
        } = self;

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
            &mut ui_config,
            &UiConfig::ROOT_NODE_ID,
            panel_serde,
        );

        ui_config
    }
}

fn convert_nodes_recurse_panel(
    style_index_to_id: &HashMap<usize, StyleId>,
    nodes: &Vec<UiNodeJson>,
    ui_config: &mut UiConfig,
    parent_panel_id: &NodeId,
    parent_panel_serde: &PanelJson,
) {
    for child_index in &parent_panel_serde.children {
        let child_index = *child_index as usize;
        let child_node_serde = &nodes[child_index];

        //info!("{} - child_node_serde: {:?}", child_index, child_node_serde);

        let child_node_id = match child_node_serde.widget_kind() {
            WidgetKind::Panel => {
                let child_node_id = ui_config.create_node(
                    child_node_serde
                        .id_str
                        .as_ref()
                        .map(|id_str| id_str.as_str()),
                    Widget::Panel(Panel::new()),
                );
                let Widget::Panel(panel_parent) =
                    &mut ui_config.node_mut(parent_panel_id).unwrap().widget
                else {
                    panic!("Expected panel widget");
                };
                panel_parent.add_child(child_node_id);

                // add style
                if let Some(style_index) = &child_node_serde.style_id {
                    let style_id = *style_index_to_id.get(style_index).unwrap();
                    let child_node = ui_config.node_mut(&child_node_id).unwrap();
                    child_node.set_style_id(style_id);
                }

                // recurse
                let WidgetJson::Panel(child_widget_serde) = &child_node_serde.widget else {
                    panic!("Expected panel widget");
                };
                convert_nodes_recurse_panel(
                    style_index_to_id,
                    nodes,
                    ui_config,
                    &child_node_id,
                    child_widget_serde,
                );

                child_node_id
            }
            WidgetKind::Button => {
                let WidgetJson::Button(child_widget_serde) = &child_node_serde.widget else {
                    panic!("Expected button widget");
                };
                let child_node_id = ui_config.create_node(
                    child_node_serde.id_str.as_ref().map(|s| s.as_str()),
                    Widget::Button(Button::new()),
                );
                let Widget::Panel(panel_parent) =
                    &mut ui_config.node_mut(parent_panel_id).unwrap().widget
                else {
                    panic!("Expected panel widget");
                };
                panel_parent.add_child(child_node_id);

                // add style
                if let Some(style_index) = &child_node_serde.style_id {
                    let style_id = *style_index_to_id.get(style_index).unwrap();
                    let child_node = ui_config.node_mut(&child_node_id).unwrap();
                    child_node.set_style_id(style_id);
                }

                // add navigation
                set_button_navigation(child_widget_serde, ui_config, &child_node_id);

                // recurse
                convert_nodes_recurse_button(
                    style_index_to_id,
                    nodes,
                    ui_config,
                    &child_node_id,
                    child_widget_serde,
                );

                child_node_id
            }
            WidgetKind::Text => {
                let WidgetJson::Text(child_widget_serde) = &child_node_serde.widget else {
                    panic!("Expected text widget");
                };
                let child = Text::new(child_widget_serde.init_text.as_str());
                let child_node_id = ui_config.create_node(
                    child_node_serde.id_str.as_ref().map(|s| s.as_str()),
                    Widget::Text(child),
                );
                let Widget::Panel(panel_parent) =
                    &mut ui_config.node_mut(parent_panel_id).unwrap().widget
                else {
                    panic!("Expected panel widget");
                };
                panel_parent.add_child(child_node_id);

                // add style
                if let Some(style_index) = &child_node_serde.style_id {
                    let style_id = *style_index_to_id.get(style_index).unwrap();
                    let child_node = ui_config.node_mut(&child_node_id).unwrap();
                    child_node.set_style_id(style_id);
                }

                child_node_id
            }
            WidgetKind::Textbox => {
                let WidgetJson::Textbox(child_widget_serde) = &child_node_serde.widget else {
                    panic!("Expected textbox widget");
                };
                let mut child = Textbox::new();
                child.is_password = child_widget_serde.is_password;
                child.validation = child_widget_serde.validation.map(|v| v.into());
                let child_node_id = ui_config.create_node(
                    child_node_serde.id_str.as_ref().map(|s| s.as_str()),
                    Widget::Textbox(child),
                );
                let Widget::Panel(panel_parent) =
                    &mut ui_config.node_mut(parent_panel_id).unwrap().widget
                else {
                    panic!("Expected panel widget");
                };
                panel_parent.add_child(child_node_id);

                // add style
                if let Some(style_index) = &child_node_serde.style_id {
                    let style_id = *style_index_to_id.get(style_index).unwrap();
                    let child_node = ui_config.node_mut(&child_node_id).unwrap();
                    child_node.set_style_id(style_id);
                }

                // add navigation
                set_textbox_navigation(child_widget_serde, ui_config, &child_node_id);

                child_node_id
            }
            WidgetKind::Spinner => {
                let child = Spinner::new();
                let child_node_id = ui_config.create_node(
                    child_node_serde.id_str.as_ref().map(|s| s.as_str()),
                    Widget::Spinner(child),
                );
                let Widget::Panel(panel_parent) =
                    &mut ui_config.node_mut(parent_panel_id).unwrap().widget
                else {
                    panic!("Expected panel widget");
                };
                panel_parent.add_child(child_node_id);

                // add style
                if let Some(style_index) = &child_node_serde.style_id {
                    let style_id = *style_index_to_id.get(style_index).unwrap();
                    let child_node = ui_config.node_mut(&child_node_id).unwrap();
                    child_node.set_style_id(style_id);
                }

                child_node_id
            }
            WidgetKind::UiContainer => {
                let child = UiContainer::new();
                let child_node_id = ui_config.create_node(
                    child_node_serde.id_str.as_ref().map(|s| s.as_str()),
                    Widget::UiContainer(child),
                );
                let Widget::Panel(panel_parent) =
                    &mut ui_config.node_mut(parent_panel_id).unwrap().widget
                else {
                    panic!("Expected panel widget");
                };
                panel_parent.add_child(child_node_id);

                // add style
                if let Some(style_index) = &child_node_serde.style_id {
                    let style_id = *style_index_to_id.get(style_index).unwrap();
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
    ui_config: &mut UiConfig,
    parent_button_id: &NodeId,
    parent_button_serde: &ButtonJson,
) {
    for child_index in &parent_button_serde.panel.children {
        let child_index = *child_index as usize;
        let child_node_serde = &nodes[child_index];

        //info!("{} - child_node_serde: {:?}", child_index, child_node_serde);

        let child_node_id = match child_node_serde.widget_kind() {
            WidgetKind::Panel => {
                let child = Panel::new();
                let child_id = ui_config.create_node(
                    child_node_serde.id_str.as_ref().map(|s| s.as_str()),
                    Widget::Panel(child),
                );
                let Widget::Button(parent_button) =
                    &mut ui_config.node_mut(parent_button_id).unwrap().widget
                else {
                    panic!("Expected button widget");
                };
                parent_button.add_child(child_id);

                // add style
                if let Some(style_index) = &child_node_serde.style_id {
                    let style_id = *style_index_to_id.get(style_index).unwrap();
                    let child_node = ui_config.node_mut(&child_id).unwrap();
                    child_node.set_style_id(style_id);
                }
                let WidgetJson::Panel(child_widget_serde) = &child_node_serde.widget else {
                    panic!("Expected panel widget");
                };

                // recurse
                convert_nodes_recurse_panel(
                    style_index_to_id,
                    nodes,
                    ui_config,
                    &child_id,
                    child_widget_serde,
                );

                child_id
            }
            WidgetKind::Text => {
                let WidgetJson::Text(child_widget_serde) = &child_node_serde.widget else {
                    panic!("Expected text widget");
                };
                let child = Text::new(child_widget_serde.init_text.as_str());
                let child_id = ui_config.create_node(
                    child_node_serde.id_str.as_ref().map(|s| s.as_str()),
                    Widget::Text(child),
                );
                let Widget::Button(parent_button) =
                    &mut ui_config.node_mut(parent_button_id).unwrap().widget
                else {
                    panic!("Expected button widget");
                };
                parent_button.add_child(child_id);

                // add style
                for style_index in &child_node_serde.style_id {
                    let style_id = *style_index_to_id.get(style_index).unwrap();
                    let child_node = ui_config.node_mut(&child_id).unwrap();
                    child_node.set_style_id(style_id);
                }

                child_id
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
            Self::Pixels(pixels) => SizeUnits::Pixels(pixels),
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
            Self::Spinner(spinner) => WidgetStyle::Spinner(spinner.into()),
            Self::UiContainer => WidgetStyle::UiContainer,
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
            text_color: self.text_color.map(Into::into),
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
            text_color: self.text_color.map(Into::into),
            hover_color: self.hover_color.map(Into::into),
            active_color: self.active_color.map(Into::into),
            select_color: self.select_color.map(Into::into),
        }
    }
}

impl Into<SpinnerStyle> for SpinnerStyleJson {
    fn into(self) -> SpinnerStyle {
        SpinnerStyle {
            background_color: self.background_color.map(Into::into),
            background_alpha: self.background_alpha,
            spinner_color: self.spinner_color.map(Into::into),
        }
    }
}

impl Into<ValidationType> for ValidationJson {
    fn into(self) -> ValidationType {
        match self {
            Self::Alphanumeric => ValidationType::Username,
            Self::Password => ValidationType::Password,
            Self::Email => ValidationType::Email,
        }
    }
}
