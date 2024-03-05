use std::any::Any;

use asset_render::AssetManager;
use render_api::{resources::RenderFrame, components::{RenderLayer, Transform}};

use crate::{ui::Globals, style::NodeStyle, node::NodeStore, cache::LayoutCache};

pub(crate) trait Widget: Any + Send + Sync + WidgetBoxClone {
    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn draw(
        &self,
        render_frame: &mut RenderFrame,
        render_layer_opt: Option<&RenderLayer>,
        asset_manager: &AssetManager,
        globals: &Globals,
        cache: &LayoutCache,
        store: &NodeStore,
        node_style: &NodeStyle,
        transform: &Transform
    );
}

pub(crate) trait WidgetBoxClone {
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