use std::fmt::{Display, Formatter};

use ui_layout::{Alignment, LayoutType, MarginUnits, Node, PositionType, SizeUnits, Solid, TextMeasurer};

use crate::{Text, UiStore, UiVisibilityStore, WidgetKind};

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

impl Node for NodeId {
    type Store = UiStore;
    type StateStore = UiVisibilityStore;
    type ChildIter<'t> = std::slice::Iter<'t, Self>;
    type CacheKey = Self;

    fn key(&self) -> Self::CacheKey {
        *self
    }

    fn children<'t>(&'t self, store: &'t Self::Store) -> Self::ChildIter<'t> {
        if !store.node_kind(self).has_children() {
            return [].iter();
        }
        let node_ref = store.get_node(self).unwrap();
        let widget_kind = node_ref.widget_kind();
        match widget_kind {
            WidgetKind::Panel => {
                let panel_ref = store.panel_ref(self).unwrap();
                return panel_ref.children.iter();
            }
            WidgetKind::Button => {
                let button_ref = store.button_ref(self).unwrap();
                return button_ref.panel.children.iter();
            }
            _ => panic!("impossible"),
        }
    }

    fn visible(&self, store: &Self::StateStore) -> bool {
        store.get_node_visibility(self).unwrap()
    }

    fn layout_type(&self, store: &Self::Store) -> Option<LayoutType> {
        if !store.node_kind(self).has_children() {
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

    fn position_type(&self, store: &Self::Store) -> Option<PositionType> {
        let mut output = PositionType::default();

        store.for_each_node_style(self, |node_style| {
            if let Some(position_type) = node_style.position_type {
                output = position_type;
            }
        });

        Some(output)
    }

    fn width(&self, store: &Self::Store) -> Option<SizeUnits> {
        let mut output = SizeUnits::default();

        if store.node_kind(self).is_text() {
            return Some(output);
        }

        store.for_each_node_style(self, |node_style| {
            if let Some(width) = node_style.width {
                output = width;
            }
        });

        Some(output)
    }

    fn height(&self, store: &Self::Store) -> Option<SizeUnits> {
        let mut output = SizeUnits::default();

        store.for_each_node_style(self, |node_style| {
            if let Some(height) = node_style.height {
                output = height;
            }
        });

        Some(output)
    }

    fn width_min(&self, store: &Self::Store) -> Option<SizeUnits> {
        let mut output = SizeUnits::default();

        if store.node_kind(self).is_text() {
            return Some(output);
        }

        store.for_each_node_style(self, |node_style| {
            if let Some(width_min) = node_style.width_min {
                output = width_min;
            }
        });

        Some(output)
    }

    fn height_min(&self, store: &Self::Store) -> Option<SizeUnits> {
        let mut output = SizeUnits::default();

        if store.node_kind(self).is_text() {
            return Some(output);
        }

        store.for_each_node_style(self, |node_style| {
            if let Some(height_min) = node_style.height_min {
                output = height_min;
            }
        });

        Some(output)
    }

    fn width_max(&self, store: &Self::Store) -> Option<SizeUnits> {
        let mut output = SizeUnits::default();

        if store.node_kind(self).is_text() {
            return Some(output);
        }

        store.for_each_node_style(self, |node_style| {
            if let Some(width_max) = node_style.width_max {
                output = width_max;
            }
        });

        Some(output)
    }

    fn height_max(&self, store: &Self::Store) -> Option<SizeUnits> {
        let mut output = SizeUnits::default();

        if store.node_kind(self).is_text() {
            return Some(output);
        }

        store.for_each_node_style(self, |node_style| {
            if let Some(height_max) = node_style.height_max {
                output = height_max;
            }
        });

        Some(output)
    }

    fn margin_left(&self, store: &Self::Store) -> Option<MarginUnits> {
        let mut output = MarginUnits::default();

        store.for_each_node_style(self, |node_style| {
            if let Some(margin_left) = node_style.margin_left {
                output = margin_left;
            }
        });

        Some(output)
    }

    fn margin_right(&self, store: &Self::Store) -> Option<MarginUnits> {
        let mut output = MarginUnits::default();

        store.for_each_node_style(self, |node_style| {
            if let Some(margin_right) = node_style.margin_right {
                output = margin_right;
            }
        });

        Some(output)
    }

    fn margin_top(&self, store: &Self::Store) -> Option<MarginUnits> {
        let mut output = MarginUnits::default();

        store.for_each_node_style(self, |node_style| {
            if let Some(margin_top) = node_style.margin_top {
                output = margin_top;
            }
        });

        Some(output)
    }

    fn margin_bottom(&self, store: &Self::Store) -> Option<MarginUnits> {
        let mut output = MarginUnits::default();

        store.for_each_node_style(self, |node_style| {
            if let Some(margin_bottom) = node_style.margin_bottom {
                output = margin_bottom;
            }
        });

        Some(output)
    }

    fn padding_left(&self, store: &Self::Store) -> Option<SizeUnits> {
        if !store.node_kind(self).has_children() {
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

    fn padding_right(&self, store: &Self::Store) -> Option<SizeUnits> {
        if !store.node_kind(self).has_children() {
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

    fn padding_top(&self, store: &Self::Store) -> Option<SizeUnits> {
        if !store.node_kind(self).has_children() {
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

    fn padding_bottom(&self, store: &Self::Store) -> Option<SizeUnits> {
        if !store.node_kind(self).has_children() {
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

    fn row_between(&self, store: &Self::Store) -> Option<SizeUnits> {
        if !store.node_kind(self).has_children() {
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

    fn col_between(&self, store: &Self::Store) -> Option<SizeUnits> {
        if !store.node_kind(self).has_children() {
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

    fn is_solid(&self, store: &Self::Store) -> Option<Solid> {
        if !store.node_kind(self).can_solid() {
            return None;
        }

        let mut output = None;

        store.for_each_node_style(self, |node_style| {
            if let Some(solid) = node_style.solid_override {
                output = Some(solid);
            }
        });

        output
    }

    fn is_text(&self, store: &Self::Store) -> bool {
        store.node_kind(self).is_text()
    }

    fn calculate_text_width(&self, store: &Self::Store, text_measurer: &dyn TextMeasurer, height: f32) -> f32 {
        let text_ref = store.text_ref(self).unwrap();
        let text = text_ref.text.as_str();
        let (raw_width, raw_height) = Text::measure_raw_text_size(text_measurer, text);
        let scale = height / raw_height;
        raw_width * scale
    }

    fn aspect_ratio(&self, store: &Self::Store) -> Option<f32> {

        let mut output = 1.0; // TODO: put this into a constant

        if !store.node_kind(self).can_solid() {
            return Some(output);
        }

        store.for_each_node_style(self, |node_style| {
            if let Some((w, h)) = node_style.aspect_ratio() {
                output = w / h;
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
        if !store.node_kind(self).has_children() {
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
        if !store.node_kind(self).has_children() {
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