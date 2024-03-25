use render_api::base::{Color, CpuMaterial};
use storage::Handle;
use ui_layout::{Alignment, MarginUnits, PositionType, SizeUnits, TextMeasurer};

use crate::{store::UiStore, style::{NodeStyle, StyleId}, NodeId, Ui, WidgetStyle};

#[derive(Clone)]
pub struct Text {
    pub text: String,
    pub background_color_handle: Option<Handle<CpuMaterial>>,
}

impl Text {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            background_color_handle: None,
        }
    }

    pub fn inner_text(&self) -> &str {
        &self.text
    }

    pub fn measure_raw_text_width(
        text_measurer: &dyn TextMeasurer,
        text: &str,
    ) -> f32 {
        let mut width = 0.0;

        for c in text.chars() {
            if width > 0.0 {
                width += 6.0; // between character spacing - TODO: replace with config
            }

            let c: u8 = if c.is_ascii() {
                c as u8
            } else {
                42 // asterisk
            };
            let subimage_index = (c - 32) as usize;

            // get character width in order to move cursor appropriately
            let icon_width = text_measurer.get_raw_char_width(subimage_index);

            width += icon_width;
        }

        width
    }
}

#[derive(Clone, Copy)]
pub struct TextStyle {
    pub background_color: Option<Color>,
    pub(crate) background_alpha: Option<f32>,
}

impl TextStyle {
    pub fn empty() -> Self {
        Self {
            background_color: None,
            background_alpha: None,
        }
    }

    pub fn background_alpha(&self) -> Option<f32> {
        self.background_alpha
    }

    pub(crate) fn set_background_alpha(&mut self, val: f32) {
        // validate
        if val < 0.0 || val > 1.0 {
            panic!("background_alpha must be between 0.0 and 1.0");
        }
        if (val * 10.0).fract() != 0.0 {
            panic!("background_alpha must be a multiple of 0.1");
        }

        self.background_alpha = Some(val);
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

pub struct TextStyleRef<'a> {
    store: &'a UiStore,
    node_id: NodeId,
}

impl<'a> TextStyleRef<'a> {
    pub(crate) fn new(store: &'a UiStore, node_id: NodeId) -> Self {
        Self { store, node_id }
    }

    pub fn background_color(&self) -> Color {
        let mut output = Color::BLACK; // TODO: put into const var!

        self.store.for_each_text_style(&self.node_id, |style| {
            if let Some(color) = style.background_color {
                output = color;
            }
        });

        output
    }

    pub fn background_alpha(&self) -> f32 {
        let mut output = 0.0; // TODO: put into const var!

        self.store.for_each_text_style(&self.node_id, |style| {
            if let Some(alpha) = style.background_alpha {
                output = alpha;
            }
        });

        output
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

    fn get_text_style_mut(&mut self) -> &mut TextStyle {
        if let WidgetStyle::Text(text_style) = &mut self.get_style_mut().widget_style {
            text_style
        } else {
            panic!("StyleId does not reference a TextStyle");
        }
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

    // set height
    fn set_height_units(&mut self, height: SizeUnits) -> &mut Self {
        self.get_style_mut().height = Some(height);
        self
    }

    // set_height_min
    fn set_height_min_units(&mut self, min_height: SizeUnits) -> &mut Self {
        self.get_style_mut().height_min = Some(min_height);
        self
    }

    // set_height_max
    fn set_height_max_units(&mut self, max_height: SizeUnits) -> &mut Self {
        self.get_style_mut().height_max = Some(max_height);
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
        self.get_style_mut().margin_left = Some(left);
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
        self.get_style_mut().margin_right = Some(right);
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
        self.get_style_mut().margin_top = Some(top);
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
        self.get_style_mut().margin_bottom = Some(bottom);
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
