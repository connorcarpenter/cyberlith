use crate::{Alignment, LayoutType, MarginUnits, NodeId, NodeStore, SizeUnits};

/// used for converting layout properties into a direction-agnostic value.
impl NodeId {
    pub(crate) fn main(&self, store: &dyn NodeStore, parent_layout_type: LayoutType) -> SizeUnits {
        match parent_layout_type {
            LayoutType::Row => self.width(store),
            LayoutType::Column => self.height(store),
        }
    }

    pub(crate) fn main_min(
        &self,
        store: &dyn NodeStore,
        parent_layout_type: LayoutType,
    ) -> SizeUnits {
        parent_layout_type.select(
            store,
            |store| self.width_min(store),
            |store| self.height_min(store),
        )
    }

    pub(crate) fn main_max(
        &self,
        store: &dyn NodeStore,
        parent_layout_type: LayoutType,
    ) -> SizeUnits {
        parent_layout_type.select(
            store,
            |store| self.width_max(store),
            |store| self.height_max(store),
        )
    }

    pub(crate) fn cross(&self, store: &dyn NodeStore, parent_layout_type: LayoutType) -> SizeUnits {
        parent_layout_type.select(store, |store| self.height(store), |store| self.width(store))
    }

    pub(crate) fn cross_min(
        &self,
        store: &dyn NodeStore,
        parent_layout_type: LayoutType,
    ) -> SizeUnits {
        parent_layout_type.select(
            store,
            |store| self.height_min(store),
            |store| self.width_min(store),
        )
    }

    pub(crate) fn cross_max(
        &self,
        store: &dyn NodeStore,
        parent_layout_type: LayoutType,
    ) -> SizeUnits {
        parent_layout_type.select(
            store,
            |store| self.height_max(store),
            |store| self.width_max(store),
        )
    }

    pub(crate) fn margin_main_before(
        &self,
        store: &dyn NodeStore,
        parent_layout_type: LayoutType,
    ) -> MarginUnits {
        parent_layout_type.select(
            store,
            |store| self.margin_left(store),
            |store| self.margin_top(store),
        )
    }

    pub(crate) fn margin_main_after(
        &self,
        store: &dyn NodeStore,
        parent_layout_type: LayoutType,
    ) -> MarginUnits {
        parent_layout_type.select(
            store,
            |store| self.margin_right(store),
            |store| self.margin_bottom(store),
        )
    }

    pub(crate) fn margin_cross_before(
        &self,
        store: &dyn NodeStore,
        parent_layout_type: LayoutType,
    ) -> MarginUnits {
        parent_layout_type.select(
            store,
            |store| self.margin_top(store),
            |store| self.margin_left(store),
        )
    }

    pub(crate) fn margin_cross_after(
        &self,
        store: &dyn NodeStore,
        parent_layout_type: LayoutType,
    ) -> MarginUnits {
        parent_layout_type.select(
            store,
            |store| self.margin_bottom(store),
            |store| self.margin_right(store),
        )
    }

    pub(crate) fn padding_main_before(
        &self,
        store: &dyn NodeStore,
        parent_layout_type: LayoutType,
    ) -> SizeUnits {
        parent_layout_type.select(
            store,
            |store| self.padding_left(store),
            |store| self.padding_top(store),
        )
    }

    pub(crate) fn padding_main_after(
        &self,
        store: &dyn NodeStore,
        parent_layout_type: LayoutType,
    ) -> SizeUnits {
        parent_layout_type.select(
            store,
            |store| self.padding_right(store),
            |store| self.padding_bottom(store),
        )
    }

    pub(crate) fn padding_cross_before(
        &self,
        store: &dyn NodeStore,
        parent_layout_type: LayoutType,
    ) -> SizeUnits {
        parent_layout_type.select(
            store,
            |store| self.padding_top(store),
            |store| self.padding_left(store),
        )
    }

    pub(crate) fn padding_cross_after(
        &self,
        store: &dyn NodeStore,
        parent_layout_type: LayoutType,
    ) -> SizeUnits {
        parent_layout_type.select(
            store,
            |store| self.padding_bottom(store),
            |store| self.padding_right(store),
        )
    }

    pub(crate) fn main_between(
        &self,
        store: &dyn NodeStore,
        parent_layout_type: LayoutType,
    ) -> SizeUnits {
        parent_layout_type.select(
            store,
            |store| self.col_between(store),
            |store| self.row_between(store),
        )
    }

    pub(crate) fn self_align(
        &self,
        store: &dyn NodeStore,
        parent_layout_type: LayoutType,
    ) -> Alignment {
        parent_layout_type.select(
            store,
            |store| self.self_valign(store),
            |store| self.self_halign(store),
        )
    }

    pub(crate) fn children_align(
        &self,
        store: &dyn NodeStore,
        layout_type: LayoutType,
    ) -> Alignment {
        layout_type.select(
            store,
            |store| self.children_halign(store),
            |store| self.children_valign(store),
        )
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Solid {
    Fit,  // maximum axis uses aspect ratio
    Fill, // minimum axis uses aspect ratio
}
