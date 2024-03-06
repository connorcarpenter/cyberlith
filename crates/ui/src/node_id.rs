use std::fmt::{Display, Formatter};
use bevy_log::info;

use morphorm::{LayoutType, Node, PositionType, SizeUnits, Solid, SpaceUnits};

use crate::{panel::Panel, node::{NodeKind, NodeStore, UiNode}};

#[derive(Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash, Debug, Default)]
pub struct NodeId(u32);

impl NodeId {
    pub(crate) const fn new(id: u32) -> Self {
        NodeId(id)
    }

    pub(crate) fn increment(&mut self) {
        self.0 += 1;
    }

    pub(crate) fn panel_ref<'a>(&'a self, store: &'a NodeStore) -> Option<&Panel> {
        let node = store.get(self)?;
        if node.kind == NodeKind::Panel {
            return UiNode::downcast_ref::<Panel>(
                node.widget.as_ref()
            );
        }
        None
    }

    // pub(crate) fn label_ref<'a>(&'a self, store: &'a NodeStore) -> Option<&Label> {
    //     let node = store.get(self)?;
    //     if node.kind == NodeKind::Label {
    //         return UiNode::downcast_ref::<Label>(
    //             node.widget.as_ref()
    //         );
    //     }
    //     None
    // }
}

impl Display for NodeId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Node for NodeId {
    type Store = NodeStore;
    type Tree = NodeStore;
    type ChildIter<'t> = std::slice::Iter<'t, NodeId>;
    type CacheKey = Self;
    type SubLayout<'a> = ();

    fn key(&self) -> Self::CacheKey {
        *self
    }

    fn children<'t>(&'t self, store: &'t NodeStore) -> Self::ChildIter<'t> {

        if let Some(panel_ref) = self.panel_ref(store) {
            return panel_ref.children.iter();
        }

        return [].iter();
    }

    fn visible(&self, store: &NodeStore) -> bool {
        if let Some(node) = store.get(self) {
            node.visible
        } else {
            false
        }
    }

    fn layout_type(&self, store: &NodeStore) -> Option<LayoutType> {
        let panel_ref = self.panel_ref(store)?;
        Some(panel_ref.style.layout_type)
    }

    fn position_type(&self, store: &NodeStore) -> Option<PositionType> {
        let node = store.get(self)?;
        Some(node.style.position_type)
    }

    fn width(&self, store: &NodeStore) -> Option<SizeUnits> {
        let node = store.get(self)?;
        Some(node.style.width)
    }

    fn height(&self, store: &NodeStore) -> Option<SizeUnits> {
        let node = store.get(self)?;
        Some(node.style.height)
    }

    fn left(&self, store: &NodeStore) -> Option<SpaceUnits> {
        let node = store.get(self)?;
        Some(node.style.margin_left)
    }

    fn right(&self, store: &NodeStore) -> Option<SpaceUnits> {
        let node = store.get(self)?;
        Some(node.style.margin_right)
    }

    fn top(&self, store: &NodeStore) -> Option<SpaceUnits> {
        let node = store.get(self)?;
        Some(node.style.margin_top)
    }

    fn bottom(&self, store: &NodeStore) -> Option<SpaceUnits> {
        let node = store.get(self)?;
        Some(node.style.margin_bottom)
    }

    fn content_size(
        &self,
        _store: &NodeStore,
        _sublayout: &mut Self::SubLayout<'_>,

        // only 1 of these will be Some
        _computed_width: Option<f32>,
        _computed_height: Option<f32>,

    ) -> Option<(f32, f32)> {
        info!("content size");
        None
    }

    fn child_left(&self, store: &NodeStore) -> Option<SpaceUnits> {
        let panel_ref = self.panel_ref(store)?;
        Some(panel_ref.style.padding_left)
    }

    fn child_right(&self, store: &NodeStore) -> Option<SpaceUnits> {
        let panel_ref = self.panel_ref(store)?;
        Some(panel_ref.style.padding_right)
    }

    fn child_top(&self, store: &NodeStore) -> Option<SpaceUnits> {
        let panel_ref = self.panel_ref(store)?;
        Some(panel_ref.style.padding_top)
    }

    fn child_bottom(&self, store: &NodeStore) -> Option<SpaceUnits> {
        let panel_ref = self.panel_ref(store)?;
        Some(panel_ref.style.padding_bottom)
    }

    fn row_between(&self, store: &NodeStore) -> Option<SpaceUnits> {
        let panel_ref = self.panel_ref(store)?;
        Some(panel_ref.style.row_between)
    }

    fn col_between(&self, store: &NodeStore) -> Option<SpaceUnits> {
        let panel_ref = self.panel_ref(store)?;
        Some(panel_ref.style.col_between)
    }

    fn min_width(&self, store: &NodeStore) -> Option<SizeUnits> {
        let node = store.get(self)?;
        Some(node.style.width_min)
    }

    fn min_height(&self, store: &NodeStore) -> Option<SizeUnits> {
        let node = store.get(self)?;
        Some(node.style.height_min)
    }

    fn max_width(&self, store: &NodeStore) -> Option<SizeUnits> {
        let node = store.get(self)?;
        Some(node.style.width_max)
    }

    fn max_height(&self, store: &NodeStore) -> Option<SizeUnits> {
        let node = store.get(self)?;
        Some(node.style.height_max)
    }

    fn min_left(&self, store: &NodeStore) -> Option<SpaceUnits> {
        let node = store.get(self)?;
        Some(node.style.margin_left_min)
    }

    fn min_right(&self, store: &NodeStore) -> Option<SpaceUnits> {
        let node = store.get(self)?;
        Some(node.style.margin_right_min)
    }

    fn min_top(&self, store: &NodeStore) -> Option<SpaceUnits> {
        let node = store.get(self)?;
        Some(node.style.margin_top_min)
    }

    fn min_bottom(&self, store: &NodeStore) -> Option<SpaceUnits> {
        let node = store.get(self)?;
        Some(node.style.margin_bottom_min)
    }

    fn max_left(&self, store: &NodeStore) -> Option<SpaceUnits> {
        let node = store.get(self)?;
        Some(node.style.margin_left_max)
    }

    fn max_right(&self, store: &NodeStore) -> Option<SpaceUnits> {
        let node = store.get(self)?;
        Some(node.style.margin_right_max)
    }

    fn max_top(&self, store: &NodeStore) -> Option<SpaceUnits> {
        let node = store.get(self)?;
        Some(node.style.margin_top_max)
    }

    fn max_bottom(&self, store: &NodeStore) -> Option<SpaceUnits> {
        let node = store.get(self)?;
        Some(node.style.margin_bottom_max)
    }

    fn solid(&self, store: &NodeStore) -> Option<Solid> {
        let node = store.get(self)?;
        let val = node.style.solid_override?;
        Some(val)
    }

    fn aspect_ratio(&self, store: &Self::Store) -> Option<f32> {
        let node = store.get(self)?;
        Some(node.style.aspect_ratio_w_over_h)
    }
}
