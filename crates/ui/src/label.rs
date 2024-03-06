use std::any::Any;

use morphorm::{PositionType, SizeUnits, SpaceUnits};

use asset_render::AssetManager;
use render_api::{resources::RenderFrame, components::{RenderLayer, Transform}};

use crate::{cache::LayoutCache, node::{UiNode, NodeStore}, style::NodeStyle, widget::Widget, ui::Globals, Ui, NodeId};

#[derive(Clone)]
pub struct Label {
    text: String,
    _style: LabelStyle,
}

impl Label {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            _style: LabelStyle::default(),
        }
    }
}

impl Widget for Label {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn draw(
        &self,
        render_frame: &mut RenderFrame,
        render_layer_opt: Option<&RenderLayer>,
        asset_manager: &AssetManager,
        globals: &Globals,
        _cache: &LayoutCache,
        _store: &NodeStore,
        _node_style: &NodeStyle,
        transform: &Transform
    ) {
        let Some(text_icon_handle) = globals.get_text_icon_handle() else {
            panic!("No text handle found in globals");
        };
        let Some(text_color_handle) = globals.get_text_color_handle() else {
            panic!("No text color handle found in globals");
        };

        asset_manager.draw_text(
            render_frame,
            render_layer_opt,
            text_icon_handle,
            text_color_handle,
            transform,
            &self.text,
        );
    }
}

#[derive(Clone, Default, Copy)]
pub struct LabelStyle {

}

pub struct LabelMut<'a> {
    ui: &'a mut Ui,
    node_id: NodeId,
}

impl<'a> LabelMut<'a> {
    pub(crate) fn new(ui: &'a mut Ui, panel_id: NodeId) -> Self {
        Self { ui, node_id: panel_id }
    }

    pub fn set_visible(&mut self, visible: bool) -> &mut Self {
        if let Some(panel) = self.ui.node_mut(&self.node_id) {
            panel.visible = visible;
        }
        self
    }

    pub fn style(&mut self, inner_fn: impl FnOnce(&mut LabelStyleMut)) -> &mut Self {
        let mut style_mut = LabelStyleMut::new(self.ui, self.node_id);
        inner_fn(&mut style_mut);
        self
    }
}

pub struct LabelStyleMut<'a> {
    ui: &'a mut Ui,
    node_id: NodeId,
}

impl<'a> LabelStyleMut<'a> {
    pub(crate) fn new(ui: &'a mut Ui, node_id: NodeId) -> Self {
        Self { ui, node_id }
    }

    fn get_ref(&self) -> &UiNode {
        self.ui.node_ref(&self.node_id).unwrap()
    }

    fn get_mut(&mut self) -> &mut UiNode {
        self.ui.node_mut(&self.node_id).unwrap()
    }

    // fn get_label_ref(&self) -> &Label {
    //     self.get_ref().widget.as_ref().as_any().downcast_ref::<Label>().unwrap()
    // }
    //
    // fn get_label_mut(&mut self) -> &mut Label {
    //     self.get_mut().widget.as_mut().as_any_mut().downcast_mut::<Label>().unwrap()
    // }

    // getters

    pub fn position_type(&self) -> PositionType {
        self.get_ref().style.position_type
    }

    pub fn width(&self) -> SizeUnits {
        self.get_ref().style.width
    }

    pub fn height(&self) -> SizeUnits {
        self.get_ref().style.height
    }

    pub fn width_min(&self) -> SizeUnits {
        self.get_ref().style.width_min
    }

    pub fn width_max(&self) -> SizeUnits {
        self.get_ref().style.width_max
    }

    pub fn height_min(&self) -> SizeUnits {
        self.get_ref().style.height_min
    }

    pub fn height_max(&self) -> SizeUnits {
        self.get_ref().style.height_max
    }

    pub fn margin_left(&self) -> SpaceUnits {
        self.get_ref().style.margin_left
    }

    pub fn margin_right(&self) -> SpaceUnits {
        self.get_ref().style.margin_right
    }

    pub fn margin_top(&self) -> SpaceUnits {
        self.get_ref().style.margin_top
    }

    pub fn margin_bottom(&self) -> SpaceUnits {
        self.get_ref().style.margin_bottom
    }

    pub fn margin_left_min(&self) -> SpaceUnits {
        self.get_ref().style.margin_left_min
    }

    pub fn margin_left_max(&self) -> SpaceUnits {
        self.get_ref().style.margin_left_max
    }

    pub fn margin_right_min(&self) -> SpaceUnits {
        self.get_ref().style.margin_right_min
    }

    pub fn margin_right_max(&self) -> SpaceUnits {
        self.get_ref().style.margin_right_max
    }

    pub fn margin_top_min(&self) -> SpaceUnits {
        self.get_ref().style.margin_top_min
    }

    pub fn margin_top_max(&self) -> SpaceUnits {
        self.get_ref().style.margin_top_max
    }

    pub fn margin_bottom_min(&self) -> SpaceUnits {
        self.get_ref().style.margin_bottom_min
    }

    pub fn margin_bottom_max(&self) -> SpaceUnits {
        self.get_ref().style.margin_bottom_max
    }

    // setters

    pub fn set_absolute(&mut self) -> &mut Self {
        self.get_mut().style.position_type = PositionType::Absolute;
        self
    }

    pub fn set_relative(&mut self) -> &mut Self {
        self.get_mut().style.position_type = PositionType::Relative;
        self
    }

    // set_width
    fn set_width_units(&mut self, width: SizeUnits) -> &mut Self {
        self.get_mut().style.width = width;
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
        self.get_mut().style.height = height;
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
        self.set_size_units(SizeUnits::Percentage(width_pc), SizeUnits::Percentage(height_pc))
    }

    // set_width_min
    fn set_width_min_units(&mut self, min_width: SizeUnits) -> &mut Self {
        self.get_mut().style.width_min = min_width;
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
        self.get_mut().style.height_min = min_height;
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
        self.set_size_min_units(SizeUnits::Pixels(min_width_px), SizeUnits::Pixels(min_height_px))
    }

    pub fn set_size_min_pc(&mut self, min_width_pc: f32, min_height_pc: f32) -> &mut Self {
        self.set_size_min_units(
            SizeUnits::Percentage(min_width_pc),
            SizeUnits::Percentage(min_height_pc),
        )
    }

    // set_width_max
    fn set_width_max_units(&mut self, max_width: SizeUnits) -> &mut Self {
        self.get_mut().style.width_max = max_width;
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
        self.get_mut().style.height_max = max_height;
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
        self.set_size_max_units(SizeUnits::Pixels(max_width_px), SizeUnits::Pixels(max_height_px))
    }

    pub fn set_size_max_pc(&mut self, max_width_pc: f32, max_height_pc: f32) -> &mut Self {
        self.set_size_max_units(
            SizeUnits::Percentage(max_width_pc),
            SizeUnits::Percentage(max_height_pc),
        )
    }

    // set_left
    fn set_margin_left_units(&mut self, left: SpaceUnits) -> &mut Self {
        self.get_mut().style.margin_left = left;
        self
    }

    pub fn set_margin_left_auto(&mut self) -> &mut Self {
        self.set_margin_left_units(SpaceUnits::Auto)
    }

    pub fn set_margin_left_px(&mut self, left_px: f32) -> &mut Self {
        self.set_margin_left_units(SpaceUnits::Pixels(left_px))
    }

    pub fn set_margin_left_pc(&mut self, left_pc: f32) -> &mut Self {
        self.set_margin_left_units(SpaceUnits::Percentage(left_pc))
    }

    pub fn set_margin_left_st(&mut self, left_st: f32) -> &mut Self {
        self.set_margin_left_units(SpaceUnits::Stretch(left_st))
    }

    // set_right
    fn set_margin_right_units(&mut self, right: SpaceUnits) -> &mut Self {
        self.get_mut().style.margin_right = right;
        self
    }

    pub fn set_margin_right_auto(&mut self) -> &mut Self {
        self.set_margin_right_units(SpaceUnits::Auto)
    }

    pub fn set_margin_right_px(&mut self, right_px: f32) -> &mut Self {
        self.set_margin_right_units(SpaceUnits::Pixels(right_px))
    }

    pub fn set_margin_right_pc(&mut self, right_pc: f32) -> &mut Self {
        self.set_margin_right_units(SpaceUnits::Percentage(right_pc))
    }

    pub fn set_margin_right_st(&mut self, right_st: f32) -> &mut Self {
        self.set_margin_right_units(SpaceUnits::Stretch(right_st))
    }

    // set_top
    fn set_margin_top_units(&mut self, top: SpaceUnits) -> &mut Self {
        self.get_mut().style.margin_top = top;
        self
    }

    pub fn set_margin_top_auto(&mut self) -> &mut Self {
        self.set_margin_top_units(SpaceUnits::Auto)
    }

    pub fn set_margin_top_px(&mut self, top_px: f32) -> &mut Self {
        self.set_margin_top_units(SpaceUnits::Pixels(top_px))
    }

    pub fn set_margin_top_pc(&mut self, top_pc: f32) -> &mut Self {
        self.set_margin_top_units(SpaceUnits::Percentage(top_pc))
    }

    pub fn set_margin_top_st(&mut self, top_st: f32) -> &mut Self {
        self.set_margin_top_units(SpaceUnits::Stretch(top_st))
    }

    // set_bottom
    fn set_margin_bottom_units(&mut self, bottom: SpaceUnits) -> &mut Self {
        self.get_mut().style.margin_bottom = bottom;
        self
    }

    pub fn set_margin_bottom_auto(&mut self) -> &mut Self {
        self.set_margin_bottom_units(SpaceUnits::Auto)
    }

    pub fn set_margin_bottom_px(&mut self, bottom_px: f32) -> &mut Self {
        self.set_margin_bottom_units(SpaceUnits::Pixels(bottom_px))
    }

    pub fn set_margin_bottom_pc(&mut self, bottom_pc: f32) -> &mut Self {
        self.set_margin_bottom_units(SpaceUnits::Percentage(bottom_pc))
    }

    pub fn set_margin_bottom_st(&mut self, bottom_st: f32) -> &mut Self {
        self.set_margin_bottom_units(SpaceUnits::Stretch(bottom_st))
    }

    // set_margin_left_min
    fn set_margin_left_min_units(&mut self, min_left: SpaceUnits) -> &mut Self {
        self.get_mut().style.margin_left_min = min_left;
        self
    }

    pub fn set_margin_left_min_auto(&mut self) -> &mut Self {
        self.set_margin_left_min_units(SpaceUnits::Auto)
    }

    pub fn set_margin_left_min_px(&mut self, min_left_px: f32) -> &mut Self {
        self.set_margin_left_min_units(SpaceUnits::Pixels(min_left_px))
    }

    pub fn set_margin_left_min_pc(&mut self, min_left_pc: f32) -> &mut Self {
        self.set_margin_left_min_units(SpaceUnits::Percentage(min_left_pc))
    }

    pub fn set_margin_left_min_st(&mut self, min_left_st: f32) -> &mut Self {
        self.set_margin_left_min_units(SpaceUnits::Stretch(min_left_st))
    }

    // set_margin_right_min
    fn set_margin_right_min_units(&mut self, min_right: SpaceUnits) -> &mut Self {
        self.get_mut().style.margin_right_min = min_right;
        self
    }

    pub fn set_margin_right_min_auto(&mut self) -> &mut Self {
        self.set_margin_right_min_units(SpaceUnits::Auto)
    }

    pub fn set_margin_right_min_px(&mut self, min_right_px: f32) -> &mut Self {
        self.set_margin_right_min_units(SpaceUnits::Pixels(min_right_px))
    }

    pub fn set_margin_right_min_pc(&mut self, min_right_pc: f32) -> &mut Self {
        self.set_margin_right_min_units(SpaceUnits::Percentage(min_right_pc))
    }

    pub fn set_margin_right_min_st(&mut self, min_right_st: f32) -> &mut Self {
        self.set_margin_right_min_units(SpaceUnits::Stretch(min_right_st))
    }

    // set_margin_top_min
    fn set_margin_top_min_units(&mut self, min_top: SpaceUnits) -> &mut Self {
        self.get_mut().style.margin_top_min = min_top;
        self
    }

    pub fn set_margin_top_min_auto(&mut self) -> &mut Self {
        self.set_margin_top_min_units(SpaceUnits::Auto)
    }

    pub fn set_margin_top_min_px(&mut self, min_top_px: f32) -> &mut Self {
        self.set_margin_top_min_units(SpaceUnits::Pixels(min_top_px))
    }

    pub fn set_margin_top_min_pc(&mut self, min_top_pc: f32) -> &mut Self {
        self.set_margin_top_min_units(SpaceUnits::Percentage(min_top_pc))
    }

    pub fn set_margin_top_min_st(&mut self, min_top_st: f32) -> &mut Self {
        self.set_margin_top_min_units(SpaceUnits::Stretch(min_top_st))
    }

    // set_margin_bottom_min
    fn set_margin_bottom_min_units(&mut self, min_bottom: SpaceUnits) -> &mut Self {
        self.get_mut().style.margin_bottom_min = min_bottom;
        self
    }

    pub fn set_margin_bottom_min_auto(&mut self) -> &mut Self {
        self.set_margin_bottom_min_units(SpaceUnits::Auto)
    }

    pub fn set_margin_bottom_min_px(&mut self, min_bottom_px: f32) -> &mut Self {
        self.set_margin_bottom_min_units(SpaceUnits::Pixels(min_bottom_px))
    }

    pub fn set_margin_bottom_min_pc(&mut self, min_bottom_pc: f32) -> &mut Self {
        self.set_margin_bottom_min_units(SpaceUnits::Percentage(min_bottom_pc))
    }

    pub fn set_margin_bottom_min_st(&mut self, min_bottom_st: f32) -> &mut Self {
        self.set_margin_bottom_min_units(SpaceUnits::Stretch(min_bottom_st))
    }

    // set_margin_left_max
    fn set_margin_left_max_units(&mut self, max_left: SpaceUnits) -> &mut Self {
        self.get_mut().style.margin_left_max = max_left;
        self
    }

    pub fn set_margin_left_max_auto(&mut self) -> &mut Self {
        self.set_margin_left_max_units(SpaceUnits::Auto)
    }

    pub fn set_margin_left_max_px(&mut self, max_left_px: f32) -> &mut Self {
        self.set_margin_left_max_units(SpaceUnits::Pixels(max_left_px))
    }

    pub fn set_margin_left_max_pc(&mut self, max_left_pc: f32) -> &mut Self {
        self.set_margin_left_max_units(SpaceUnits::Percentage(max_left_pc))
    }

    pub fn set_margin_left_max_st(&mut self, max_left_st: f32) -> &mut Self {
        self.set_margin_left_max_units(SpaceUnits::Stretch(max_left_st))
    }

    // set_margin_right_max
    fn set_margin_right_max_units(&mut self, max_right: SpaceUnits) -> &mut Self {
        self.get_mut().style.margin_right_max = max_right;
        self
    }

    pub fn set_margin_right_max_auto(&mut self) -> &mut Self {
        self.set_margin_right_max_units(SpaceUnits::Auto)
    }

    pub fn set_margin_right_max_px(&mut self, max_right_px: f32) -> &mut Self {
        self.set_margin_right_max_units(SpaceUnits::Pixels(max_right_px))
    }

    pub fn set_margin_right_max_pc(&mut self, max_right_pc: f32) -> &mut Self {
        self.set_margin_right_max_units(SpaceUnits::Percentage(max_right_pc))
    }

    pub fn set_margin_right_max_st(&mut self, max_right_st: f32) -> &mut Self {
        self.set_margin_right_max_units(SpaceUnits::Stretch(max_right_st))
    }

    // set_margin_top_max
    fn set_margin_top_max_units(&mut self, max_top: SpaceUnits) -> &mut Self {
        self.get_mut().style.margin_top_max = max_top;
        self
    }

    pub fn set_margin_top_max_auto(&mut self) -> &mut Self {
        self.set_margin_top_max_units(SpaceUnits::Auto)
    }

    pub fn set_margin_top_max_px(&mut self, max_top_px: f32) -> &mut Self {
        self.set_margin_top_max_units(SpaceUnits::Pixels(max_top_px))
    }

    pub fn set_margin_top_max_pc(&mut self, max_top_pc: f32) -> &mut Self {
        self.set_margin_top_max_units(SpaceUnits::Percentage(max_top_pc))
    }

    pub fn set_margin_top_max_st(&mut self, max_top_st: f32) -> &mut Self {
        self.set_margin_top_max_units(SpaceUnits::Stretch(max_top_st))
    }

    // set_margin_bottom_max
    fn set_margin_bottom_max_units(&mut self, max_bottom: SpaceUnits) -> &mut Self {
        self.get_mut().style.margin_bottom_max = max_bottom;
        self
    }

    pub fn set_margin_bottom_max_auto(&mut self) -> &mut Self {
        self.set_margin_bottom_max_units(SpaceUnits::Auto)
    }

    pub fn set_margin_bottom_max_px(&mut self, max_bottom_px: f32) -> &mut Self {
        self.set_margin_bottom_max_units(SpaceUnits::Pixels(max_bottom_px))
    }

    pub fn set_margin_bottom_max_pc(&mut self, max_bottom_pc: f32) -> &mut Self {
        self.set_margin_bottom_max_units(SpaceUnits::Percentage(max_bottom_pc))
    }

    pub fn set_margin_bottom_max_st(&mut self, max_bottom_st: f32) -> &mut Self {
        self.set_margin_bottom_max_units(SpaceUnits::Stretch(max_bottom_st))
    }

    // set_margin
    pub fn set_margin_auto(&mut self) -> &mut Self {
        self.set_margin_left_auto()
            .set_margin_right_auto()
            .set_margin_top_auto()
            .set_margin_bottom_auto()
    }

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

    pub fn set_margin_st(&mut self, left: f32, right: f32, top: f32, bottom: f32) -> &mut Self {
        self.set_margin_left_st(left)
            .set_margin_right_st(right)
            .set_margin_top_st(top)
            .set_margin_bottom_st(bottom)
    }
}