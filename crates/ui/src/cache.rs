use std::collections::HashMap;

use ui_layout::{Cache, Node};

use crate::NodeId;

#[derive(Default)]
pub struct LayoutCache {
    rect: HashMap<NodeId, (f32, f32, f32, f32)>,
}

impl LayoutCache {
    pub fn new() -> Self {
        Self {
            rect: HashMap::new(),
        }
    }

    pub fn bounds(&self, node: &NodeId) -> Option<(f32, f32, f32, f32)> {
        self.rect
            .get(node)
            .map(|(width, height, posx, posy)| (*width, *height, *posx, *posy))
    }
}

impl Cache for LayoutCache {
    type Node = NodeId;

    fn width(&self, node: &Self::Node) -> f32 {
        if let Some(rect) = self.rect.get(&node.key()) {
            return rect.0;
        }

        0.0
    }

    fn height(&self, node: &Self::Node) -> f32 {
        if let Some(rect) = self.rect.get(&node.key()) {
            return rect.1;
        }

        0.0
    }

    fn posx(&self, node: &Self::Node) -> f32 {
        if let Some(rect) = self.rect.get(&node.key()) {
            return rect.2;
        }

        0.0
    }

    fn posy(&self, node: &Self::Node) -> f32 {
        if let Some(rect) = self.rect.get(&node.key()) {
            return rect.3;
        }

        0.0
    }

    fn set_bounds(&mut self, node: &Self::Node, posx: f32, posy: f32, width: f32, height: f32) {
        if let Some(rect) = self.rect.get_mut(&node.key()) {
            //info!("setting bounds for node: {:?}", node.key());
            rect.0 = width;
            rect.1 = height;
            rect.2 = posx;
            rect.3 = posy;
        } else {
            //info!("inserting bounds for node: {:?}", node.key());
            self.rect.insert(node.key(), (width, height, posx, posy));
        }
    }
}
