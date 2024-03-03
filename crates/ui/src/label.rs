use bevy_log::info;
use asset_render::{AssetManager, TextStyle};
use render_api::{resources::RenderFrame, components::{RenderLayer, Transform}};

use crate::{widget::Widget, ui::Globals};

#[derive(Clone)]
pub struct Label {
    text: String,
}

impl Label {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
        }
    }
}

impl Widget for Label {
    fn draw(
        &self,
        render_frame: &mut RenderFrame,
        render_layer_opt: Option<&RenderLayer>,
        asset_manager: &AssetManager,
        globals: &Globals,
        transform: &Transform
    ) {
        let Some(text_handle) = globals.get_text_handle() else {
            panic!("No text handle found in globals");
        };

        // TODO: use some kind of text style from parent panel
        // TODO: text should fill the entire panel
        let style = TextStyle::new(transform.scale.y, 6.0);

        // info!("Drawing label: {}", self.text);
        let mut position = transform.translation;
        position.y += transform.scale.y / 2.0;

        asset_manager.draw_text(
            render_frame,
            render_layer_opt,
            text_handle,
            &style,
            &position,
            &self.text,
        );
    }
}