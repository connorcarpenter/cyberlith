use std::collections::HashMap;

use crate::{LayoutType, NodeId};

#[derive(Default)]
pub struct LayoutCache {
    // width, height, x, y, z
    rect: HashMap<NodeId, (f32, f32, f32, f32, f32)>,
}

impl LayoutCache {
    pub fn new() -> Self {
        Self {
            rect: HashMap::new(),
        }
    }

    // output is (width, height, x, y, z)
    pub fn bounds(&self, node: &NodeId) -> Option<(f32, f32, f32, f32, f32)> {
        self.rect
            .get(node)
            .map(|(width, height, posx, posy, posz)| (*width, *height, *posx, *posy, *posz))
    }

    pub fn set_bounds(
        &mut self,
        node: &NodeId,
        posx: f32,
        posy: f32,
        posz: f32,
        width: f32,
        height: f32,
    ) {
        if let Some(rect) = self.rect.get_mut(node) {
            //info!("setting bounds for node: {:?}", node.key());
            rect.0 = width;
            rect.1 = height;
            rect.2 = posx;
            rect.3 = posy;
            rect.4 = posz;
        } else {
            //info!("inserting bounds for node: {:?}", node.key());
            self.rect.insert(*node, (width, height, posx, posy, posz));
        }
    }
}

impl LayoutCache {
    pub(crate) fn set_rect(
        &mut self,
        node: &NodeId,
        parent_layout_type: LayoutType,
        main_pos: f32,
        cross_pos: f32,
        main: f32,
        cross: f32,
    ) {
        match parent_layout_type {
            LayoutType::Row => self.set_bounds(node, main_pos, cross_pos, 0.0, main, cross),
            LayoutType::Column => self.set_bounds(node, cross_pos, main_pos, 0.0, cross, main),
        }
    }
}
