use std::fmt::{Display, Formatter};

use crate::{
    layout::layout, Alignment, LayoutCache, LayoutType, MarginUnits, NodeStore, PositionType, Size,
    SizeUnits, Solid, TextMeasurer, UiVisibilityStore,
};

#[derive(Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash, Debug, Default)]
pub struct NodeId(u32);

impl NodeId {
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    pub fn as_usize(&self) -> usize {
        self.0 as usize
    }

    pub fn from_usize(id: usize) -> Self {
        Self(id as u32)
    }
}

impl Display for NodeId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl NodeId {
    pub fn layout(
        &self,
        cache: &mut LayoutCache,
        store: &dyn NodeStore,
        state_store: &UiVisibilityStore,
        text_measurer: &dyn TextMeasurer,
        viewport_width: f32,
        viewport_height: f32,
    ) -> Size {
        cache.set_bounds(self, 0.0, 0.0, 0.0, viewport_width, viewport_height);
        layout(
            true,
            self,
            LayoutType::Column,
            (viewport_width, viewport_height),
            0.0,
            0.0,
            cache,
            store,
            state_store,
            text_measurer,
        )
    }
}

impl NodeId {
    pub(crate) fn children<'a>(&'a self, store: &'a dyn NodeStore) -> std::slice::Iter<NodeId> {
        store.node_children(self)
    }

    pub(crate) fn visible(&self, store: &UiVisibilityStore) -> bool {
        store.get_node_visibility(self).unwrap()
    }

    // all of these unwrap_or_default
    pub(crate) fn layout_type(&self, store: &dyn NodeStore) -> LayoutType {
        store.node_layout_type(self)
    }

    // all of these unwrap_or_default
    pub(crate) fn position_type(&self, store: &dyn NodeStore) -> PositionType {
        store.node_position_type(self)
    }

    // all of these unwrap_or(SizeUnits::Percentage(100.0))
    pub(crate) fn width(&self, store: &dyn NodeStore) -> SizeUnits {
        store.node_width(self)
    }

    // all of these unwrap_or(SizeUnits::Percentage(100.0))
    pub(crate) fn height(&self, store: &dyn NodeStore) -> SizeUnits {
        store.node_height(self)
    }

    // all of these unwrap_or(SizeUnits::Pixels(0.0))
    pub(crate) fn width_min(&self, store: &dyn NodeStore) -> SizeUnits {
        store.node_width_min(self)
    }

    // all of these unwrap_or(SizeUnits::Pixels(0.0))
    pub(crate) fn height_min(&self, store: &dyn NodeStore) -> SizeUnits {
        store.node_height_min(self)
    }

    // all of these unwrap_or(SizeUnits::Pixels(f32::MAX))
    pub(crate) fn width_max(&self, store: &dyn NodeStore) -> SizeUnits {
        store.node_width_max(self)
    }

    // all of these unwrap_or(SizeUnits::Pixels(f32::MAX))
    pub(crate) fn height_max(&self, store: &dyn NodeStore) -> SizeUnits {
        store.node_height_max(self)
    }

    // all of these unwrap_or_default
    pub(crate) fn margin_left(&self, store: &dyn NodeStore) -> MarginUnits {
        store.node_margin_left(self)
    }

    // all of these unwrap_or_default
    pub(crate) fn margin_right(&self, store: &dyn NodeStore) -> MarginUnits {
        store.node_margin_right(self)
    }

    // all of these unwrap_or_default
    pub(crate) fn margin_top(&self, store: &dyn NodeStore) -> MarginUnits {
        store.node_margin_top(self)
    }

    // all of these unwrap_or_default
    pub(crate) fn margin_bottom(&self, store: &dyn NodeStore) -> MarginUnits {
        store.node_margin_bottom(self)
    }

    // all of these unwrap_or_default
    pub(crate) fn padding_left(&self, store: &dyn NodeStore) -> SizeUnits {
        store.node_padding_left(self)
    }

    // all of these unwrap_or_default
    pub(crate) fn padding_right(&self, store: &dyn NodeStore) -> SizeUnits {
        store.node_padding_right(self)
    }

    // all of these unwrap_or_default
    pub(crate) fn padding_top(&self, store: &dyn NodeStore) -> SizeUnits {
        store.node_padding_top(self)
    }

    // all of these unwrap_or_default
    pub(crate) fn padding_bottom(&self, store: &dyn NodeStore) -> SizeUnits {
        store.node_padding_bottom(self)
    }

    // all of these unwrap_or_default
    pub(crate) fn row_between(&self, store: &dyn NodeStore) -> SizeUnits {
        store.node_row_between(self)
    }

    // all of these unwrap_or_default
    pub(crate) fn col_between(&self, store: &dyn NodeStore) -> SizeUnits {
        store.node_col_between(self)
    }

    // no default here .. None doesn't do anything, Some does
    pub(crate) fn is_viewport(&self, store: &dyn NodeStore) -> bool {
        store.node_is_viewport(self)
    }

    // no default here .. None doesn't do anything, Some does
    pub(crate) fn is_solid(&self, store: &dyn NodeStore) -> Option<Solid> {
        store.node_is_solid(self)
    }

    pub(crate) fn is_text(&self, store: &dyn NodeStore) -> bool {
        store.node_is_text(self)
    }

    pub(crate) fn calculate_text_width(
        &self,
        store: &dyn NodeStore,
        text_measurer: &dyn TextMeasurer,
        height: f32,
    ) -> f32 {
        store.node_calculate_text_width(self, text_measurer, height)
    }

    // panics if solid() is None but this isn't ..
    pub(crate) fn aspect_ratio(&self, store: &dyn NodeStore) -> Option<f32> {
        store.node_aspect_ratio(self)
    }

    // all of these unwrap_or_default
    pub(crate) fn self_halign(&self, store: &dyn NodeStore) -> Alignment {
        store.node_self_halign(self)
    }

    // all of these unwrap_or_default
    pub(crate) fn self_valign(&self, store: &dyn NodeStore) -> Alignment {
        store.node_self_valign(self)
    }

    // all of these unwrap_or_default
    pub(crate) fn children_halign(&self, store: &dyn NodeStore) -> Alignment {
        store.node_children_halign(self)
    }

    // all of these unwrap_or_default
    pub(crate) fn children_valign(&self, store: &dyn NodeStore) -> Alignment {
        store.node_children_valign(self)
    }
}
