use ui_types::{ButtonStyle, NodeStyle, PanelStyle, StyleId, TextboxStyle, TextStyle, UiConfig, WidgetStyle};

use crate::{ButtonStyleMut, PanelMut, PanelStyleMut, TextboxStyleMut, TextStyleMut};

pub struct UiConfigBuilder;

impl UiConfigBuilder {

    pub fn root_mut(ui_config: &mut UiConfig) -> PanelMut {
        PanelMut::new(ui_config, UiConfig::ROOT_NODE_ID)
    }

    pub fn create_panel_style(ui_config: &mut UiConfig, mut func: impl FnMut(&mut PanelStyleMut)) -> StyleId {

        let new_style_id = ui_config.create_style(NodeStyle::empty(WidgetStyle::Panel(PanelStyle::empty())));
        let mut style_mut = PanelStyleMut::new(ui_config, new_style_id);
        func(&mut style_mut);

        return new_style_id;
    }

    pub fn create_button_style(ui_config: &mut UiConfig, mut func: impl FnMut(&mut ButtonStyleMut)) -> StyleId {

        let new_style_id = ui_config.create_style(NodeStyle::empty(WidgetStyle::Button(ButtonStyle::empty())));
        let mut style_mut = ButtonStyleMut::new(ui_config, new_style_id);
        func(&mut style_mut);

        return new_style_id;
    }

    pub fn create_text_style(ui_config: &mut UiConfig, mut func: impl FnMut(&mut TextStyleMut)) -> StyleId {

        let new_style_id = ui_config.create_style(NodeStyle::empty(WidgetStyle::Text(TextStyle::empty())));
        let mut style_mut = TextStyleMut::new(ui_config, new_style_id);
        func(&mut style_mut);

        return new_style_id;
    }

    pub fn create_textbox_style(ui_config: &mut UiConfig, mut func: impl FnMut(&mut TextboxStyleMut)) -> StyleId {

        let new_style_id = ui_config.create_style(NodeStyle::empty(WidgetStyle::Textbox(TextboxStyle::empty())));
        let mut style_mut = TextboxStyleMut::new(ui_config, new_style_id);
        func(&mut style_mut);

        return new_style_id;
    }
}