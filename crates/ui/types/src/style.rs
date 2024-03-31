use ui_layout::{Alignment, MarginUnits, PositionType, SizeUnits, Solid};

use crate::{panel::PanelStyle, text::TextStyle, ButtonStyle, TextboxStyle};

#[derive(Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash, Debug, Default)]
pub struct StyleId(u32);

impl StyleId {
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    pub fn as_usize(&self) -> usize {
        self.0 as usize
    }
}

#[derive(Clone, Copy)]
pub enum WidgetStyle {
    Panel(PanelStyle),
    Text(TextStyle),
    Button(ButtonStyle),
    Textbox(TextboxStyle),
}

#[derive(Clone, Copy)]
pub struct NodeStyle {
    pub parent_style: Option<StyleId>,

    pub widget_style: WidgetStyle,

    pub position_type: Option<PositionType>,

    pub width: Option<SizeUnits>,
    pub height: Option<SizeUnits>,
    pub width_min: Option<SizeUnits>,
    pub width_max: Option<SizeUnits>,
    pub height_min: Option<SizeUnits>,
    pub height_max: Option<SizeUnits>,

    pub margin_left: Option<MarginUnits>,
    pub margin_right: Option<MarginUnits>,
    pub margin_top: Option<MarginUnits>,
    pub margin_bottom: Option<MarginUnits>,

    pub solid_override: Option<Solid>,
    aspect_ratio: Option<(f32, f32)>,

    pub self_halign: Option<Alignment>,
    pub self_valign: Option<Alignment>,
}

impl NodeStyle {
    pub fn empty(widget_style: WidgetStyle) -> Self {
        Self {
            parent_style: None,
            widget_style,
            position_type: None,

            width: None,
            height: None,
            width_min: None,
            width_max: None,
            height_min: None,
            height_max: None,

            margin_left: None,
            margin_right: None,
            margin_top: None,
            margin_bottom: None,

            solid_override: None,
            aspect_ratio: None,

            self_halign: None,
            self_valign: None,
        }
    }

    pub fn aspect_ratio(&self) -> Option<(f32, f32)> {
        self.aspect_ratio
    }

    pub fn set_aspect_ratio(&mut self, width: f32, height: f32) {
        // validate
        if width.fract() != 0.0 || height.fract() != 0.0 {
            panic!(
                "Aspect ratio must be a whole number, got: {} / {}",
                width, height
            );
        }
        if width < 0.0 || height < 0.0 {
            panic!(
                "Aspect ratio must be a positive number, got: {} / {}",
                width, height
            );
        }
        if width >= 256.0 || height >= 256.0 {
            panic!("Aspect ratio must be <= 256, got: {} / {}", width, height);
        }

        self.aspect_ratio = Some((width, height));
    }
}
