use std::any::Any;

use bevy_log::warn;

use morphorm::{LayoutType, PositionType, SizeUnits, Solid, SpaceUnits};

use asset_render::AssetManager;
use render_api::{base::{Color, CpuMaterial}, resources::RenderFrame, components::{RenderLayer, Transform}};
use storage::Handle;

use crate::{ui::draw_node, style::NodeStyle, node::{NodeKind, NodeStore, UiNode}, label::{Label, LabelMut}, cache::LayoutCache, widget::Widget, ui::Globals, NodeId, Ui};

#[derive(Clone)]
pub struct Panel {
    pub(crate) children: Vec<NodeId>,
    pub(crate) style: PanelStyle,
}

impl Panel {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            style: PanelStyle::default(),
        }
    }

    pub fn add_child(&mut self, child_id: NodeId) {
        self.children.push(child_id);
    }
}

impl Widget for Panel {
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
        cache: &LayoutCache,
        store: &NodeStore,
        _node_style: &NodeStyle,
        transform: &Transform
    ) {
        // draw panel
        let Some(mat_handle) = self.style.background_color_handle else {
            warn!("no color handle for panel"); // probably will need to do better debugging later
            return;
        };

        let box_handle = globals.get_box_mesh_handle().unwrap();
        render_frame.draw_mesh(render_layer_opt, box_handle, &mat_handle, &transform);

        for child_id in self.children.iter() {
            //info!("drawing child: {:?}", child);
            draw_node( // TODO: make this configurable?
                       render_frame,
                       render_layer_opt,
                       asset_manager,
                       globals,
                       cache,
                       store,
                       child_id,
                       (
                           transform.translation.x,
                           transform.translation.y,
                           transform.translation.z + 0.1
                       ),
            );
        }
    }
}

#[derive(Clone, Default, Copy)]
pub(crate) struct PanelStyle {

    pub(crate) background_color: Color,
    pub(crate) background_color_handle: Option<Handle<CpuMaterial>>,

    pub(crate) layout_type: LayoutType,

    pub(crate) padding_left: SpaceUnits,
    pub(crate) padding_right: SpaceUnits,
    pub(crate) padding_top: SpaceUnits,
    pub(crate) padding_bottom: SpaceUnits,

    pub(crate) row_between: SpaceUnits,
    pub(crate) col_between: SpaceUnits,
}

pub struct PanelMut<'a> {
    ui: &'a mut Ui,
    node_id: NodeId,
}

impl<'a> PanelMut<'a> {
    pub(crate) fn new(ui: &'a mut Ui, node_id: NodeId) -> Self {
        Self { ui, node_id }
    }

    pub fn set_visible(&mut self, visible: bool) -> &mut Self {
        if let Some(panel) = self.ui.node_mut(&self.node_id) {
            panel.visible = visible;
        }
        self
    }

    pub fn style(&mut self, inner_fn: impl FnOnce(&mut PanelStyleMut)) -> &mut Self {
        let mut style_mut = PanelStyleMut::new(self.ui, self.node_id);
        inner_fn(&mut style_mut);
        self
    }

    pub fn contents(&'a mut self, inner_fn: impl FnOnce(&mut PanelContentsMut)) -> &mut Self {
        let mut context = PanelContentsMut::new(self.ui, self.node_id);
        inner_fn(&mut context);
        self
    }
}

// only used for adding children
pub struct PanelContentsMut<'a> {
    ui: &'a mut Ui,
    node_id: NodeId,
}

impl<'a> PanelContentsMut<'a> {
    pub(crate) fn new(ui: &'a mut Ui, node_id: NodeId) -> Self {
        Self { ui, node_id }
    }

    fn get_mut(&mut self) -> &mut UiNode {
        self.ui.node_mut(&self.node_id).unwrap()
    }

    fn get_panel_mut(&mut self) -> &mut Panel {
        self.get_mut().widget.as_mut().as_any_mut().downcast_mut::<Panel>().unwrap()
    }

    pub fn add_panel<'b>(self: &'b mut PanelContentsMut<'a>) -> PanelMut<'b> {
        // creates a new panel, returning a context for it
        let new_id = self.ui.create_node(&NodeKind::Panel, Panel::new());

        // add new panel to children
        self.get_panel_mut().add_child(new_id);

        PanelMut::<'b>::new(self.ui, new_id)
    }

    pub fn add_label<'b>(self: &'b mut PanelContentsMut<'a>, text: &str) -> LabelMut<'b> {

        // creates a new panel, returning a context for it
        let new_id = self.ui.create_node(&NodeKind::Label, Label::new(text));

        // add new panel to children
        self.get_panel_mut().add_child(new_id);

        LabelMut::<'b>::new(self.ui, new_id)
    }
}

pub struct PanelStyleMut<'a> {
    ui: &'a mut Ui,
    node_id: NodeId,
}

impl<'a> PanelStyleMut<'a> {
    pub(crate) fn new(ui: &'a mut Ui, panel_id: NodeId) -> Self {
        Self { ui, node_id: panel_id }
    }

    fn get_ref(&self) -> &UiNode {
        self.ui.node_ref(&self.node_id).unwrap()
    }

    fn get_mut(&mut self) -> &mut UiNode {
        self.ui.node_mut(&self.node_id).unwrap()
    }

    fn get_panel_ref(&self) -> &Panel {
        self.get_ref().widget.as_ref().as_any().downcast_ref::<Panel>().unwrap()
    }

    fn get_panel_mut(&mut self) -> &mut Panel {
        self.get_mut().widget.as_mut().as_any_mut().downcast_mut::<Panel>().unwrap()
    }

    // getters

    pub fn background_color(&self) -> Color {
        self.get_panel_ref().style.background_color
    }

    pub fn background_color_handle(&self) -> Option<Handle<CpuMaterial>> {
        self.get_panel_ref().style.background_color_handle
    }

    pub fn layout_type(&self) -> LayoutType {
        self.get_panel_ref().style.layout_type
    }

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

    pub fn border_left(&self) -> SpaceUnits {
        self.get_ref().style.border_left
    }

    pub fn border_right(&self) -> SpaceUnits {
        self.get_ref().style.border_right
    }

    pub fn border_top(&self) -> SpaceUnits {
        self.get_ref().style.border_top
    }

    pub fn border_bottom(&self) -> SpaceUnits {
        self.get_ref().style.border_bottom
    }

    pub fn padding_left(&self) -> SpaceUnits {
        self.get_panel_ref().style.padding_left
    }

    pub fn padding_right(&self) -> SpaceUnits {
        self.get_panel_ref().style.padding_right
    }

    pub fn padding_top(&self) -> SpaceUnits {
        self.get_panel_ref().style.padding_top
    }

    pub fn padding_bottom(&self) -> SpaceUnits {
        self.get_panel_ref().style.padding_bottom
    }

    pub fn row_between(&self) -> SpaceUnits {
        self.get_panel_ref().style.row_between
    }

    pub fn col_between(&self) -> SpaceUnits {
        self.get_panel_ref().style.col_between
    }

    // setters

    pub fn set_background_color(&mut self, color: Color) -> &mut Self {
        let current_color = self.background_color();
        if color != current_color {
            self.get_panel_mut().style.background_color = color;
            self.get_panel_mut().style.background_color_handle = None;
        }
        self
    }

    pub fn set_background_color_handle(&mut self, handle: Handle<CpuMaterial>) -> &mut Self {
        self.get_panel_mut().style.background_color_handle = Some(handle);
        self
    }

    pub fn set_horizontal(&mut self) -> &mut Self {
        self.get_panel_mut().style.layout_type = LayoutType::Row;
        self
    }

    pub fn set_vertical(&mut self) -> &mut Self {
        self.get_panel_mut().style.layout_type = LayoutType::Column;
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

    // set_margin_min
    pub fn set_margin_min_auto(&mut self) -> &mut Self {
        self.set_margin_left_min_auto()
            .set_margin_right_min_auto()
            .set_margin_top_min_auto()
            .set_margin_bottom_min_auto()
    }

    pub fn set_margin_min_px(&mut self, left: f32, right: f32, top: f32, bottom: f32) -> &mut Self {
        self.set_margin_left_min_px(left)
            .set_margin_right_min_px(right)
            .set_margin_top_min_px(top)
            .set_margin_bottom_min_px(bottom)
    }

    pub fn set_margin_min_pc(&mut self, left: f32, right: f32, top: f32, bottom: f32) -> &mut Self {
        self.set_margin_left_min_pc(left)
            .set_margin_right_min_pc(right)
            .set_margin_top_min_pc(top)
            .set_margin_bottom_min_pc(bottom)
    }

    pub fn set_margin_min_st(&mut self, left: f32, right: f32, top: f32, bottom: f32) -> &mut Self {
        self.set_margin_left_min_st(left)
            .set_margin_right_min_st(right)
            .set_margin_top_min_st(top)
            .set_margin_bottom_min_st(bottom)
    }

    // set margin max
    pub fn set_margin_max_auto(&mut self) -> &mut Self {
        self.set_margin_left_max_auto()
            .set_margin_right_max_auto()
            .set_margin_top_max_auto()
            .set_margin_bottom_max_auto()
    }

    pub fn set_margin_max_px(&mut self, left: f32, right: f32, top: f32, bottom: f32) -> &mut Self {
        self.set_margin_left_max_px(left)
            .set_margin_right_max_px(right)
            .set_margin_top_max_px(top)
            .set_margin_bottom_max_px(bottom)
    }

    pub fn set_margin_max_pc(&mut self, left: f32, right: f32, top: f32, bottom: f32) -> &mut Self {
        self.set_margin_left_max_pc(left)
            .set_margin_right_max_pc(right)
            .set_margin_top_max_pc(top)
            .set_margin_bottom_max_pc(bottom)
    }

    pub fn set_margin_max_st(&mut self, left: f32, right: f32, top: f32, bottom: f32) -> &mut Self {
        self.set_margin_left_max_st(left)
            .set_margin_right_max_st(right)
            .set_margin_top_max_st(top)
            .set_margin_bottom_max_st(bottom)
    }

    // set_border_left
    fn set_border_left_units(&mut self, border_left: SpaceUnits) -> &mut Self {
        self.get_mut().style.border_left = border_left;
        self
    }

    pub fn set_border_left_auto(&mut self) -> &mut Self {
        self.set_border_left_units(SpaceUnits::Auto)
    }

    pub fn set_border_left_px(&mut self, border_left_px: f32) -> &mut Self {
        self.set_border_left_units(SpaceUnits::Pixels(border_left_px))
    }

    pub fn set_border_left_pc(&mut self, border_left_pc: f32) -> &mut Self {
        self.set_border_left_units(SpaceUnits::Percentage(border_left_pc))
    }

    pub fn set_border_left_st(&mut self, border_left_st: f32) -> &mut Self {
        self.set_border_left_units(SpaceUnits::Stretch(border_left_st))
    }

    // set_border_right
    fn set_border_right_units(&mut self, border_right: SpaceUnits) -> &mut Self {
        self.get_mut().style.border_right = border_right;
        self
    }

    pub fn set_border_right_auto(&mut self) -> &mut Self {
        self.set_border_right_units(SpaceUnits::Auto)
    }

    pub fn set_border_right_px(&mut self, border_right_px: f32) -> &mut Self {
        self.set_border_right_units(SpaceUnits::Pixels(border_right_px))
    }

    pub fn set_border_right_pc(&mut self, border_right_pc: f32) -> &mut Self {
        self.set_border_right_units(SpaceUnits::Percentage(border_right_pc))
    }

    pub fn set_border_right_st(&mut self, border_right_st: f32) -> &mut Self {
        self.set_border_right_units(SpaceUnits::Stretch(border_right_st))
    }

    // set_border_top
    fn set_border_top_units(&mut self, border_top: SpaceUnits) -> &mut Self {
        self.get_mut().style.border_top = border_top;
        self
    }

    pub fn set_border_top_auto(&mut self) -> &mut Self {
        self.set_border_top_units(SpaceUnits::Auto)
    }

    pub fn set_border_top_px(&mut self, border_top_px: f32) -> &mut Self {
        self.set_border_top_units(SpaceUnits::Pixels(border_top_px))
    }

    pub fn set_border_top_pc(&mut self, border_top_pc: f32) -> &mut Self {
        self.set_border_top_units(SpaceUnits::Percentage(border_top_pc))
    }

    pub fn set_border_top_st(&mut self, border_top_st: f32) -> &mut Self {
        self.set_border_top_units(SpaceUnits::Stretch(border_top_st))
    }

    // set_border_bottom
    fn set_border_bottom_units(&mut self, border_bottom: SpaceUnits) -> &mut Self {
        self.get_mut().style.border_bottom = border_bottom;
        self
    }

    pub fn set_border_bottom_auto(&mut self) -> &mut Self {
        self.set_border_bottom_units(SpaceUnits::Auto)
    }

    pub fn set_border_bottom_px(&mut self, border_bottom_px: f32) -> &mut Self {
        self.set_border_bottom_units(SpaceUnits::Pixels(border_bottom_px))
    }

    pub fn set_border_bottom_pc(&mut self, border_bottom_pc: f32) -> &mut Self {
        self.set_border_bottom_units(SpaceUnits::Percentage(border_bottom_pc))
    }

    pub fn set_border_bottom_st(&mut self, border_bottom_st: f32) -> &mut Self {
        self.set_border_bottom_units(SpaceUnits::Stretch(border_bottom_st))
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

    // set_padding_left
    fn set_padding_left_units(&mut self, child_left: SpaceUnits) -> &mut Self {
        self.get_panel_mut().style.padding_left = child_left;
        self
    }

    pub fn set_padding_left_auto(&mut self) -> &mut Self {
        self.set_padding_left_units(SpaceUnits::Auto)
    }

    pub fn set_padding_left_px(&mut self, child_left_px: f32) -> &mut Self {
        self.set_padding_left_units(SpaceUnits::Pixels(child_left_px))
    }

    pub fn set_padding_left_pc(&mut self, child_left_pc: f32) -> &mut Self {
        self.set_padding_left_units(SpaceUnits::Percentage(child_left_pc))
    }

    pub fn set_padding_left_st(&mut self, child_left_st: f32) -> &mut Self {
        self.set_padding_left_units(SpaceUnits::Stretch(child_left_st))
    }

    // set_padding_right
    fn set_padding_right_units(&mut self, child_right: SpaceUnits) -> &mut Self {
        self.get_panel_mut().style.padding_right = child_right;
        self
    }

    pub fn set_padding_right_auto(&mut self) -> &mut Self {
        self.set_padding_right_units(SpaceUnits::Auto)
    }

    pub fn set_padding_right_px(&mut self, child_right_px: f32) -> &mut Self {
        self.set_padding_right_units(SpaceUnits::Pixels(child_right_px))
    }

    pub fn set_padding_right_pc(&mut self, child_right_pc: f32) -> &mut Self {
        self.set_padding_right_units(SpaceUnits::Percentage(child_right_pc))
    }

    pub fn set_padding_right_st(&mut self, child_right_st: f32) -> &mut Self {
        self.set_padding_right_units(SpaceUnits::Stretch(child_right_st))
    }

    // set_padding_top
    fn set_padding_top_units(&mut self, child_top: SpaceUnits) -> &mut Self {
        self.get_panel_mut().style.padding_top = child_top;
        self
    }

    pub fn set_padding_top_auto(&mut self) -> &mut Self {
        self.set_padding_top_units(SpaceUnits::Auto)
    }

    pub fn set_padding_top_px(&mut self, child_top_px: f32) -> &mut Self {
        self.set_padding_top_units(SpaceUnits::Pixels(child_top_px))
    }

    pub fn set_padding_top_pc(&mut self, child_top_pc: f32) -> &mut Self {
        self.set_padding_top_units(SpaceUnits::Percentage(child_top_pc))
    }

    pub fn set_padding_top_st(&mut self, child_top_st: f32) -> &mut Self {
        self.set_padding_top_units(SpaceUnits::Stretch(child_top_st))
    }

    // set_padding_bottom
    fn set_padding_bottom_units(&mut self, child_bottom: SpaceUnits) -> &mut Self {
        self.get_panel_mut().style.padding_bottom = child_bottom;
        self
    }

    pub fn set_padding_bottom_auto(&mut self) -> &mut Self {
        self.set_padding_bottom_units(SpaceUnits::Auto)
    }

    pub fn set_padding_bottom_px(&mut self, child_bottom_px: f32) -> &mut Self {
        self.set_padding_bottom_units(SpaceUnits::Pixels(child_bottom_px))
    }

    pub fn set_padding_bottom_pc(&mut self, child_bottom_pc: f32) -> &mut Self {
        self.set_padding_bottom_units(SpaceUnits::Percentage(child_bottom_pc))
    }

    pub fn set_padding_bottom_st(&mut self, child_bottom_st: f32) -> &mut Self {
        self.set_padding_bottom_units(SpaceUnits::Stretch(child_bottom_st))
    }

    // set_padding
    pub fn set_padding_auto(&mut self) -> &mut Self {
        self.set_padding_left_auto()
            .set_padding_right_auto()
            .set_padding_top_auto()
            .set_padding_bottom_auto()
    }

    pub fn set_padding_px(&mut self, left: f32, right: f32, top: f32, bottom: f32) -> &mut Self {
        self.set_padding_left_px(left)
            .set_padding_right_px(right)
            .set_padding_top_px(top)
            .set_padding_bottom_px(bottom)
    }

    pub fn set_padding_pc(&mut self, left: f32, right: f32, top: f32, bottom: f32) -> &mut Self {
        self.set_padding_left_pc(left)
            .set_padding_right_pc(right)
            .set_padding_top_pc(top)
            .set_padding_bottom_pc(bottom)
    }

    pub fn set_padding_st(&mut self, left: f32, right: f32, top: f32, bottom: f32) -> &mut Self {
        self.set_padding_left_st(left)
            .set_padding_right_st(right)
            .set_padding_top_st(top)
            .set_padding_bottom_st(bottom)
    }

    // set_row_between
    fn set_row_between_units(&mut self, row_between: SpaceUnits) -> &mut Self {
        self.get_panel_mut().style.row_between = row_between;
        self
    }

    pub fn set_row_between_auto(&mut self) -> &mut Self {
        self.set_row_between_units(SpaceUnits::Auto)
    }

    pub fn set_row_between_px(&mut self, row_between_px: f32) -> &mut Self {
        self.set_row_between_units(SpaceUnits::Pixels(row_between_px))
    }

    pub fn set_row_between_pc(&mut self, row_between_pc: f32) -> &mut Self {
        self.set_row_between_units(SpaceUnits::Percentage(row_between_pc))
    }

    pub fn set_row_between_st(&mut self, row_between_st: f32) -> &mut Self {
        self.set_row_between_units(SpaceUnits::Stretch(row_between_st))
    }

    // set_col_between
    fn set_col_between_units(&mut self, column_between: SpaceUnits) -> &mut Self {
        self.get_panel_mut().style.col_between = column_between;
        self
    }

    pub fn set_col_between_auto(&mut self) -> &mut Self {
        self.set_col_between_units(SpaceUnits::Auto)
    }

    pub fn set_col_between_px(&mut self, column_between_px: f32) -> &mut Self {
        self.set_col_between_units(SpaceUnits::Pixels(column_between_px))
    }

    pub fn set_col_between_pc(&mut self, column_between_pc: f32) -> &mut Self {
        self.set_col_between_units(SpaceUnits::Percentage(column_between_pc))
    }

    pub fn set_col_between_st(&mut self, column_between_st: f32) -> &mut Self {
        self.set_col_between_units(SpaceUnits::Stretch(column_between_st))
    }

    // solid stuff

    pub fn set_solid_fit(&mut self) -> &mut Self {
        self.get_mut().style.solid_override = Some(Solid::Fit);
        self
    }

    pub fn set_solid_fill(&mut self) -> &mut Self {
        self.get_mut().style.solid_override = Some(Solid::Fill);
        self
    }

    pub fn set_aspect_ratio(&mut self, width: f32, height: f32) -> &mut Self {
        self.get_mut().style.aspect_ratio_w_over_h = width / height;
        self
    }
}