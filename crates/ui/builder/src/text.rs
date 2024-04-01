
use render_api::base::Color;
use ui_layout::{Alignment, MarginUnits, PositionType, SizeUnits};
use ui_builder_config::{NodeId, NodeStyle, StyleId, TextStyle, UiConfig, WidgetStyle};

pub struct TextMut<'a> {
    ui_config: &'a mut UiConfig,
    node_id: NodeId,
}

impl<'a> TextMut<'a> {
    pub(crate) fn new(ui_config: &'a mut UiConfig, panel_id: NodeId) -> Self {
        Self {
            ui_config,
            node_id: panel_id,
        }
    }

    pub fn set_style(&mut self, style_id: StyleId) -> &mut Self {
        let node = self.ui_config.node_mut(&self.node_id).unwrap();
        node.set_style_id(style_id);
        self
    }
}

pub struct TextStyleMut<'a> {
    ui_config: &'a mut UiConfig,
    style_id: StyleId,
}

impl<'a> TextStyleMut<'a> {
    pub(crate) fn new(ui_config: &'a mut UiConfig, style_id: StyleId) -> Self {
        Self { ui_config, style_id }
    }

    fn get_style_mut(&mut self) -> &mut NodeStyle {
        self.ui_config.style_mut(&self.style_id).unwrap()
    }

    fn get_text_style_mut(&mut self) -> &mut TextStyle {
        if let WidgetStyle::Text(text_style) = &mut self.get_style_mut().base.widget_style {
            text_style
        } else {
            panic!("StyleId does not reference a TextStyle");
        }
    }

    // setters
    pub fn set_parent_style(&mut self, style_id: StyleId) -> &mut Self {
        self.get_style_mut().parent_style = Some(style_id);
        self
    }

    pub fn set_background_color(&mut self, color: Color) -> &mut Self {
        self.get_text_style_mut().background_color = Some(color);
        self
    }

    pub fn set_background_alpha(&mut self, alpha: f32) -> &mut Self {
        self.get_text_style_mut().set_background_alpha(alpha);
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

    // set height
    fn set_height_units(&mut self, height: SizeUnits) -> &mut Self {
        self.get_style_mut().base.height = Some(height);
        self
    }

    // set_height_min
    fn set_height_min_units(&mut self, min_height: SizeUnits) -> &mut Self {
        self.get_style_mut().base.height_min = Some(min_height);
        self
    }

    // set_height_max
    fn set_height_max_units(&mut self, max_height: SizeUnits) -> &mut Self {
        self.get_style_mut().base.height_max = Some(max_height);
        self
    }

    // set size
    fn set_size_units(&mut self, height: SizeUnits) -> &mut Self {
        self.set_height_units(height);
        self
    }

    pub fn set_size_px(&mut self, height_px: f32) -> &mut Self {
        self.set_size_units(SizeUnits::Pixels(height_px));
        self
    }

    pub fn set_size_pc(&mut self, height_pc: f32) -> &mut Self {
        self.set_size_units(SizeUnits::Percentage(height_pc));
        self
    }

    pub fn set_size_vp(&mut self, height_vp: f32) -> &mut Self {
        self.set_size_units(SizeUnits::Viewport(height_vp));
        self
    }

    // set size min
    fn set_size_min_units(&mut self, height: SizeUnits) -> &mut Self {
        self.set_height_min_units(height);
        self
    }

    pub fn set_size_min_px(&mut self, height_px: f32) -> &mut Self {
        self.set_size_min_units(SizeUnits::Pixels(height_px));
        self
    }

    pub fn set_size_min_pc(&mut self, height_pc: f32) -> &mut Self {
        self.set_size_min_units(SizeUnits::Percentage(height_pc));
        self
    }

    pub fn set_size_min_vp(&mut self, height_vp: f32) -> &mut Self {
        self.set_size_min_units(SizeUnits::Viewport(height_vp));
        self
    }

    // set size max
    fn set_size_max_units(&mut self, height: SizeUnits) -> &mut Self {
        self.set_height_max_units(height);
        self
    }

    pub fn set_size_max_px(&mut self, height_px: f32) -> &mut Self {
        self.set_size_max_units(SizeUnits::Pixels(height_px));
        self
    }

    pub fn set_size_max_pc(&mut self, height_pc: f32) -> &mut Self {
        self.set_size_max_units(SizeUnits::Percentage(height_pc));
        self
    }

    pub fn set_size_max_vp(&mut self, height_vp: f32) -> &mut Self {
        self.set_size_max_units(SizeUnits::Viewport(height_vp));
        self
    }

    // set_left
    fn set_margin_left_units(&mut self, left: MarginUnits) -> &mut Self {
        self.get_style_mut().base.margin_left = Some(left);
        self
    }

    pub fn set_margin_left_px(&mut self, left_px: f32) -> &mut Self {
        self.set_margin_left_units(MarginUnits::Pixels(left_px))
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

    pub fn set_margin_right_px(&mut self, right_px: f32) -> &mut Self {
        self.set_margin_right_units(MarginUnits::Pixels(right_px))
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

    pub fn set_margin_top_px(&mut self, top_px: f32) -> &mut Self {
        self.set_margin_top_units(MarginUnits::Pixels(top_px))
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

    pub fn set_margin_bottom_px(&mut self, bottom_px: f32) -> &mut Self {
        self.set_margin_bottom_units(MarginUnits::Pixels(bottom_px))
    }

    pub fn set_margin_bottom_pc(&mut self, bottom_pc: f32) -> &mut Self {
        self.set_margin_bottom_units(MarginUnits::Percentage(bottom_pc))
    }

    pub fn set_margin_bottom_vp(&mut self, bottom_vp: f32) -> &mut Self {
        self.set_margin_bottom_units(MarginUnits::Viewport(bottom_vp))
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

    pub fn set_margin_vp(&mut self, left: f32, right: f32, top: f32, bottom: f32) -> &mut Self {
        self.set_margin_left_vp(left)
            .set_margin_right_vp(right)
            .set_margin_top_vp(top)
            .set_margin_bottom_vp(bottom)
    }
}
