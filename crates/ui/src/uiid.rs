use bevy_log::info;
use morphorm::{LayoutType, Node, PositionType, Units};

use crate::panel::PanelStore;

#[derive(Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash, Debug, Default)]
pub struct UiId(u32);

impl UiId {
    pub(crate) const fn new(id: u32) -> Self {
        UiId(id)
    }

    pub(crate) fn get(&self) -> u32 {
        self.0
    }

    pub(crate) fn increment(&mut self) {
        self.0 += 1;
    }
}

impl Node for UiId {
    type Store = PanelStore;
    type Tree = PanelStore;
    type ChildIter<'t> = std::slice::Iter<'t, UiId>;
    type CacheKey = Self;
    type SubLayout<'a> = ();

    fn key(&self) -> Self::CacheKey {
        *self
    }

    fn children<'t>(&'t self, ui: &'t PanelStore) -> Self::ChildIter<'t> {
        info!("getting children for {:?}", self);
        if let Some(panel) = ui.get(self) {
            info!("children: {:?}", panel.children);
            panel.children.iter()
        } else {
            info!("no children");
            [].iter()
        }
    }

    fn visible(&self, ui: &PanelStore) -> bool {
        if let Some(panel) = ui.get(self) {
            panel.visible
        } else {
            false
        }
    }

    fn layout_type(&self, ui: &PanelStore) -> Option<LayoutType> {
        let panel = ui.get(self)?;
        Some(panel.style.layout_type())
    }

    fn position_type(&self, ui: &PanelStore) -> Option<PositionType> {
        let panel = ui.get(self)?;
        Some(panel.style.position_type())
    }

    fn width(&self, ui: &PanelStore) -> Option<Units> {
        let panel = ui.get(self)?;
        Some(panel.style.width())
    }

    fn height(&self, ui: &PanelStore) -> Option<Units> {
        let panel = ui.get(self)?;
        Some(panel.style.height())
    }

    fn left(&self, ui: &PanelStore) -> Option<Units> {
        let panel = ui.get(self)?;
        Some(panel.style.margin_left())
    }

    fn right(&self, ui: &PanelStore) -> Option<Units> {
        let panel = ui.get(self)?;
        Some(panel.style.margin_right())
    }

    fn top(&self, ui: &PanelStore) -> Option<Units> {
        let panel = ui.get(self)?;
        Some(panel.style.margin_top())
    }

    fn bottom(&self, ui: &PanelStore) -> Option<Units> {
        let panel = ui.get(self)?;
        Some(panel.style.margin_bottom())
    }

    fn content_size(&self, ui: &PanelStore, sublayout: &mut Self::SubLayout<'_>, parent_width: Option<f32>, parent_height: Option<f32>) -> Option<(f32, f32)> {
        // TODO!
        None
    }

    fn child_left(&self, ui: &PanelStore) -> Option<Units> {
        let panel = ui.get(self)?;
        Some(panel.style.padding_left())
    }

    fn child_right(&self, ui: &PanelStore) -> Option<Units> {
        let panel = ui.get(self)?;
        Some(panel.style.padding_right())
    }

    fn child_top(&self, ui: &PanelStore) -> Option<Units> {
        let panel = ui.get(self)?;
        Some(panel.style.padding_top())
    }

    fn child_bottom(&self, ui: &PanelStore) -> Option<Units> {
        let panel = ui.get(self)?;
        Some(panel.style.padding_bottom())
    }

    fn row_between(&self, ui: &PanelStore) -> Option<Units> {
        let panel = ui.get(self)?;
        Some(panel.style.row_between())
    }

    fn col_between(&self, ui: &PanelStore) -> Option<Units> {
        let panel = ui.get(self)?;
        Some(panel.style.col_between())
    }

    fn min_width(&self, ui: &PanelStore) -> Option<Units> {
        let panel = ui.get(self)?;
        Some(panel.style.width_min())
    }

    fn min_height(&self, ui: &PanelStore) -> Option<Units> {
        let panel = ui.get(self)?;
        Some(panel.style.height_min())
    }

    fn max_width(&self, ui: &PanelStore) -> Option<Units> {
        let panel = ui.get(self)?;
        Some(panel.style.width_max())
    }

    fn max_height(&self, ui: &PanelStore) -> Option<Units> {
        let panel = ui.get(self)?;
        Some(panel.style.height_max())
    }

    fn min_left(&self, ui: &PanelStore) -> Option<Units> {
        let panel = ui.get(self)?;
        Some(panel.style.margin_left_min())
    }

    fn min_right(&self, ui: &PanelStore) -> Option<Units> {
        let panel = ui.get(self)?;
        Some(panel.style.margin_right_min())
    }

    fn min_top(&self, ui: &PanelStore) -> Option<Units> {
        let panel = ui.get(self)?;
        Some(panel.style.margin_top_min())
    }

    fn min_bottom(&self, ui: &PanelStore) -> Option<Units> {
        let panel = ui.get(self)?;
        Some(panel.style.margin_bottom_min())
    }

    fn max_left(&self, ui: &PanelStore) -> Option<Units> {
        let panel = ui.get(self)?;
        Some(panel.style.margin_left_max())
    }

    fn max_right(&self, ui: &PanelStore) -> Option<Units> {
        let panel = ui.get(self)?;
        Some(panel.style.margin_right_max())
    }

    fn max_top(&self, ui: &PanelStore) -> Option<Units> {
        let panel = ui.get(self)?;
        Some(panel.style.margin_top_max())
    }

    fn max_bottom(&self, ui: &PanelStore) -> Option<Units> {
        let panel = ui.get(self)?;
        Some(panel.style.margin_bottom_max())
    }

    fn border_left(&self, ui: &PanelStore) -> Option<Units> {
        let panel = ui.get(self)?;
        Some(panel.style.border_left())
    }

    fn border_right(&self, ui: &PanelStore) -> Option<Units> {
        let panel = ui.get(self)?;
        Some(panel.style.border_right())
    }

    fn border_top(&self, ui: &PanelStore) -> Option<Units> {
        let panel = ui.get(self)?;
        Some(panel.style.border_top())
    }

    fn border_bottom(&self, ui: &PanelStore) -> Option<Units> {
        let panel = ui.get(self)?;
        Some(panel.style.border_bottom())
    }
}
