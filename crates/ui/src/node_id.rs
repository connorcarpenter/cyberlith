use std::fmt::{Display, Formatter};

use layout::{Alignment, LayoutType, MarginUnits, Node, PositionType, SizeUnits, Solid};

use crate::node::{UiStore, WidgetKind};

#[derive(Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash, Debug, Default)]
pub struct NodeId(u32);

impl NodeId {
    pub(crate) const fn new(id: u32) -> Self {
        Self(id)
    }

    pub(crate) fn as_usize(&self) -> usize {
        self.0 as usize
    }
}

impl Display for NodeId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Node for NodeId {
    type Store = UiStore;
    type Tree = UiStore;
    type ChildIter<'t> = std::slice::Iter<'t, Self>;
    type CacheKey = Self;
    type SubLayout<'a> = ();

    fn key(&self) -> Self::CacheKey {
        *self
    }

    fn children<'t>(&'t self, store: &'t UiStore) -> Self::ChildIter<'t> {
        if let Some(panel_ref) = store.panel_ref(self) {
            return panel_ref.children.iter();
        }

        return [].iter();
    }

    fn visible(&self, store: &UiStore) -> bool {
        if let Some(node) = store.get_node(self) {
            node.visible
        } else {
            false
        }
    }

    fn layout_type(&self, store: &UiStore) -> Option<LayoutType> {

        if store.node_kind(self) != WidgetKind::Panel {
            return None;
        }

        let mut output = LayoutType::default();

        store.for_each_panel_style(self, |panel_style| {
            if let Some(layout_type) = panel_style.layout_type {
                output = layout_type;
            }
        });

        Some(output)
    }

    fn position_type(&self, store: &UiStore) -> Option<PositionType> {
        let mut output = PositionType::default();

        store.for_each_node_style(self, |node_style| {
            if let Some(position_type) = node_style.position_type {
                output = position_type;
            }
        });

        Some(output)
    }

    fn width(&self, store: &UiStore) -> Option<SizeUnits> {
        let mut output = SizeUnits::default();

        store.for_each_node_style(self, |node_style| {
            if let Some(width) = node_style.width {
                output = width;
            }
        });

        Some(output)
    }

    fn height(&self, store: &UiStore) -> Option<SizeUnits> {
        let mut output = SizeUnits::default();

        store.for_each_node_style(self, |node_style| {
            if let Some(height) = node_style.height {
                output = height;
            }
        });

        Some(output)
    }

    fn width_min(&self, store: &UiStore) -> Option<SizeUnits> {
        let mut output = SizeUnits::default();

        store.for_each_node_style(self, |node_style| {
            if let Some(width_min) = node_style.width_min {
                output = width_min;
            }
        });

        Some(output)
    }

    fn height_min(&self, store: &UiStore) -> Option<SizeUnits> {
        let mut output = SizeUnits::default();

        store.for_each_node_style(self, |node_style| {
            if let Some(height_min) = node_style.height_min {
                output = height_min;
            }
        });

        Some(output)
    }

    fn width_max(&self, store: &UiStore) -> Option<SizeUnits> {
        let mut output = SizeUnits::default();

        store.for_each_node_style(self, |node_style| {
            if let Some(width_max) = node_style.width_max {
                output = width_max;
            }
        });

        Some(output)
    }

    fn height_max(&self, store: &UiStore) -> Option<SizeUnits> {
        let mut output = SizeUnits::default();

        store.for_each_node_style(self, |node_style| {
            if let Some(height_max) = node_style.height_max {
                output = height_max;
            }
        });

        Some(output)
    }

    fn margin_left(&self, store: &UiStore) -> Option<MarginUnits> {
        let mut output = MarginUnits::default();

        store.for_each_node_style(self, |node_style| {
            if let Some(margin_left) = node_style.margin_left {
                output = margin_left;
            }
        });

        Some(output)
    }

    fn margin_right(&self, store: &UiStore) -> Option<MarginUnits> {
        let mut output = MarginUnits::default();

        store.for_each_node_style(self, |node_style| {
            if let Some(margin_right) = node_style.margin_right {
                output = margin_right;
            }
        });

        Some(output)
    }

    fn margin_top(&self, store: &UiStore) -> Option<MarginUnits> {
        let mut output = MarginUnits::default();

        store.for_each_node_style(self, |node_style| {
            if let Some(margin_top) = node_style.margin_top {
                output = margin_top;
            }
        });

        Some(output)
    }

    fn margin_bottom(&self, store: &UiStore) -> Option<MarginUnits> {
        let mut output = MarginUnits::default();

        store.for_each_node_style(self, |node_style| {
            if let Some(margin_bottom) = node_style.margin_bottom {
                output = margin_bottom;
            }
        });

        Some(output)
    }

    fn padding_left(&self, store: &UiStore) -> Option<SizeUnits> {

        if store.node_kind(self) != WidgetKind::Panel {
            return None;
        }

        let mut output = SizeUnits::default();

        store.for_each_panel_style(self, |panel_style| {
            if let Some(padding_left) = panel_style.padding_left {
                output = padding_left;
            }
        });

        Some(output)
    }

    fn padding_right(&self, store: &UiStore) -> Option<SizeUnits> {

        if store.node_kind(self) != WidgetKind::Panel {
            return None;
        }

        let mut output = SizeUnits::default();

        store.for_each_panel_style(self, |panel_style| {
            if let Some(padding_right) = panel_style.padding_right {
                output = padding_right;
            }
        });

        Some(output)
    }

    fn padding_top(&self, store: &UiStore) -> Option<SizeUnits> {

        if store.node_kind(self) != WidgetKind::Panel {
            return None;
        }

        let mut output = SizeUnits::default();

        store.for_each_panel_style(self, |panel_style| {
            if let Some(padding_top) = panel_style.padding_top {
                output = padding_top;
            }
        });

        Some(output)
    }

    fn padding_bottom(&self, store: &UiStore) -> Option<SizeUnits> {

        if store.node_kind(self) != WidgetKind::Panel {
            return None;
        }

        let mut output = SizeUnits::default();

        store.for_each_panel_style(self, |panel_style| {
            if let Some(padding_bottom) = panel_style.padding_bottom {
                output = padding_bottom;
            }
        });

        Some(output)
    }

    fn row_between(&self, store: &UiStore) -> Option<SizeUnits> {

        if store.node_kind(self) != WidgetKind::Panel {
            return None;
        }

        let mut output = SizeUnits::default();

        store.for_each_panel_style(self, |panel_style| {
            if let Some(row_between) = panel_style.row_between {
                output = row_between;
            }
        });

        Some(output)
    }

    fn col_between(&self, store: &UiStore) -> Option<SizeUnits> {

        if store.node_kind(self) != WidgetKind::Panel {
            return None;
        }

        let mut output = SizeUnits::default();

        store.for_each_panel_style(self, |panel_style| {
            if let Some(col_between) = panel_style.col_between {
                output = col_between;
            }
        });

        Some(output)
    }

    fn solid(&self, store: &UiStore) -> Option<Solid> {
        let mut output = None;

        store.for_each_node_style(self, |node_style| {
            if let Some(solid) = node_style.solid_override {
                output = Some(solid);
            }
        });

        output
    }

    fn aspect_ratio(&self, store: &Self::Store) -> Option<f32> {
        let mut output = 1.0; // TODO: put this into a constant

        store.for_each_node_style(self, |node_style| {
            if let Some(aspect_ratio) = node_style.aspect_ratio_w_over_h {
                output = aspect_ratio;
            }
        });

        Some(output)
    }

    fn self_halign(&self, store: &Self::Store) -> Option<Alignment> {
        let mut output = Alignment::default();

        store.for_each_node_style(self, |node_style| {
            if let Some(self_halign) = node_style.self_halign {
                output = self_halign;
            }
        });

        Some(output)
    }

    fn self_valign(&self, store: &Self::Store) -> Option<Alignment> {
        let mut output = Alignment::default();

        store.for_each_node_style(self, |node_style| {
            if let Some(self_valign) = node_style.self_valign {
                output = self_valign;
            }
        });

        Some(output)
    }

    fn children_halign(&self, store: &Self::Store) -> Option<Alignment> {

        if store.node_kind(self) != WidgetKind::Panel {
            return None;
        }

        let mut output = Alignment::default();

        store.for_each_panel_style(self, |panel_style| {
            if let Some(children_halign) = panel_style.children_halign {
                output = children_halign;
            }
        });

        Some(output)
    }

    fn children_valign(&self, store: &Self::Store) -> Option<Alignment> {

        if store.node_kind(self) != WidgetKind::Panel {
            return None;
        }

        let mut output = Alignment::default();

        store.for_each_panel_style(self, |panel_style| {
            if let Some(children_valign) = panel_style.children_valign {
                output = children_valign;
            }
        });

        Some(output)
    }
}
