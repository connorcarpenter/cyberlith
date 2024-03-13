use ui_layout::{Alignment, MarginUnits, PositionType, SizeUnits, Solid};

use crate::{text::TextStyle, panel::PanelStyle};

#[derive(Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash, Debug, Default)]
pub struct StyleId(u32);

impl StyleId {
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    pub(crate) fn as_usize(&self) -> usize {
        self.0 as usize
    }
}

#[derive(Clone, Copy)]
pub enum WidgetStyle {
    Panel(PanelStyle),
    Text(TextStyle),
}

#[derive(Clone, Copy)]
pub struct NodeStyle {

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
    pub aspect_ratio_w_over_h: Option<f32>,

    pub self_halign: Option<Alignment>,
    pub self_valign: Option<Alignment>,
}

impl NodeStyle {
    pub(crate) fn empty(widget_style: WidgetStyle) -> Self {
        Self {
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
            aspect_ratio_w_over_h: None,

            self_halign: None,
            self_valign: None,
        }
    }
}