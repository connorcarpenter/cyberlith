use morphorm::{PositionType, SizeUnits, Solid, SpaceUnits};

#[derive(Clone, Default, Copy)]
pub(crate) struct NodeStyle {

    pub(crate) position_type: PositionType,

    pub(crate) width: SizeUnits,
    pub(crate) height: SizeUnits,
    pub(crate) width_min: SizeUnits,
    pub(crate) width_max: SizeUnits,
    pub(crate) height_min: SizeUnits,
    pub(crate) height_max: SizeUnits,

    pub(crate) margin_left: SpaceUnits,
    pub(crate) margin_right: SpaceUnits,
    pub(crate) margin_top: SpaceUnits,
    pub(crate) margin_bottom: SpaceUnits,

    pub(crate) solid_override: Option<Solid>,
    pub(crate) aspect_ratio_w_over_h: f32,
}