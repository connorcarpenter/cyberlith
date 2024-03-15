use std::any::Any;

use render_api::{
    components::{RenderLayer, Transform},
    resources::RenderFrame,
};
use ui_layout::{Alignment, MarginUnits, PositionType, SizeUnits, Solid};

use crate::{
    cache::LayoutCache,
    node::UiStore,
    style::{NodeStyle, StyleId},
    ui::Globals,
    widget::Widget,
    NodeId, Ui,
};

#[derive(Clone)]
pub struct Text {
    pub text: String,
}

impl Text {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
        }
    }

    pub fn inner_text(&self) -> &str {
        &self.text
    }
}

#[derive(Clone, Copy)]
pub struct TextStyle {}

impl TextStyle {
    pub fn empty() -> Self {
        Self {}
    }
}

pub struct TextMut<'a> {
    ui: &'a mut Ui,
    node_id: NodeId,
}

impl<'a> TextMut<'a> {
    pub(crate) fn new(ui: &'a mut Ui, panel_id: NodeId) -> Self {
        Self {
            ui,
            node_id: panel_id,
        }
    }

    pub fn set_visible(&mut self, visible: bool) -> &mut Self {
        if let Some(panel) = self.ui.node_mut(&self.node_id) {
            panel.visible = visible;
        }
        self
    }

    pub fn add_style(&mut self, style_id: StyleId) -> &mut Self {
        let node = self.ui.node_mut(&self.node_id).unwrap();
        node.style_ids.push(style_id);
        self
    }
}

pub struct TextStyleMut<'a> {
    ui: &'a mut Ui,
    style_id: StyleId,
}

impl<'a> TextStyleMut<'a> {
    pub(crate) fn new(ui: &'a mut Ui, style_id: StyleId) -> Self {
        Self { ui, style_id }
    }

    fn get_style_mut(&mut self) -> &mut NodeStyle {
        self.ui.style_mut(&self.style_id).unwrap()
    }

    pub fn set_absolute(&mut self) -> &mut Self {
        self.get_style_mut().position_type = Some(PositionType::Absolute);
        self
    }

    pub fn set_relative(&mut self) -> &mut Self {
        self.get_style_mut().position_type = Some(PositionType::Relative);
        self
    }

    pub fn set_self_halign(&mut self, align: Alignment) -> &mut Self {
        self.get_style_mut().self_halign = Some(align);
        self
    }

    pub fn set_self_valign(&mut self, align: Alignment) -> &mut Self {
        self.get_style_mut().self_valign = Some(align);
        self
    }

    // set_width
    fn set_width_units(&mut self, width: SizeUnits) -> &mut Self {
        self.get_style_mut().width = Some(width);
        self
    }

    pub fn set_width_auto(&mut self) -> &mut Self {
        self.set_width_units(SizeUnits::Auto)
    }

    pub fn set_width_px(&mut self, width_px: f32) -> &mut Self {
        self.set_width_units(SizeUnits::Pixels(width_px))
    }

    pub fn set_width_pc(&mut self, width_pc: f32) -> &mut Self {
        self.set_width_units(SizeUnits::Percentage(width_pc))
    }

    // set height
    fn set_height_units(&mut self, height: SizeUnits) -> &mut Self {
        self.get_style_mut().height = Some(height);
        self
    }

    pub fn set_height_auto(&mut self) -> &mut Self {
        self.set_height_units(SizeUnits::Auto)
    }

    pub fn set_height_px(&mut self, width_px: f32) -> &mut Self {
        self.set_height_units(SizeUnits::Pixels(width_px))
    }

    pub fn set_height_pc(&mut self, width_pc: f32) -> &mut Self {
        self.set_height_units(SizeUnits::Percentage(width_pc))
    }

    // set size
    fn set_size_units(&mut self, width: SizeUnits, height: SizeUnits) -> &mut Self {
        self.set_width_units(width);
        self.set_height_units(height);
        self
    }

    pub fn set_size_auto(&mut self) -> &mut Self {
        self.set_size_units(SizeUnits::Auto, SizeUnits::Auto)
    }

    pub fn set_size_px(&mut self, width_px: f32, height_px: f32) -> &mut Self {
        self.set_size_units(SizeUnits::Pixels(width_px), SizeUnits::Pixels(height_px))
    }

    pub fn set_size_pc(&mut self, width_pc: f32, height_pc: f32) -> &mut Self {
        self.set_size_units(
            SizeUnits::Percentage(width_pc),
            SizeUnits::Percentage(height_pc),
        )
    }

    // set_width_min
    fn set_width_min_units(&mut self, min_width: SizeUnits) -> &mut Self {
        self.get_style_mut().width_min = Some(min_width);
        self
    }

    pub fn set_width_min_auto(&mut self) -> &mut Self {
        self.set_width_min_units(SizeUnits::Auto)
    }

    pub fn set_width_min_px(&mut self, min_width_px: f32) -> &mut Self {
        self.set_width_min_units(SizeUnits::Pixels(min_width_px))
    }

    pub fn set_width_min_pc(&mut self, min_width_pc: f32) -> &mut Self {
        self.set_width_min_units(SizeUnits::Percentage(min_width_pc))
    }

    // set_height_min
    fn set_height_min_units(&mut self, min_height: SizeUnits) -> &mut Self {
        self.get_style_mut().height_min = Some(min_height);
        self
    }

    pub fn set_height_min_auto(&mut self) -> &mut Self {
        self.set_height_min_units(SizeUnits::Auto)
    }

    pub fn set_height_min_px(&mut self, min_height_px: f32) -> &mut Self {
        self.set_height_min_units(SizeUnits::Pixels(min_height_px))
    }

    pub fn set_height_min_pc(&mut self, min_height_pc: f32) -> &mut Self {
        self.set_height_min_units(SizeUnits::Percentage(min_height_pc))
    }

    // set_size_min
    fn set_size_min_units(&mut self, min_width: SizeUnits, min_height: SizeUnits) -> &mut Self {
        self.set_width_min_units(min_width);
        self.set_height_min_units(min_height);
        self
    }

    pub fn set_size_min_auto(&mut self) -> &mut Self {
        self.set_size_min_units(SizeUnits::Auto, SizeUnits::Auto)
    }

    pub fn set_size_min_px(&mut self, min_width_px: f32, min_height_px: f32) -> &mut Self {
        self.set_size_min_units(
            SizeUnits::Pixels(min_width_px),
            SizeUnits::Pixels(min_height_px),
        )
    }

    pub fn set_size_min_pc(&mut self, min_width_pc: f32, min_height_pc: f32) -> &mut Self {
        self.set_size_min_units(
            SizeUnits::Percentage(min_width_pc),
            SizeUnits::Percentage(min_height_pc),
        )
    }

    // set_width_max
    fn set_width_max_units(&mut self, max_width: SizeUnits) -> &mut Self {
        self.get_style_mut().width_max = Some(max_width);
        self
    }

    pub fn set_width_max_auto(&mut self) -> &mut Self {
        self.set_width_max_units(SizeUnits::Auto)
    }

    pub fn set_width_max_px(&mut self, max_width_px: f32) -> &mut Self {
        self.set_width_max_units(SizeUnits::Pixels(max_width_px))
    }

    pub fn set_width_max_pc(&mut self, max_width_pc: f32) -> &mut Self {
        self.set_width_max_units(SizeUnits::Percentage(max_width_pc))
    }

    // set_height_max
    fn set_height_max_units(&mut self, max_height: SizeUnits) -> &mut Self {
        self.get_style_mut().height_max = Some(max_height);
        self
    }

    pub fn set_height_max_auto(&mut self) -> &mut Self {
        self.set_height_max_units(SizeUnits::Auto)
    }

    pub fn set_height_max_px(&mut self, max_height_px: f32) -> &mut Self {
        self.set_height_max_units(SizeUnits::Pixels(max_height_px))
    }

    pub fn set_height_max_pc(&mut self, max_height_pc: f32) -> &mut Self {
        self.set_height_max_units(SizeUnits::Percentage(max_height_pc))
    }

    // set_size_max
    fn set_size_max_units(&mut self, max_width: SizeUnits, max_height: SizeUnits) -> &mut Self {
        self.set_width_max_units(max_width);
        self.set_height_max_units(max_height);
        self
    }

    pub fn set_size_max_auto(&mut self) -> &mut Self {
        self.set_size_max_units(SizeUnits::Auto, SizeUnits::Auto)
    }

    pub fn set_size_max_px(&mut self, max_width_px: f32, max_height_px: f32) -> &mut Self {
        self.set_size_max_units(
            SizeUnits::Pixels(max_width_px),
            SizeUnits::Pixels(max_height_px),
        )
    }

    pub fn set_size_max_pc(&mut self, max_width_pc: f32, max_height_pc: f32) -> &mut Self {
        self.set_size_max_units(
            SizeUnits::Percentage(max_width_pc),
            SizeUnits::Percentage(max_height_pc),
        )
    }

    // set_left
    fn set_margin_left_units(&mut self, left: MarginUnits) -> &mut Self {
        self.get_style_mut().margin_left = Some(left);
        self
    }

    pub fn set_margin_left_px(&mut self, left_px: f32) -> &mut Self {
        self.set_margin_left_units(MarginUnits::Pixels(left_px))
    }

    pub fn set_margin_left_pc(&mut self, left_pc: f32) -> &mut Self {
        self.set_margin_left_units(MarginUnits::Percentage(left_pc))
    }

    // set_right
    fn set_margin_right_units(&mut self, right: MarginUnits) -> &mut Self {
        self.get_style_mut().margin_right = Some(right);
        self
    }

    pub fn set_margin_right_px(&mut self, right_px: f32) -> &mut Self {
        self.set_margin_right_units(MarginUnits::Pixels(right_px))
    }

    pub fn set_margin_right_pc(&mut self, right_pc: f32) -> &mut Self {
        self.set_margin_right_units(MarginUnits::Percentage(right_pc))
    }

    // set_top
    fn set_margin_top_units(&mut self, top: MarginUnits) -> &mut Self {
        self.get_style_mut().margin_top = Some(top);
        self
    }

    pub fn set_margin_top_px(&mut self, top_px: f32) -> &mut Self {
        self.set_margin_top_units(MarginUnits::Pixels(top_px))
    }

    pub fn set_margin_top_pc(&mut self, top_pc: f32) -> &mut Self {
        self.set_margin_top_units(MarginUnits::Percentage(top_pc))
    }

    // set_bottom
    fn set_margin_bottom_units(&mut self, bottom: MarginUnits) -> &mut Self {
        self.get_style_mut().margin_bottom = Some(bottom);
        self
    }

    pub fn set_margin_bottom_px(&mut self, bottom_px: f32) -> &mut Self {
        self.set_margin_bottom_units(MarginUnits::Pixels(bottom_px))
    }

    pub fn set_margin_bottom_pc(&mut self, bottom_pc: f32) -> &mut Self {
        self.set_margin_bottom_units(MarginUnits::Percentage(bottom_pc))
    }

    // set_margin

    pub fn set_margin_px(&mut self, left: f32, right: f32, top: f32, bottom: f32) -> &mut Self {
        self.set_margin_left_px(left)
            .set_margin_right_px(right)
            .set_margin_top_px(top)
            .set_margin_bottom_px(bottom)
    }

    pub fn set_margin_pc(&mut self, left: f32, right: f32, top: f32, bottom: f32) -> &mut Self {
        self.set_margin_left_pc(left)
            .set_margin_right_pc(right)
            .set_margin_top_pc(top)
            .set_margin_bottom_pc(bottom)
    }

    // solid stuff

    pub fn set_solid_fit(&mut self) -> &mut Self {
        self.get_style_mut().solid_override = Some(Solid::Fit);
        self
    }

    pub fn set_solid_fill(&mut self) -> &mut Self {
        self.get_style_mut().solid_override = Some(Solid::Fill);
        self
    }

    pub fn set_aspect_ratio(&mut self, width: f32, height: f32) -> &mut Self {
        self.get_style_mut().set_aspect_ratio(width, height);
        self
    }
}
