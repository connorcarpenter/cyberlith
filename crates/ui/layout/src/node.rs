use crate::{layout, types::*, Cache, TextMeasurer};

pub trait Node: Sized {
    type Store;
    type StateStore;
    type ChildIter<'t>: Iterator<Item = &'t Self>
    where
        Self: 't;
    type CacheKey;

    fn layout<C: Cache<Node = Self>>(
        &self,
        cache: &mut C,
        store: &Self::Store,
        state_store: &Self::StateStore,
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
            viewport_height,
            viewport_width,
            cache,
            store,
            state_store,
            text_measurer
        )
    }

    /// Returns a key which can be used to set/get computed layout data from the [`cache`](crate::Cache).
    fn key(&self) -> Self::CacheKey;

    /// Returns an iterator over the children of the node.
    fn children<'t>(&'t self, store: &'t Self::Store) -> Self::ChildIter<'t>;

    /// Returns a boolean representing whether the node is visible to layout.
    fn visible(&self, state_store: &Self::StateStore) -> bool;

    /// Returns the layout type of the node.
    fn layout_type(&self, store: &Self::Store) -> Option<LayoutType>;

    /// Returns the position type of the node.
    fn position_type(&self, store: &Self::Store) -> Option<PositionType>;

    /// Returns the desired width of the node.
    fn width(&self, store: &Self::Store) -> Option<SizeUnits>;

    /// Returns the desired height of the node.
    fn height(&self, store: &Self::Store) -> Option<SizeUnits>;

    /// Returns the minimum width of the node.
    fn width_min(&self, store: &Self::Store) -> Option<SizeUnits>;

    /// Returns the minimum height of the node.
    fn height_min(&self, store: &Self::Store) -> Option<SizeUnits>;

    /// Returns the maximum width of the node.
    fn width_max(&self, store: &Self::Store) -> Option<SizeUnits>;

    /// Returns the maximum height of the node.
    fn height_max(&self, store: &Self::Store) -> Option<SizeUnits>;

    /// Returns the desired left-side space of the node.
    fn margin_left(&self, store: &Self::Store) -> Option<MarginUnits>;

    /// Returns the desired right-side space of the node.
    fn margin_right(&self, store: &Self::Store) -> Option<MarginUnits>;

    /// Returns the desired top-side space of the node.
    fn margin_top(&self, store: &Self::Store) -> Option<MarginUnits>;

    /// Returns the desired bottom-side space of the node.
    fn margin_bottom(&self, store: &Self::Store) -> Option<MarginUnits>;

    /// Returns the desired left-side child-space of the node.
    fn padding_left(&self, store: &Self::Store) -> Option<SizeUnits>;

    /// Returns the desired left-side child-space of the node.
    fn padding_right(&self, store: &Self::Store) -> Option<SizeUnits>;

    /// Returns the desired left-side child-space of the node.
    fn padding_top(&self, store: &Self::Store) -> Option<SizeUnits>;

    /// Returns the desired left-side child-space of the node.
    fn padding_bottom(&self, store: &Self::Store) -> Option<SizeUnits>;

    /// Returns the desired space to applied between the children of the node on the vertical axis.
    fn row_between(&self, store: &Self::Store) -> Option<SizeUnits>;

    /// Returns the desired space to be applied between the children of the node on the horizontal axis.
    fn col_between(&self, store: &Self::Store) -> Option<SizeUnits>;

    /// Returns the solid override of the node.
    fn is_solid(&self, store: &Self::Store) -> Option<Solid>;

    /// Returns whether node is text.
    fn is_text(&self, store: &Self::Store) -> bool;

    /// Returns whether node is text.
    fn calculate_text_width(&self, store: &Self::Store, text_measurer: &dyn TextMeasurer, height: f32) -> f32;

    /// Returns the aspect ratio of the node. (width / height)
    fn aspect_ratio(&self, store: &Self::Store) -> Option<f32>;

    /// Returns the horizontal alignment of the node.
    fn self_halign(&self, store: &Self::Store) -> Option<Alignment>;

    /// Returns the vertical alignment of the node.
    fn self_valign(&self, store: &Self::Store) -> Option<Alignment>;

    /// Returns the horizontal alignment of the node's children
    fn children_halign(&self, store: &Self::Store) -> Option<Alignment>;

    /// Returns the vertical alignment of the node's children
    fn children_valign(&self, store: &Self::Store) -> Option<Alignment>;
}

#[derive(Eq, PartialEq, Clone, Copy, Default)]
pub enum Alignment {
    Start,
    #[default]
    Center,
    End,
}

impl Alignment {
    pub(crate) fn has_start(&self) -> bool {
        match self {
            Alignment::Start | Alignment::Center => true,
            _ => false,
        }
    }

    pub(crate) fn has_end(&self) -> bool {
        match self {
            Alignment::End | Alignment::Center => true,
            _ => false,
        }
    }
}

/// Helper trait used internally for converting layout properties into a direction-agnostic value.
pub(crate) trait NodeExt: Node {
    fn main(&self, store: &Self::Store, parent_layout_type: LayoutType) -> SizeUnits {
        match parent_layout_type {
            LayoutType::Row => self.width(store).unwrap_or(SizeUnits::Percentage(100.0)),
            LayoutType::Column => self.height(store).unwrap_or(SizeUnits::Percentage(100.0)),
        }
    }

    fn main_min(&self, store: &Self::Store, parent_layout_type: LayoutType) -> SizeUnits {
        parent_layout_type.select_unwrap_default(
            store,
            |store| self.width_min(store),
            |store| self.height_min(store),
            SizeUnits::Pixels(0.0),
        )
    }

    fn main_max(&self, store: &Self::Store, parent_layout_type: LayoutType) -> SizeUnits {
        parent_layout_type.select_unwrap_default(
            store,
            |store| self.width_max(store),
            |store| self.height_max(store),
            SizeUnits::Pixels(f32::MAX),
        )
    }

    fn cross(&self, store: &Self::Store, parent_layout_type: LayoutType) -> SizeUnits {
        match parent_layout_type {
            LayoutType::Row => self.height(store).unwrap_or(SizeUnits::Percentage(100.0)),
            LayoutType::Column => self.width(store).unwrap_or(SizeUnits::Percentage(100.0)),
        }
    }

    fn cross_min(&self, store: &Self::Store, parent_layout_type: LayoutType) -> SizeUnits {
        parent_layout_type.select_unwrap_default(
            store,
            |store| self.height_min(store),
            |store| self.width_min(store),
            SizeUnits::Pixels(0.0),
        )
    }

    fn cross_max(&self, store: &Self::Store, parent_layout_type: LayoutType) -> SizeUnits {
        parent_layout_type.select_unwrap_default(
            store,
            |store| self.height_max(store),
            |store| self.width_max(store),
            SizeUnits::Pixels(f32::MAX),
        )
    }

    fn margin_main_before(
        &self,
        store: &Self::Store,
        parent_layout_type: LayoutType,
    ) -> MarginUnits {
        parent_layout_type.select_unwrap(
            store,
            |store| self.margin_left(store),
            |store| self.margin_top(store),
        )
    }

    fn margin_main_after(
        &self,
        store: &Self::Store,
        parent_layout_type: LayoutType,
    ) -> MarginUnits {
        parent_layout_type.select_unwrap(
            store,
            |store| self.margin_right(store),
            |store| self.margin_bottom(store),
        )
    }

    fn margin_cross_before(
        &self,
        store: &Self::Store,
        parent_layout_type: LayoutType,
    ) -> MarginUnits {
        parent_layout_type.select_unwrap(
            store,
            |store| self.margin_top(store),
            |store| self.margin_left(store),
        )
    }

    fn margin_cross_after(
        &self,
        store: &Self::Store,
        parent_layout_type: LayoutType,
    ) -> MarginUnits {
        parent_layout_type.select_unwrap(
            store,
            |store| self.margin_bottom(store),
            |store| self.margin_right(store),
        )
    }

    fn padding_main_before(
        &self,
        store: &Self::Store,
        parent_layout_type: LayoutType,
    ) -> SizeUnits {
        parent_layout_type.select_unwrap(
            store,
            |store| self.padding_left(store),
            |store| self.padding_top(store),
        )
    }

    fn padding_main_after(&self, store: &Self::Store, parent_layout_type: LayoutType) -> SizeUnits {
        parent_layout_type.select_unwrap(
            store,
            |store| self.padding_right(store),
            |store| self.padding_bottom(store),
        )
    }

    fn padding_cross_before(
        &self,
        store: &Self::Store,
        parent_layout_type: LayoutType,
    ) -> SizeUnits {
        parent_layout_type.select_unwrap(
            store,
            |store| self.padding_top(store),
            |store| self.padding_left(store),
        )
    }

    fn padding_cross_after(
        &self,
        store: &Self::Store,
        parent_layout_type: LayoutType,
    ) -> SizeUnits {
        parent_layout_type.select_unwrap(
            store,
            |store| self.padding_bottom(store),
            |store| self.padding_right(store),
        )
    }

    fn main_between(&self, store: &Self::Store, parent_layout_type: LayoutType) -> SizeUnits {
        parent_layout_type.select_unwrap(
            store,
            |store| self.col_between(store),
            |store| self.row_between(store),
        )
    }

    // Currently unused until wrapping is implemented
    fn cross_between(&self, store: &Self::Store, parent_layout_type: LayoutType) -> SizeUnits {
        parent_layout_type.select_unwrap(
            store,
            |store| self.row_between(store),
            |store| self.col_between(store),
        )
    }

    fn self_align(&self, store: &Self::Store, parent_layout_type: LayoutType) -> Alignment {
        parent_layout_type.select_unwrap(
            store,
            |store| self.self_valign(store),
            |store| self.self_halign(store),
        )
    }

    fn children_align(&self, store: &Self::Store, layout_type: LayoutType) -> Alignment {
        layout_type.select_unwrap(
            store,
            |store| self.children_halign(store),
            |store| self.children_valign(store),
        )
    }
}

// Implement `NodeExt` for all types which implement `Node`.
impl<N: Node> NodeExt for N {}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Solid {
    Fit,  // maximum axis uses aspect ratio
    Fill, // minimum axis uses aspect ratio
}