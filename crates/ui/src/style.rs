use layout::{PositionType, SizeUnits, Solid, MarginUnits, Alignment};

#[derive(Clone, Default, Copy)]
pub(crate) struct NodeStyle {

    pub(crate) position_type: PositionType,

    pub(crate) width: SizeUnits,
    pub(crate) height: SizeUnits,
    pub(crate) width_min: SizeUnits,
    pub(crate) width_max: SizeUnits,
    pub(crate) height_min: SizeUnits,
    pub(crate) height_max: SizeUnits,

    pub(crate) margin_left: MarginUnits,
    pub(crate) margin_right: MarginUnits,
    pub(crate) margin_top: MarginUnits,
    pub(crate) margin_bottom: MarginUnits,

    pub(crate) solid_override: Option<Solid>,
    pub(crate) aspect_ratio_w_over_h: f32,

    pub(crate) self_halign: Alignment,
    pub(crate) self_valign: Alignment,
}