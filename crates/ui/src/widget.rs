use asset_render::AssetManager;
use render_api::{resources::RenderFrame, components::{RenderLayer, Transform}};

use crate::ui::Globals;

pub trait Widget: Send + Sync + 'static + WidgetBoxClone {
    fn draw(
        &self,
        render_frame: &mut RenderFrame,
        render_layer_opt: Option<&RenderLayer>,
        asset_manager: &AssetManager,
        globals: &Globals,
        transform: &Transform
    );
}

pub trait WidgetBoxClone {
    fn clone_box(&self) -> Box<dyn Widget>;
}

impl<T: Widget + Clone + 'static> WidgetBoxClone for T {
    fn clone_box(&self) -> Box<dyn Widget> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Widget> {
    fn clone(&self) -> Box<dyn Widget> {
        WidgetBoxClone::clone_box(self.as_ref())
    }
}