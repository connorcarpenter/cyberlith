use crate::{
    Alignment, LayoutType, MarginUnits, NodeId, PositionType, SizeUnits, Solid, TextMeasurer,
};

pub trait NodeStore {
    fn node_children(&self, id: &NodeId) -> std::slice::Iter<NodeId>;
    fn node_layout_type(&self, id: &NodeId) -> LayoutType;
    fn node_position_type(&self, id: &NodeId) -> PositionType;
    fn node_width(&self, id: &NodeId) -> SizeUnits;
    fn node_height(&self, id: &NodeId) -> SizeUnits;
    fn node_width_min(&self, id: &NodeId) -> SizeUnits;
    fn node_height_min(&self, id: &NodeId) -> SizeUnits;
    fn node_width_max(&self, id: &NodeId) -> SizeUnits;
    fn node_height_max(&self, id: &NodeId) -> SizeUnits;
    fn node_margin_left(&self, id: &NodeId) -> MarginUnits;
    fn node_margin_right(&self, id: &NodeId) -> MarginUnits;
    fn node_margin_top(&self, id: &NodeId) -> MarginUnits;
    fn node_margin_bottom(&self, id: &NodeId) -> MarginUnits;
    fn node_padding_left(&self, id: &NodeId) -> MarginUnits;
    fn node_padding_right(&self, id: &NodeId) -> MarginUnits;
    fn node_padding_top(&self, id: &NodeId) -> MarginUnits;
    fn node_padding_bottom(&self, id: &NodeId) -> MarginUnits;
    fn node_row_between(&self, id: &NodeId) -> MarginUnits;
    fn node_col_between(&self, id: &NodeId) -> MarginUnits;
    fn node_is_solid(&self, id: &NodeId) -> Option<Solid>;
    fn node_is_viewport(&self, id: &NodeId) -> bool;
    fn node_is_text(&self, id: &NodeId) -> bool;
    fn node_calculate_text_width(
        &self,
        id: &NodeId,
        text_measurer: &dyn TextMeasurer,
        height: f32,
    ) -> f32;
    fn node_aspect_ratio(&self, id: &NodeId) -> Option<f32>;
    fn node_self_halign(&self, id: &NodeId) -> Alignment;
    fn node_self_valign(&self, id: &NodeId) -> Alignment;
    fn node_children_halign(&self, id: &NodeId) -> Alignment;
    fn node_children_valign(&self, id: &NodeId) -> Alignment;
}
