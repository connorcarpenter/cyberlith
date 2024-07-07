use ui_builder_config::{NodeId, NodeStyle, StyleId, UiConfig};
use ui_layout::{Alignment, MarginUnits, PositionType, SizeUnits};

pub struct UiContainerMut<'a> {
    ui_config: &'a mut UiConfig,
    node_id: NodeId,
}

impl<'a> UiContainerMut<'a> {
    pub(crate) fn new(ui_config: &'a mut UiConfig, node_id: NodeId) -> Self {
        Self { ui_config, node_id }
    }

    pub fn set_style(&mut self, style_id: StyleId) -> &mut Self {
        let node = self.ui_config.node_mut(&self.node_id).unwrap();
        node.set_style_id(style_id);
        self
    }
}

pub struct UiContainerStyleMut<'a> {
    ui_config: &'a mut UiConfig,
    style_id: StyleId,
}

impl<'a> UiContainerStyleMut<'a> {
    pub fn new(ui_config: &'a mut UiConfig, style_id: StyleId) -> Self {
        Self {
            ui_config,
            style_id,
        }
    }

    fn get_style_mut(&mut self) -> &mut NodeStyle {
        self.ui_config.style_mut(&self.style_id).unwrap()
    }

    // setters

    pub fn set_id(&mut self, id_str: &str) -> &mut Self {
        self.get_style_mut().id_str = Some(id_str.to_string());
        self
    }

    pub fn set_parent_style(&mut self, style_id: StyleId) -> &mut Self {
        self.get_style_mut().parent_style = Some(style_id);
        self
    }

    pub fn set_absolute(&mut self) -> &mut Self {
        self.get_style_mut().base.position_type = Some(PositionType::Absolute);
        self
    }

    pub fn set_relative(&mut self) -> &mut Self {
        self.get_style_mut().base.position_type = Some(PositionType::Relative);
        self
    }

    pub fn set_self_halign(&mut self, align: Alignment) -> &mut Self {
        self.get_style_mut().base.self_halign = Some(align);
        self
    }

    pub fn set_self_valign(&mut self, align: Alignment) -> &mut Self {
        self.get_style_mut().base.self_valign = Some(align);
        self
    }

    // set_width
    fn set_width_units(&mut self, width: SizeUnits) -> &mut Self {
        self.get_style_mut().base.width = Some(width);
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

    pub fn set_width_vp(&mut self, width_vp: f32) -> &mut Self {
        self.set_width_units(SizeUnits::Viewport(width_vp))
    }

    // set height
    fn set_height_units(&mut self, height: SizeUnits) -> &mut Self {
        self.get_style_mut().base.height = Some(height);
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

    pub fn set_height_vp(&mut self, width_vp: f32) -> &mut Self {
        self.set_height_units(SizeUnits::Viewport(width_vp))
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

    pub fn set_size_vp(&mut self, width_vp: f32, height_vp: f32) -> &mut Self {
        self.set_size_units(
            SizeUnits::Viewport(width_vp),
            SizeUnits::Viewport(height_vp),
        )
    }

    // set_width_min
    fn set_width_min_units(&mut self, min_width: SizeUnits) -> &mut Self {
        self.get_style_mut().base.width_min = Some(min_width);
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

    pub fn set_width_min_vp(&mut self, min_width_vp: f32) -> &mut Self {
        self.set_width_min_units(SizeUnits::Viewport(min_width_vp))
    }

    // set_height_min
    fn set_height_min_units(&mut self, min_height: SizeUnits) -> &mut Self {
        self.get_style_mut().base.height_min = Some(min_height);
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

    pub fn set_height_min_vp(&mut self, min_height_vp: f32) -> &mut Self {
        self.set_height_min_units(SizeUnits::Viewport(min_height_vp))
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

    pub fn set_size_min_vp(&mut self, min_width_vp: f32, min_height_vp: f32) -> &mut Self {
        self.set_size_min_units(
            SizeUnits::Viewport(min_width_vp),
            SizeUnits::Viewport(min_height_vp),
        )
    }

    // set_width_max
    fn set_width_max_units(&mut self, max_width: SizeUnits) -> &mut Self {
        self.get_style_mut().base.width_max = Some(max_width);
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

    pub fn set_width_max_vp(&mut self, max_width_vp: f32) -> &mut Self {
        self.set_width_max_units(SizeUnits::Viewport(max_width_vp))
    }

    // set_height_max
    fn set_height_max_units(&mut self, max_height: SizeUnits) -> &mut Self {
        self.get_style_mut().base.height_max = Some(max_height);
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

    pub fn set_height_max_vp(&mut self, max_height_vp: f32) -> &mut Self {
        self.set_height_max_units(SizeUnits::Viewport(max_height_vp))
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

    pub fn set_size_max_vp(&mut self, max_width_vp: f32, max_height_vp: f32) -> &mut Self {
        self.set_size_max_units(
            SizeUnits::Viewport(max_width_vp),
            SizeUnits::Viewport(max_height_vp),
        )
    }

    // set_left
    fn set_margin_left_units(&mut self, left: MarginUnits) -> &mut Self {
        self.get_style_mut().base.margin_left = Some(left);
        self
    }

    pub fn set_margin_left_pc(&mut self, left_pc: f32) -> &mut Self {
        self.set_margin_left_units(MarginUnits::Percentage(left_pc))
    }

    pub fn set_margin_left_vp(&mut self, left_vp: f32) -> &mut Self {
        self.set_margin_left_units(MarginUnits::Viewport(left_vp))
    }

    // set_right
    fn set_margin_right_units(&mut self, right: MarginUnits) -> &mut Self {
        self.get_style_mut().base.margin_right = Some(right);
        self
    }

    pub fn set_margin_right_pc(&mut self, right_pc: f32) -> &mut Self {
        self.set_margin_right_units(MarginUnits::Percentage(right_pc))
    }

    pub fn set_margin_right_vp(&mut self, right_vp: f32) -> &mut Self {
        self.set_margin_right_units(MarginUnits::Viewport(right_vp))
    }

    // set_top
    fn set_margin_top_units(&mut self, top: MarginUnits) -> &mut Self {
        self.get_style_mut().base.margin_top = Some(top);
        self
    }

    pub fn set_margin_top_pc(&mut self, top_pc: f32) -> &mut Self {
        self.set_margin_top_units(MarginUnits::Percentage(top_pc))
    }

    pub fn set_margin_top_vp(&mut self, top_vp: f32) -> &mut Self {
        self.set_margin_top_units(MarginUnits::Viewport(top_vp))
    }

    // set_bottom
    fn set_margin_bottom_units(&mut self, bottom: MarginUnits) -> &mut Self {
        self.get_style_mut().base.margin_bottom = Some(bottom);
        self
    }

    pub fn set_margin_bottom_pc(&mut self, bottom_pc: f32) -> &mut Self {
        self.set_margin_bottom_units(MarginUnits::Percentage(bottom_pc))
    }

    pub fn set_margin_bottom_vp(&mut self, bottom_vp: f32) -> &mut Self {
        self.set_margin_bottom_units(MarginUnits::Viewport(bottom_vp))
    }

    // set_margin

    pub fn set_margin_pc(&mut self, left: f32, right: f32, top: f32, bottom: f32) -> &mut Self {
        self.set_margin_left_pc(left)
            .set_margin_right_pc(right)
            .set_margin_top_pc(top)
            .set_margin_bottom_pc(bottom)
    }

    pub fn set_margin_vp(&mut self, left: f32, right: f32, top: f32, bottom: f32) -> &mut Self {
        self.set_margin_left_vp(left)
            .set_margin_right_vp(right)
            .set_margin_top_vp(top)
            .set_margin_bottom_vp(bottom)
    }
}
