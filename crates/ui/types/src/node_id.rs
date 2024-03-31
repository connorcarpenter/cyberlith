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

    // all of these unwrap_or_default
    fn layout_type(&self, store: &Self::Store) -> LayoutType {
        let mut output = LayoutType::default();

        if store.node_kind(self).has_children() {
            if let Some(panel_style) = store.panel_style(self) {
                if let Some(layout_type) = panel_style.layout_type {
                    output = layout_type;
                }
            }
        }

        output
    }

    // all of these unwrap_or_default
    fn position_type(&self, store: &Self::Store) -> PositionType {
        let mut output = PositionType::default();

        if let Some(node_style) = store.node_style(self) {
            if let Some(layout_type) = node_style.position_type {
                output = layout_type;
            }
        }

        output
    }

    // all of these unwrap_or(SizeUnits::Percentage(100.0))
    fn width(&self, store: &Self::Store) -> SizeUnits {

        if store.node_kind(self).is_text() {
            return SizeUnits::Auto;
        }

        let mut output = SizeUnits::Percentage(100.0);

        if let Some(node_style) = store.node_style(self) {
            if let Some(width) = node_style.width {
                output = width;
            }
        }

        output
    }

    // all of these unwrap_or(SizeUnits::Percentage(100.0))
    fn height(&self, store: &Self::Store) -> SizeUnits {
        if store.node_kind(self).is_text() {
            return SizeUnits::Auto;
        }

        let mut output = SizeUnits::Percentage(100.0);

        if let Some(node_style) = store.node_style(self) {
            if let Some(height) = node_style.height {
                output = height;
            }
        }

        output
    }

    // all of these unwrap_or(SizeUnits::Pixels(0.0))
    fn width_min(&self, store: &Self::Store) -> SizeUnits {
        if store.node_kind(self).is_text() {
            return SizeUnits::Auto;
        }

        let mut output = SizeUnits::Pixels(0.0);

        if let Some(node_style) = store.node_style(self) {
            if let Some(width_min) = node_style.width_min {
                output = width_min;
            }
        }

        output
    }

    // all of these unwrap_or(SizeUnits::Pixels(0.0))
    fn height_min(&self, store: &Self::Store) -> SizeUnits {
        if store.node_kind(self).is_text() {
            return SizeUnits::Auto;
        }

        let mut output = SizeUnits::Pixels(0.0);

        if let Some(node_style) = store.node_style(self) {
            if let Some(height_min) = node_style.height_min {
                output = height_min;
            }
        }

        output
    }

    // all of these unwrap_or(SizeUnits::Pixels(f32::MAX))
    fn width_max(&self, store: &Self::Store) -> SizeUnits {
        if store.node_kind(self).is_text() {
            return SizeUnits::Auto;
        }

        let mut output = SizeUnits::Pixels(f32::MAX);

        if let Some(node_style) = store.node_style(self) {
            if let Some(width_max) = node_style.width_max {
                output = width_max;
            }
        }

        output
    }

    // all of these unwrap_or(SizeUnits::Pixels(f32::MAX))
    fn height_max(&self, store: &Self::Store) -> SizeUnits {
        if store.node_kind(self).is_text() {
            return SizeUnits::Auto;
        }

        let mut output = SizeUnits::Pixels(f32::MAX);

        if let Some(node_style) = store.node_style(self) {
            if let Some(height_max) = node_style.height_max {
                output = height_max;
            }
        }

        output
    }

    // all of these unwrap_or_default
    fn margin_left(&self, store: &Self::Store) -> MarginUnits {
        let mut output = MarginUnits::default();

        if let Some(node_style) = store.node_style(self) {
            if let Some(margin_left) = node_style.margin_left {
                output = margin_left;
            }
        }

        output
    }

    // all of these unwrap_or_default
    fn margin_right(&self, store: &Self::Store) -> MarginUnits {
        let mut output = MarginUnits::default();

        if let Some(node_style) = store.node_style(self) {
            if let Some(margin_right) = node_style.margin_right {
                output = margin_right;
            }
        }

        output
    }

    // all of these unwrap_or_default
    fn margin_top(&self, store: &Self::Store) -> MarginUnits {
        let mut output = MarginUnits::default();

        if let Some(node_style) = store.node_style(self) {
            if let Some(margin_top) = node_style.margin_top {
                output = margin_top;
            }
        }

        output
    }

    // all of these unwrap_or_default
    fn margin_bottom(&self, store: &Self::Store) -> MarginUnits {
        let mut output = MarginUnits::default();

        if let Some(node_style) = store.node_style(self) {
            if let Some(margin_bottom) = node_style.margin_bottom {
                output = margin_bottom;
            }
        }

        output
    }

    // all of these unwrap_or_default
    fn padding_left(&self, store: &Self::Store) -> SizeUnits {
        let mut output = SizeUnits::default();

        if !store.node_kind(self).has_children() {
            return output;
        }

        if let Some(panel_style) = store.panel_style(self) {
            if let Some(padding_left) = panel_style.padding_left {
                output = padding_left;
            }
        }

        output
    }

    // all of these unwrap_or_default
    fn padding_right(&self, store: &Self::Store) -> SizeUnits {
        let mut output = SizeUnits::default();

        if !store.node_kind(self).has_children() {
            return output;
        }

        if let Some(panel_style) = store.panel_style(self) {
            if let Some(padding_right) = panel_style.padding_right {
                output = padding_right;
            }
        }

        output
    }

    // all of these unwrap_or_default
    fn padding_top(&self, store: &Self::Store) -> SizeUnits {
        let mut output = SizeUnits::default();

        if !store.node_kind(self).has_children() {
            return output;
        }

        if let Some(panel_style) = store.panel_style(self) {
            if let Some(padding_top) = panel_style.padding_top {
                output = padding_top;
            }
        }

        output
    }

    // all of these unwrap_or_default
    fn padding_bottom(&self, store: &Self::Store) -> SizeUnits {
        let mut output = SizeUnits::default();

        if !store.node_kind(self).has_children() {
            return output;
        }

        if let Some(panel_style) = store.panel_style(self) {
            if let Some(padding_bottom) = panel_style.padding_bottom {
                output = padding_bottom;
            }
        }

        output
    }

    // all of these unwrap_or_default
    fn row_between(&self, store: &Self::Store) -> SizeUnits {
        let mut output = SizeUnits::default();

        if !store.node_kind(self).has_children() {
            return output;
        }

        if let Some(panel_style) = store.panel_style(self) {
            if let Some(row_between) = panel_style.row_between {
                output = row_between;
            }
        }

        output
    }

    // all of these unwrap_or_default
    fn col_between(&self, store: &Self::Store) -> SizeUnits {
        let mut output = SizeUnits::default();

        if !store.node_kind(self).has_children() {
            return output;
        }

        if let Some(panel_style) = store.panel_style(self) {
            if let Some(col_between) = panel_style.col_between {
                output = col_between;
            }
        }

        output
    }

    // no default here .. None doesn't do anything, Some does
    fn is_solid(&self, store: &Self::Store) -> Option<Solid> {
        if !store.node_kind(self).can_solid() {
            return None;
        }

        store.node_style(self)?.solid_override
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

    // panics if solid() is None but this isn't ..
    fn aspect_ratio(&self, store: &Self::Store) -> Option<f32> {

        let mut output = 1.0; // TODO: put this into a constant

        if !store.node_kind(self).can_solid() {
            return Some(output);
        }

        if let Some(node_style) = store.node_style(self) {
            if let Some((w, h)) = node_style.aspect_ratio() {
                output = w / h;
            }
        }

        Some(output)
    }

    // all of these unwrap_or_default
    fn self_halign(&self, store: &Self::Store) -> Alignment {
        let mut output = Alignment::default();

        if let Some(node_style) = store.node_style(self) {
            if let Some(self_halign) = node_style.self_halign {
                output = self_halign;
            }
        }

        output
    }

    // all of these unwrap_or_default
    fn self_valign(&self, store: &Self::Store) -> Alignment {
        let mut output = Alignment::default();

        if let Some(node_style) = store.node_style(self) {
            if let Some(self_valign) = node_style.self_valign {
                output = self_valign;
            }
        }

        output
    }

    // all of these unwrap_or_default
    fn children_halign(&self, store: &Self::Store) -> Alignment {
        let mut output = Alignment::default();

        if !store.node_kind(self).has_children() {
            return output;
        }

        if let Some(panel_style) = store.panel_style(self) {
            if let Some(children_halign) = panel_style.children_halign {
                output = children_halign;
            }
        }

        output
    }

    // all of these unwrap_or_default
    fn children_valign(&self, store: &Self::Store) -> Alignment {
        let mut output = Alignment::default();

        if !store.node_kind(self).has_children() {
            return output;
        }

        if let Some(panel_style) = store.panel_style(self) {
            if let Some(children_valign) = panel_style.children_valign {
                output = children_valign;
            }
        }

        output
    }
}