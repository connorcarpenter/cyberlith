use ui_layout::{Alignment, MarginUnits, PositionType, SizeUnits, Solid};

use crate::{text::TextStyle, panel::PanelStyle};

#[derive(Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash, Debug, Default)]
pub struct StyleId(u32);

impl StyleId {
    pub(crate) const fn new(id: u32) -> Self {
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
pub(crate) struct NodeStyle {

    pub(crate) widget_style: WidgetStyle,

    pub(crate) position_type: Option<PositionType>,

    pub(crate) width: Option<SizeUnits>,
    pub(crate) height: Option<SizeUnits>,
    pub(crate) width_min: Option<SizeUnits>,
    pub(crate) width_max: Option<SizeUnits>,
    pub(crate) height_min: Option<SizeUnits>,
    pub(crate) height_max: Option<SizeUnits>,

    pub(crate) margin_left: Option<MarginUnits>,
    pub(crate) margin_right: Option<MarginUnits>,
    pub(crate) margin_top: Option<MarginUnits>,
    pub(crate) margin_bottom: Option<MarginUnits>,

    pub(crate) solid_override: Option<Solid>,
    pub(crate) aspect_ratio_w_over_h: Option<f32>,

    pub(crate) self_halign: Option<Alignment>,
    pub(crate) self_valign: Option<Alignment>,
}

impl NodeStyle {
    pub fn empty(widget_style: WidgetStyle) -> Self {
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