use std::any::Any;

use morphorm::{PositionType, Units};

use asset_render::AssetManager;
use render_api::{base::{Color, CpuMaterial}, resources::RenderFrame, components::{RenderLayer, Transform}};
use storage::Handle;

use crate::{cache::LayoutCache, node::{UiNode, NodeStore}, style::NodeStyle, widget::Widget, ui::Globals, Ui, NodeId};

#[derive(Clone)]
pub struct Label {
    text: String,
    style: LabelStyle,
}

impl Label {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            style: LabelStyle::default(),
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

        // TODO: use some kind of text style from parent panel
        // TODO: text should fill the entire panel
        //let style = TextStyle::new(transform.scale.y, 6.0);

        // info!("Drawing label: {}", self.text);

        asset_manager.draw_text(
            render_frame,
            render_layer_opt,
            text_icon_handle,
            text_color_handle,
            &transform,
            &self.text,
        );
    }
}

#[derive(Clone, Default, Copy)]
pub struct LabelStyle {

}

pub struct LabelRef<'a> {
    ui: &'a Ui,
    node_id: NodeId,
}

impl<'a> LabelRef<'a> {
    pub(crate) fn new(ui: &'a Ui, node_id: NodeId) -> Self {
        Self { ui, node_id }
    }

    pub fn style(&self, inner_fn: impl FnOnce(LabelStyleRef)) -> &Self {
        let style_ref = LabelStyleRef::new(self.ui, self.node_id);
        inner_fn(style_ref);
        self
    }
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

pub struct LabelStyleRef<'a> {
    ui: &'a Ui,
    node_id: NodeId,
}

impl<'a> LabelStyleRef<'a> {
    pub(crate) fn new(ui: &'a Ui, node_id: NodeId) -> Self {
        Self { ui, node_id }
    }

    fn get_ref(&self) -> &UiNode {
        self.ui.node_ref(&self.node_id).unwrap()
    }

    fn get_label_ref(&self) -> &Label {
        self.get_ref().widget.as_ref().as_any().downcast_ref::<Label>().unwrap()
    }

    // getters

    pub fn background_color(&self) -> Color {
        self.get_ref().style.background_color
    }

    pub fn background_color_handle(&self) -> Option<Handle<CpuMaterial>> {
        self.get_ref().style.background_color_handle
    }

    pub fn position_type(&self) -> PositionType {
        self.get_ref().style.position_type
    }

    pub fn width(&self) -> Units {
        self.get_ref().style.width
    }

    pub fn height(&self) -> Units {
        self.get_ref().style.height
    }

    pub fn width_min(&self) -> Units {
        self.get_ref().style.width_min
    }

    pub fn width_max(&self) -> Units {
        self.get_ref().style.width_max
    }

    pub fn height_min(&self) -> Units {
        self.get_ref().style.height_min
    }

    pub fn height_max(&self) -> Units {
        self.get_ref().style.height_max
    }

    pub fn margin_left(&self) -> Units {
        self.get_ref().style.margin_left
    }

    pub fn margin_right(&self) -> Units {
        self.get_ref().style.margin_right
    }

    pub fn margin_top(&self) -> Units {
        self.get_ref().style.margin_top
    }

    pub fn margin_bottom(&self) -> Units {
        self.get_ref().style.margin_bottom
    }

    pub fn margin_left_min(&self) -> Units {
        self.get_ref().style.margin_left_min
    }

    pub fn margin_left_max(&self) -> Units {
        self.get_ref().style.margin_left_max
    }

    pub fn margin_right_min(&self) -> Units {
        self.get_ref().style.margin_right_min
    }

    pub fn margin_right_max(&self) -> Units {
        self.get_ref().style.margin_right_max
    }

    pub fn margin_top_min(&self) -> Units {
        self.get_ref().style.margin_top_min
    }

    pub fn margin_top_max(&self) -> Units {
        self.get_ref().style.margin_top_max
    }

    pub fn margin_bottom_min(&self) -> Units {
        self.get_ref().style.margin_bottom_min
    }

    pub fn margin_bottom_max(&self) -> Units {
        self.get_ref().style.margin_bottom_max
    }

    pub fn border_left(&self) -> Units {
        self.get_ref().style.border_left
    }

    pub fn border_right(&self) -> Units {
        self.get_ref().style.border_right
    }

    pub fn border_top(&self) -> Units {
        self.get_ref().style.border_top
    }

    pub fn border_bottom(&self) -> Units {
        self.get_ref().style.border_bottom
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

    fn get_label_ref(&self) -> &Label {
        self.get_ref().widget.as_ref().as_any().downcast_ref::<Label>().unwrap()
    }

    fn get_label_mut(&mut self) -> &mut Label {
        self.get_mut().widget.as_mut().as_any_mut().downcast_mut::<Label>().unwrap()
    }

    // getters

    pub fn background_color(&self) -> Color {
        self.get_ref().style.background_color
    }

    pub fn background_color_handle(&self) -> Option<Handle<CpuMaterial>> {
        self.get_ref().style.background_color_handle
    }

    pub fn position_type(&self) -> PositionType {
        self.get_ref().style.position_type
    }

    pub fn width(&self) -> Units {
        self.get_ref().style.width
    }

    pub fn height(&self) -> Units {
        self.get_ref().style.height
    }

    pub fn width_min(&self) -> Units {
        self.get_ref().style.width_min
    }

    pub fn width_max(&self) -> Units {
        self.get_ref().style.width_max
    }

    pub fn height_min(&self) -> Units {
        self.get_ref().style.height_min
    }

    pub fn height_max(&self) -> Units {
        self.get_ref().style.height_max
    }

    pub fn margin_left(&self) -> Units {
        self.get_ref().style.margin_left
    }

    pub fn margin_right(&self) -> Units {
        self.get_ref().style.margin_right
    }

    pub fn margin_top(&self) -> Units {
        self.get_ref().style.margin_top
    }

    pub fn margin_bottom(&self) -> Units {
        self.get_ref().style.margin_bottom
    }

    pub fn margin_left_min(&self) -> Units {
        self.get_ref().style.margin_left_min
    }

    pub fn margin_left_max(&self) -> Units {
        self.get_ref().style.margin_left_max
    }

    pub fn margin_right_min(&self) -> Units {
        self.get_ref().style.margin_right_min
    }

    pub fn margin_right_max(&self) -> Units {
        self.get_ref().style.margin_right_max
    }

    pub fn margin_top_min(&self) -> Units {
        self.get_ref().style.margin_top_min
    }

    pub fn margin_top_max(&self) -> Units {
        self.get_ref().style.margin_top_max
    }

    pub fn margin_bottom_min(&self) -> Units {
        self.get_ref().style.margin_bottom_min
    }

    pub fn margin_bottom_max(&self) -> Units {
        self.get_ref().style.margin_bottom_max
    }

    pub fn border_left(&self) -> Units {
        self.get_ref().style.border_left
    }

    pub fn border_right(&self) -> Units {
        self.get_ref().style.border_right
    }

    pub fn border_top(&self) -> Units {
        self.get_ref().style.border_top
    }

    pub fn border_bottom(&self) -> Units {
        self.get_ref().style.border_bottom
    }

    // setters

    pub fn set_background_color(&mut self, color: Color) -> &mut Self {
        let current_color = self.background_color();
        if color != current_color {
            self.get_mut().style.background_color = color;
            self.get_mut().style.background_color_handle = None;
        }
        self
    }

    pub fn set_background_color_handle(&mut self, handle: Handle<CpuMaterial>) -> &mut Self {
        self.get_mut().style.background_color_handle = Some(handle);
        self
    }

    pub fn set_absolute(&mut self) -> &mut Self {
        self.get_mut().style.position_type = PositionType::SelfDirected;
        self
    }

    pub fn set_relative(&mut self) -> &mut Self {
        self.get_mut().style.position_type = PositionType::ParentDirected;
        self
    }

    // set_width
    fn set_width_units(&mut self, width: Units) -> &mut Self {
        self.get_mut().style.width = width;
        self
    }

    pub fn set_width_auto(&mut self) -> &mut Self {
        self.set_width_units(Units::Auto)
    }

    pub fn set_width_px(&mut self, width_px: f32) -> &mut Self {
        self.set_width_units(Units::Pixels(width_px))
    }

    pub fn set_width_pc(&mut self, width_pc: f32) -> &mut Self {
        self.set_width_units(Units::Percentage(width_pc))
    }

    pub fn set_width_st(&mut self, stretch: f32) -> &mut Self {
        self.set_width_units(Units::Stretch(stretch))
    }

    // set height
    fn set_height_units(&mut self, height: Units) -> &mut Self {
        self.get_mut().style.height = height;
        self
    }

    pub fn set_height_auto(&mut self) -> &mut Self {
        self.set_height_units(Units::Auto)
    }

    pub fn set_height_px(&mut self, width_px: f32) -> &mut Self {
        self.set_height_units(Units::Pixels(width_px))
    }

    pub fn set_height_pc(&mut self, width_pc: f32) -> &mut Self {
        self.set_height_units(Units::Percentage(width_pc))
    }

    pub fn set_height_st(&mut self, stretch: f32) -> &mut Self {
        self.set_height_units(Units::Stretch(stretch))
    }

    // set size
    fn set_size_units(&mut self, width: Units, height: Units) -> &mut Self {
        self.set_width_units(width);
        self.set_height_units(height);
        self
    }

    pub fn set_size_auto(&mut self) -> &mut Self {
        self.set_size_units(Units::Auto, Units::Auto)
    }

    pub fn set_size_px(&mut self, width_px: f32, height_px: f32) -> &mut Self {
        self.set_size_units(Units::Pixels(width_px), Units::Pixels(height_px))
    }

    pub fn set_size_pc(&mut self, width_pc: f32, height_pc: f32) -> &mut Self {
        self.set_size_units(Units::Percentage(width_pc), Units::Percentage(height_pc))
    }

    pub fn set_size_st(&mut self, width_st: f32, height_st: f32) -> &mut Self {
        self.set_size_units(Units::Stretch(width_st), Units::Stretch(height_st))
    }

    // set_width_min
    fn set_width_min_units(&mut self, min_width: Units) -> &mut Self {
        self.get_mut().style.width_min = min_width;
        self
    }

    pub fn set_width_min_auto(&mut self) -> &mut Self {
        self.set_width_min_units(Units::Auto)
    }

    pub fn set_width_min_px(&mut self, min_width_px: f32) -> &mut Self {
        self.set_width_min_units(Units::Pixels(min_width_px))
    }

    pub fn set_width_min_pc(&mut self, min_width_pc: f32) -> &mut Self {
        self.set_width_min_units(Units::Percentage(min_width_pc))
    }

    pub fn set_width_min_st(&mut self, min_width_st: f32) -> &mut Self {
        self.set_width_min_units(Units::Stretch(min_width_st))
    }

    // set_height_min
    fn set_height_min_units(&mut self, min_height: Units) -> &mut Self {
        self.get_mut().style.height_min = min_height;
        self
    }

    pub fn set_height_min_auto(&mut self) -> &mut Self {
        self.set_height_min_units(Units::Auto)
    }

    pub fn set_height_min_px(&mut self, min_height_px: f32) -> &mut Self {
        self.set_height_min_units(Units::Pixels(min_height_px))
    }

    pub fn set_height_min_pc(&mut self, min_height_pc: f32) -> &mut Self {
        self.set_height_min_units(Units::Percentage(min_height_pc))
    }

    pub fn set_height_min_st(&mut self, min_height_st: f32) -> &mut Self {
        self.set_height_min_units(Units::Stretch(min_height_st))
    }

    // set_size_min
    fn set_size_min_units(&mut self, min_width: Units, min_height: Units) -> &mut Self {
        self.set_width_min_units(min_width);
        self.set_height_min_units(min_height);
        self
    }

    pub fn set_size_min_auto(&mut self) -> &mut Self {
        self.set_size_min_units(Units::Auto, Units::Auto)
    }

    pub fn set_size_min_px(&mut self, min_width_px: f32, min_height_px: f32) -> &mut Self {
        self.set_size_min_units(Units::Pixels(min_width_px), Units::Pixels(min_height_px))
    }

    pub fn set_size_min_pc(&mut self, min_width_pc: f32, min_height_pc: f32) -> &mut Self {
        self.set_size_min_units(
            Units::Percentage(min_width_pc),
            Units::Percentage(min_height_pc),
        )
    }

    pub fn set_size_min_st(&mut self, min_width_st: f32, min_height_st: f32) -> &mut Self {
        self.set_size_min_units(Units::Stretch(min_width_st), Units::Stretch(min_height_st))
    }

    // set_width_max
    fn set_width_max_units(&mut self, max_width: Units) -> &mut Self {
        self.get_mut().style.width_max = max_width;
        self
    }

    pub fn set_width_max_auto(&mut self) -> &mut Self {
        self.set_width_max_units(Units::Auto)
    }

    pub fn set_width_max_px(&mut self, max_width_px: f32) -> &mut Self {
        self.set_width_max_units(Units::Pixels(max_width_px))
    }

    pub fn set_width_max_pc(&mut self, max_width_pc: f32) -> &mut Self {
        self.set_width_max_units(Units::Percentage(max_width_pc))
    }

    pub fn set_width_max_st(&mut self, max_width_st: f32) -> &mut Self {
        self.set_width_max_units(Units::Stretch(max_width_st))
    }

    // set_height_max
    fn set_height_max_units(&mut self, max_height: Units) -> &mut Self {
        self.get_mut().style.height_max = max_height;
        self
    }

    pub fn set_height_max_auto(&mut self) -> &mut Self {
        self.set_height_max_units(Units::Auto)
    }

    pub fn set_height_max_px(&mut self, max_height_px: f32) -> &mut Self {
        self.set_height_max_units(Units::Pixels(max_height_px))
    }

    pub fn set_height_max_pc(&mut self, max_height_pc: f32) -> &mut Self {
        self.set_height_max_units(Units::Percentage(max_height_pc))
    }

    pub fn set_height_max_st(&mut self, max_height_st: f32) -> &mut Self {
        self.set_height_max_units(Units::Stretch(max_height_st))
    }

    // set_size_max
    fn set_size_max_units(&mut self, max_width: Units, max_height: Units) -> &mut Self {
        self.set_width_max_units(max_width);
        self.set_height_max_units(max_height);
        self
    }

    pub fn set_size_max_auto(&mut self) -> &mut Self {
        self.set_size_max_units(Units::Auto, Units::Auto)
    }

    pub fn set_size_max_px(&mut self, max_width_px: f32, max_height_px: f32) -> &mut Self {
        self.set_size_max_units(Units::Pixels(max_width_px), Units::Pixels(max_height_px))
    }

    pub fn set_size_max_pc(&mut self, max_width_pc: f32, max_height_pc: f32) -> &mut Self {
        self.set_size_max_units(
            Units::Percentage(max_width_pc),
            Units::Percentage(max_height_pc),
        )
    }

    pub fn set_size_max_st(&mut self, max_width_st: f32, max_height_st: f32) -> &mut Self {
        self.set_size_max_units(Units::Stretch(max_width_st), Units::Stretch(max_height_st))
    }

    // set_left
    fn set_margin_left_units(&mut self, left: Units) -> &mut Self {
        self.get_mut().style.margin_left = left;
        self
    }

    pub fn set_margin_left_auto(&mut self) -> &mut Self {
        self.set_margin_left_units(Units::Auto)
    }

    pub fn set_margin_left_px(&mut self, left_px: f32) -> &mut Self {
        self.set_margin_left_units(Units::Pixels(left_px))
    }

    pub fn set_margin_left_pc(&mut self, left_pc: f32) -> &mut Self {
        self.set_margin_left_units(Units::Percentage(left_pc))
    }

    pub fn set_margin_left_st(&mut self, left_st: f32) -> &mut Self {
        self.set_margin_left_units(Units::Stretch(left_st))
    }

    // set_right
    fn set_margin_right_units(&mut self, right: Units) -> &mut Self {
        self.get_mut().style.margin_right = right;
        self
    }

    pub fn set_margin_right_auto(&mut self) -> &mut Self {
        self.set_margin_right_units(Units::Auto)
    }

    pub fn set_margin_right_px(&mut self, right_px: f32) -> &mut Self {
        self.set_margin_right_units(Units::Pixels(right_px))
    }

    pub fn set_margin_right_pc(&mut self, right_pc: f32) -> &mut Self {
        self.set_margin_right_units(Units::Percentage(right_pc))
    }

    pub fn set_margin_right_st(&mut self, right_st: f32) -> &mut Self {
        self.set_margin_right_units(Units::Stretch(right_st))
    }

    // set_top
    fn set_margin_top_units(&mut self, top: Units) -> &mut Self {
        self.get_mut().style.margin_top = top;
        self
    }

    pub fn set_margin_top_auto(&mut self) -> &mut Self {
        self.set_margin_top_units(Units::Auto)
    }

    pub fn set_margin_top_px(&mut self, top_px: f32) -> &mut Self {
        self.set_margin_top_units(Units::Pixels(top_px))
    }

    pub fn set_margin_top_pc(&mut self, top_pc: f32) -> &mut Self {
        self.set_margin_top_units(Units::Percentage(top_pc))
    }

    pub fn set_margin_top_st(&mut self, top_st: f32) -> &mut Self {
        self.set_margin_top_units(Units::Stretch(top_st))
    }

    // set_bottom
    fn set_margin_bottom_units(&mut self, bottom: Units) -> &mut Self {
        self.get_mut().style.margin_bottom = bottom;
        self
    }

    pub fn set_margin_bottom_auto(&mut self) -> &mut Self {
        self.set_margin_bottom_units(Units::Auto)
    }

    pub fn set_margin_bottom_px(&mut self, bottom_px: f32) -> &mut Self {
        self.set_margin_bottom_units(Units::Pixels(bottom_px))
    }

    pub fn set_margin_bottom_pc(&mut self, bottom_pc: f32) -> &mut Self {
        self.set_margin_bottom_units(Units::Percentage(bottom_pc))
    }

    pub fn set_margin_bottom_st(&mut self, bottom_st: f32) -> &mut Self {
        self.set_margin_bottom_units(Units::Stretch(bottom_st))
    }

    // set_margin_left_min
    fn set_margin_left_min_units(&mut self, min_left: Units) -> &mut Self {
        self.get_mut().style.margin_left_min = min_left;
        self
    }

    pub fn set_margin_left_min_auto(&mut self) -> &mut Self {
        self.set_margin_left_min_units(Units::Auto)
    }

    pub fn set_margin_left_min_px(&mut self, min_left_px: f32) -> &mut Self {
        self.set_margin_left_min_units(Units::Pixels(min_left_px))
    }

    pub fn set_margin_left_min_pc(&mut self, min_left_pc: f32) -> &mut Self {
        self.set_margin_left_min_units(Units::Percentage(min_left_pc))
    }

    pub fn set_margin_left_min_st(&mut self, min_left_st: f32) -> &mut Self {
        self.set_margin_left_min_units(Units::Stretch(min_left_st))
    }

    // set_margin_right_min
    fn set_margin_right_min_units(&mut self, min_right: Units) -> &mut Self {
        self.get_mut().style.margin_right_min = min_right;
        self
    }

    pub fn set_margin_right_min_auto(&mut self) -> &mut Self {
        self.set_margin_right_min_units(Units::Auto)
    }

    pub fn set_margin_right_min_px(&mut self, min_right_px: f32) -> &mut Self {
        self.set_margin_right_min_units(Units::Pixels(min_right_px))
    }

    pub fn set_margin_right_min_pc(&mut self, min_right_pc: f32) -> &mut Self {
        self.set_margin_right_min_units(Units::Percentage(min_right_pc))
    }

    pub fn set_margin_right_min_st(&mut self, min_right_st: f32) -> &mut Self {
        self.set_margin_right_min_units(Units::Stretch(min_right_st))
    }

    // set_margin_top_min
    fn set_margin_top_min_units(&mut self, min_top: Units) -> &mut Self {
        self.get_mut().style.margin_top_min = min_top;
        self
    }

    pub fn set_margin_top_min_auto(&mut self) -> &mut Self {
        self.set_margin_top_min_units(Units::Auto)
    }

    pub fn set_margin_top_min_px(&mut self, min_top_px: f32) -> &mut Self {
        self.set_margin_top_min_units(Units::Pixels(min_top_px))
    }

    pub fn set_margin_top_min_pc(&mut self, min_top_pc: f32) -> &mut Self {
        self.set_margin_top_min_units(Units::Percentage(min_top_pc))
    }

    pub fn set_margin_top_min_st(&mut self, min_top_st: f32) -> &mut Self {
        self.set_margin_top_min_units(Units::Stretch(min_top_st))
    }

    // set_margin_bottom_min
    fn set_margin_bottom_min_units(&mut self, min_bottom: Units) -> &mut Self {
        self.get_mut().style.margin_bottom_min = min_bottom;
        self
    }

    pub fn set_margin_bottom_min_auto(&mut self) -> &mut Self {
        self.set_margin_bottom_min_units(Units::Auto)
    }

    pub fn set_margin_bottom_min_px(&mut self, min_bottom_px: f32) -> &mut Self {
        self.set_margin_bottom_min_units(Units::Pixels(min_bottom_px))
    }

    pub fn set_margin_bottom_min_pc(&mut self, min_bottom_pc: f32) -> &mut Self {
        self.set_margin_bottom_min_units(Units::Percentage(min_bottom_pc))
    }

    pub fn set_margin_bottom_min_st(&mut self, min_bottom_st: f32) -> &mut Self {
        self.set_margin_bottom_min_units(Units::Stretch(min_bottom_st))
    }

    // set_margin_left_max
    fn set_margin_left_max_units(&mut self, max_left: Units) -> &mut Self {
        self.get_mut().style.margin_left_max = max_left;
        self
    }

    pub fn set_margin_left_max_auto(&mut self) -> &mut Self {
        self.set_margin_left_max_units(Units::Auto)
    }

    pub fn set_margin_left_max_px(&mut self, max_left_px: f32) -> &mut Self {
        self.set_margin_left_max_units(Units::Pixels(max_left_px))
    }

    pub fn set_margin_left_max_pc(&mut self, max_left_pc: f32) -> &mut Self {
        self.set_margin_left_max_units(Units::Percentage(max_left_pc))
    }

    pub fn set_margin_left_max_st(&mut self, max_left_st: f32) -> &mut Self {
        self.set_margin_left_max_units(Units::Stretch(max_left_st))
    }

    // set_margin_right_max
    fn set_margin_right_max_units(&mut self, max_right: Units) -> &mut Self {
        self.get_mut().style.margin_right_max = max_right;
        self
    }

    pub fn set_margin_right_max_auto(&mut self) -> &mut Self {
        self.set_margin_right_max_units(Units::Auto)
    }

    pub fn set_margin_right_max_px(&mut self, max_right_px: f32) -> &mut Self {
        self.set_margin_right_max_units(Units::Pixels(max_right_px))
    }

    pub fn set_margin_right_max_pc(&mut self, max_right_pc: f32) -> &mut Self {
        self.set_margin_right_max_units(Units::Percentage(max_right_pc))
    }

    pub fn set_margin_right_max_st(&mut self, max_right_st: f32) -> &mut Self {
        self.set_margin_right_max_units(Units::Stretch(max_right_st))
    }

    // set_margin_top_max
    fn set_margin_top_max_units(&mut self, max_top: Units) -> &mut Self {
        self.get_mut().style.margin_top_max = max_top;
        self
    }

    pub fn set_margin_top_max_auto(&mut self) -> &mut Self {
        self.set_margin_top_max_units(Units::Auto)
    }

    pub fn set_margin_top_max_px(&mut self, max_top_px: f32) -> &mut Self {
        self.set_margin_top_max_units(Units::Pixels(max_top_px))
    }

    pub fn set_margin_top_max_pc(&mut self, max_top_pc: f32) -> &mut Self {
        self.set_margin_top_max_units(Units::Percentage(max_top_pc))
    }

    pub fn set_margin_top_max_st(&mut self, max_top_st: f32) -> &mut Self {
        self.set_margin_top_max_units(Units::Stretch(max_top_st))
    }

    // set_margin_bottom_max
    fn set_margin_bottom_max_units(&mut self, max_bottom: Units) -> &mut Self {
        self.get_mut().style.margin_bottom_max = max_bottom;
        self
    }

    pub fn set_margin_bottom_max_auto(&mut self) -> &mut Self {
        self.set_margin_bottom_max_units(Units::Auto)
    }

    pub fn set_margin_bottom_max_px(&mut self, max_bottom_px: f32) -> &mut Self {
        self.set_margin_bottom_max_units(Units::Pixels(max_bottom_px))
    }

    pub fn set_margin_bottom_max_pc(&mut self, max_bottom_pc: f32) -> &mut Self {
        self.set_margin_bottom_max_units(Units::Percentage(max_bottom_pc))
    }

    pub fn set_margin_bottom_max_st(&mut self, max_bottom_st: f32) -> &mut Self {
        self.set_margin_bottom_max_units(Units::Stretch(max_bottom_st))
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

    // set_border_left
    fn set_border_left_units(&mut self, border_left: Units) -> &mut Self {
        self.get_mut().style.border_left = border_left;
        self
    }

    pub fn set_border_left_auto(&mut self) -> &mut Self {
        self.set_border_left_units(Units::Auto)
    }

    pub fn set_border_left_px(&mut self, border_left_px: f32) -> &mut Self {
        self.set_border_left_units(Units::Pixels(border_left_px))
    }

    pub fn set_border_left_pc(&mut self, border_left_pc: f32) -> &mut Self {
        self.set_border_left_units(Units::Percentage(border_left_pc))
    }

    pub fn set_border_left_st(&mut self, border_left_st: f32) -> &mut Self {
        self.set_border_left_units(Units::Stretch(border_left_st))
    }

    // set_border_right
    fn set_border_right_units(&mut self, border_right: Units) -> &mut Self {
        self.get_mut().style.border_right = border_right;
        self
    }

    pub fn set_border_right_auto(&mut self) -> &mut Self {
        self.set_border_right_units(Units::Auto)
    }

    pub fn set_border_right_px(&mut self, border_right_px: f32) -> &mut Self {
        self.set_border_right_units(Units::Pixels(border_right_px))
    }

    pub fn set_border_right_pc(&mut self, border_right_pc: f32) -> &mut Self {
        self.set_border_right_units(Units::Percentage(border_right_pc))
    }

    pub fn set_border_right_st(&mut self, border_right_st: f32) -> &mut Self {
        self.set_border_right_units(Units::Stretch(border_right_st))
    }

    // set_border_top
    fn set_border_top_units(&mut self, border_top: Units) -> &mut Self {
        self.get_mut().style.border_top = border_top;
        self
    }

    pub fn set_border_top_auto(&mut self) -> &mut Self {
        self.set_border_top_units(Units::Auto)
    }

    pub fn set_border_top_px(&mut self, border_top_px: f32) -> &mut Self {
        self.set_border_top_units(Units::Pixels(border_top_px))
    }

    pub fn set_border_top_pc(&mut self, border_top_pc: f32) -> &mut Self {
        self.set_border_top_units(Units::Percentage(border_top_pc))
    }

    pub fn set_border_top_st(&mut self, border_top_st: f32) -> &mut Self {
        self.set_border_top_units(Units::Stretch(border_top_st))
    }

    // set_border_bottom
    fn set_border_bottom_units(&mut self, border_bottom: Units) -> &mut Self {
        self.get_mut().style.border_bottom = border_bottom;
        self
    }

    pub fn set_border_bottom_auto(&mut self) -> &mut Self {
        self.set_border_bottom_units(Units::Auto)
    }

    pub fn set_border_bottom_px(&mut self, border_bottom_px: f32) -> &mut Self {
        self.set_border_bottom_units(Units::Pixels(border_bottom_px))
    }

    pub fn set_border_bottom_pc(&mut self, border_bottom_pc: f32) -> &mut Self {
        self.set_border_bottom_units(Units::Percentage(border_bottom_pc))
    }

    pub fn set_border_bottom_st(&mut self, border_bottom_st: f32) -> &mut Self {
        self.set_border_bottom_units(Units::Stretch(border_bottom_st))
    }

    // set_border
    pub fn set_border_auto(&mut self) -> &mut Self {
        self.set_border_left_auto()
            .set_border_right_auto()
            .set_border_top_auto()
            .set_border_bottom_auto()
    }

    pub fn set_border_px(&mut self, left: f32, right: f32, top: f32, bottom: f32) -> &mut Self {
        self.set_border_left_px(left)
            .set_border_right_px(right)
            .set_border_top_px(top)
            .set_border_bottom_px(bottom)
    }

    pub fn set_border_pc(&mut self, left: f32, right: f32, top: f32, bottom: f32) -> &mut Self {
        self.set_border_left_pc(left)
            .set_border_right_pc(right)
            .set_border_top_pc(top)
            .set_border_bottom_pc(bottom)
    }

    pub fn set_border_st(&mut self, left: f32, right: f32, top: f32, bottom: f32) -> &mut Self {
        self.set_border_left_st(left)
            .set_border_right_st(right)
            .set_border_top_st(top)
            .set_border_bottom_st(bottom)
    }

    pub fn set_aspect_ratio_w_to_h(&mut self, val: f32) -> &mut Self {
        self.get_mut().style.aspect_ratio_w_to_h = val;
        self
    }
}