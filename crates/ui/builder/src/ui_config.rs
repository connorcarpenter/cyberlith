use ui_builder_config::{
    ButtonStyle, NodeStyle, PanelStyle, SpinnerStyle, StyleId, TextStyle, TextboxStyle, UiConfig,
    WidgetStyle,
};

use crate::{
    ButtonStyleMut, PanelMut, PanelStyleMut, SpinnerStyleMut, TextStyleMut, TextboxStyleMut,
    UiContainerStyleMut,
};

pub trait UiConfigBuild {
    fn root_mut(&mut self) -> PanelMut;

    fn create_panel_style<F: FnMut(&mut PanelStyleMut)>(&mut self, func: F) -> StyleId;
    fn create_text_style<F: FnMut(&mut TextStyleMut)>(&mut self, func: F) -> StyleId;
    fn create_button_style<F: FnMut(&mut ButtonStyleMut)>(&mut self, func: F) -> StyleId;
    fn create_textbox_style<F: FnMut(&mut TextboxStyleMut)>(&mut self, func: F) -> StyleId;
    fn create_spinner_style<F: FnMut(&mut SpinnerStyleMut)>(&mut self, func: F) -> StyleId;
    fn create_ui_container_style<F: FnMut(&mut UiContainerStyleMut)>(&mut self, func: F)
        -> StyleId;
}

impl UiConfigBuild for UiConfig {
    fn root_mut(&mut self) -> PanelMut {
        PanelMut::new(self, UiConfig::ROOT_NODE_ID)
    }

    fn create_panel_style<F: FnMut(&mut PanelStyleMut)>(&mut self, mut func: F) -> StyleId {
        let new_style = NodeStyle::empty(WidgetStyle::Panel(PanelStyle::empty()));
        let new_style_id = self.insert_style(new_style);
        let mut style_mut = PanelStyleMut::new(self, new_style_id);
        func(&mut style_mut);

        return new_style_id;
    }

    fn create_button_style<F: FnMut(&mut ButtonStyleMut)>(&mut self, mut func: F) -> StyleId {
        let new_style = NodeStyle::empty(WidgetStyle::Button(ButtonStyle::empty()));
        let new_style_id = self.insert_style(new_style);
        let mut style_mut = ButtonStyleMut::new(self, new_style_id);
        func(&mut style_mut);

        return new_style_id;
    }

    fn create_text_style<F: FnMut(&mut TextStyleMut)>(&mut self, mut func: F) -> StyleId {
        let new_style = NodeStyle::empty(WidgetStyle::Text(TextStyle::empty()));
        let new_style_id = self.insert_style(new_style);
        let mut style_mut = TextStyleMut::new(self, new_style_id);
        func(&mut style_mut);

        return new_style_id;
    }

    fn create_textbox_style<F: FnMut(&mut TextboxStyleMut)>(&mut self, mut func: F) -> StyleId {
        let new_style = NodeStyle::empty(WidgetStyle::Textbox(TextboxStyle::empty()));
        let new_style_id = self.insert_style(new_style);
        let mut style_mut = TextboxStyleMut::new(self, new_style_id);
        func(&mut style_mut);

        return new_style_id;
    }

    fn create_spinner_style<F: FnMut(&mut SpinnerStyleMut)>(&mut self, mut func: F) -> StyleId {
        let new_style = NodeStyle::empty(WidgetStyle::Spinner(SpinnerStyle::empty()));
        let new_style_id = self.insert_style(new_style);
        let mut style_mut = SpinnerStyleMut::new(self, new_style_id);
        func(&mut style_mut);

        return new_style_id;
    }

    fn create_ui_container_style<F: FnMut(&mut UiContainerStyleMut)>(
        &mut self,
        mut func: F,
    ) -> StyleId {
        let new_style = NodeStyle::empty(WidgetStyle::UiContainer);
        let new_style_id = self.insert_style(new_style);
        let mut style_mut = UiContainerStyleMut::new(self, new_style_id);
        func(&mut style_mut);

        return new_style_id;
    }
}
