use morphorm::{PositionType, Solid, Units};

#[derive(Clone, Default, Copy)]
pub(crate) struct NodeStyle {

    pub(crate) position_type: PositionType,

    pub(crate) width: Units,
    pub(crate) height: Units,
    pub(crate) width_min: Units,
    pub(crate) width_max: Units,
    pub(crate) height_min: Units,
    pub(crate) height_max: Units,

    pub(crate) margin_left: Units,
    pub(crate) margin_right: Units,
    pub(crate) margin_top: Units,
    pub(crate) margin_bottom: Units,
    pub(crate) margin_left_min: Units,
    pub(crate) margin_left_max: Units,
    pub(crate) margin_right_min: Units,
    pub(crate) margin_right_max: Units,
    pub(crate) margin_top_min: Units,
    pub(crate) margin_top_max: Units,
    pub(crate) margin_bottom_min: Units,
    pub(crate) margin_bottom_max: Units,

    pub(crate) border_left: Units,
    pub(crate) border_right: Units,
    pub(crate) border_top: Units,
    pub(crate) border_bottom: Units,

    pub(crate) solid_override: Option<Solid>,
    pub(crate) aspect_ratio_w_over_h: f32,
}