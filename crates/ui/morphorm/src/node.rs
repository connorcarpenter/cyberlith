use crate::{layout, types::*, Cache};

/// A `Node` represents a layout element which can be sized and positioned based on
/// a number of layout properties.
///
/// The getter methods in this trait allow for the layout function to retrieve the
/// layout properties of the node. The `Node` trait allows for its layout properties to optionally
/// be stored externally from the node type itself by providing a `Store` associated type, a reference to
/// which is passed to the layout property methods.
///
/// Similarly, the children of the node can be optionally stored externally using the `Tree` associated type,
/// a reference to which is passed to the [`children`](crate::Node::children) method, which returns an iterator on the children of the node,
/// the type of which is specified by the `ChildIter` associated type.
pub trait Node: Sized {
    /// A type representing a store where layout properties can be stored.
    type Store;
    /// A type representing a tree structure where the children of the node can be stored.
    type Tree;
    /// An type representing an iterator over the children of the node.
    type ChildIter<'t>: Iterator<Item = &'t Self>
    where
        Self: 't;
    /// A type representing a key to store and retrieve values from the [`Cache`].
    type CacheKey;
    /// A type representing a context which can be used to save/load state when computing [content size](crate::Node::content_size).
    /// For example, a `TextContext` which could be used to measure (and cache) the size of text, which could
    /// then be used to size an `Auto` layout node using content size.
    type SubLayout<'a>;

    /// Performs layout on the given node returning its computed size.
    ///
    /// The algorithm recurses down the tree, in depth-first order, and performs
    /// layout on every node starting from the input `node`.
    ///
    /// # Arguments
    ///
    /// * `cache` - A mutable reference to the [`Cache`].
    /// * `tree` - A mutable reference to the [`Tree`](crate::Node::Tree).
    /// * `store` - A mutable reference to the [`Store`](crate::Node::Store).
    ///
    fn layout<C: Cache<Node = Self>>(
        &self,
        cache: &mut C,
        tree: &Self::Tree,
        store: &Self::Store,
    ) -> Size {
        let width = self
            .width(store)
            .map(|w| match w {
                SizeUnits::Pixels(px) => px,
                _ => panic!("Root node must have fixed size."),
            })
            .expect("Failed to get width for node");

        let height = self
            .height(store)
            .map(|w| match w {
                SizeUnits::Pixels(px) => px,
                _ => panic!("Root node must have fixed size."),
            })
            .expect("Failed to get height for node");

        cache.set_bounds(self, cache.posx(self), cache.posy(self), width, height);

        layout(self, LayoutType::Column, height, width, cache, tree, store)
    }

    /// Returns a key which can be used to set/get computed layout data from the [`cache`](crate::Cache).
    fn key(&self) -> Self::CacheKey;

    /// Returns an iterator over the children of the node.
    fn children<'t>(&'t self, tree: &'t Self::Tree) -> Self::ChildIter<'t>;

    /// Returns a boolean representing whether the node is visible to layout.
    fn visible(&self, store: &Self::Store) -> bool;

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
    fn margin_left(&self, store: &Self::Store) -> Option<SpaceUnits>;

    /// Returns the desired right-side space of the node.
    fn margin_right(&self, store: &Self::Store) -> Option<SpaceUnits>;

    /// Returns the desired top-side space of the node.
    fn margin_top(&self, store: &Self::Store) -> Option<SpaceUnits>;

    /// Returns the desired bottom-side space of the node.
    fn margin_bottom(&self, store: &Self::Store) -> Option<SpaceUnits>;

    /// Returns the desired left-side child-space of the node.
    fn padding_left(&self, store: &Self::Store) -> Option<SpaceUnits>;

    /// Returns the desired left-side child-space of the node.
    fn padding_right(&self, store: &Self::Store) -> Option<SpaceUnits>;

    /// Returns the desired left-side child-space of the node.
    fn padding_top(&self, store: &Self::Store) -> Option<SpaceUnits>;

    /// Returns the desired left-side child-space of the node.
    fn padding_bottom(&self, store: &Self::Store) -> Option<SpaceUnits>;

    /// Returns the desired space to applied between the children of the node on the vertical axis.
    fn row_between(&self, store: &Self::Store) -> Option<SpaceUnits>;

    /// Returns the desired space to be applied between the children of the node on the horizontal axis.
    fn col_between(&self, store: &Self::Store) -> Option<SpaceUnits>;

    /// Returns the solid override of the node.
    fn solid(&self, store: &Self::Store) -> Option<Solid>;

    /// Returns the aspect ratio of the node. (width / height)
    fn aspect_ratio(&self, store: &Self::Store) -> Option<f32>;

    /// Returns the horizontal alignment of the node
    fn halign(&self, store: &Self::Store) -> Option<Alignment>;

    fn valign(&self, store: &Self::Store) -> Option<Alignment>;
}

#[derive(Eq, PartialEq, Clone, Copy, Default)]
pub enum Alignment {
    Start,
    #[default]
    Center,
    End,
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

    fn margin_main_before(&self, store: &Self::Store, parent_layout_type: LayoutType) -> SpaceUnits {
        parent_layout_type.select_unwrap(store, |store| self.margin_left(store), |store| self.margin_top(store))
    }

    fn margin_main_after(&self, store: &Self::Store, parent_layout_type: LayoutType) -> SpaceUnits {
        parent_layout_type.select_unwrap(store, |store| self.margin_right(store), |store| self.margin_bottom(store))
    }

    fn margin_cross_before(&self, store: &Self::Store, parent_layout_type: LayoutType) -> SpaceUnits {
        parent_layout_type.select_unwrap(store, |store| self.margin_top(store), |store| self.margin_left(store))
    }

    fn margin_cross_after(&self, store: &Self::Store, parent_layout_type: LayoutType) -> SpaceUnits {
        parent_layout_type.select_unwrap(store, |store| self.margin_bottom(store), |store| self.margin_right(store))
    }

    fn padding_main_before(&self, store: &Self::Store, parent_layout_type: LayoutType) -> SpaceUnits {
        parent_layout_type.select_unwrap(store, |store| self.padding_left(store), |store| self.padding_top(store))
    }

    fn padding_main_after(&self, store: &Self::Store, parent_layout_type: LayoutType) -> SpaceUnits {
        parent_layout_type.select_unwrap(store, |store| self.padding_right(store), |store| self.padding_bottom(store))
    }

    fn padding_cross_before(&self, store: &Self::Store, parent_layout_type: LayoutType) -> SpaceUnits {
        parent_layout_type.select_unwrap(store, |store| self.padding_top(store), |store| self.padding_left(store))
    }

    fn padding_cross_after(&self, store: &Self::Store, parent_layout_type: LayoutType) -> SpaceUnits {
        parent_layout_type.select_unwrap(store, |store| self.padding_bottom(store), |store| self.padding_right(store))
    }

    fn main_between(&self, store: &Self::Store, parent_layout_type: LayoutType) -> SpaceUnits {
        parent_layout_type.select_unwrap(store, |store| self.col_between(store), |store| self.row_between(store))
    }

    // Currently unused until wrapping is implemented
    fn cross_between(&self, store: &Self::Store, parent_layout_type: LayoutType) -> SpaceUnits {
        parent_layout_type.select_unwrap(store, |store| self.row_between(store), |store| self.col_between(store))
    }
}

// Implement `NodeExt` for all types which implement `Node`.
impl<N: Node> NodeExt for N {}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Solid {
    Fit, // maximum axis uses aspect ratio
    Fill, // minimum axis uses aspect ratio
}