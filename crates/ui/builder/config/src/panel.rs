use render_api::base::Color;
use ui_layout::{Alignment, LayoutType, MarginUnits, NodeId};

#[derive(Clone)]
pub struct Panel {
    pub children: Vec<NodeId>,
}

impl Panel {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }

    pub fn add_child(&mut self, child_id: NodeId) {
        self.children.push(child_id);
    }
}

#[derive(Clone, Copy)]
pub struct PanelStyle {
    pub is_viewport: bool,

    pub background_color: Option<Color>,
    pub background_alpha: Option<f32>, // keep it private, need to validate

    pub layout_type: Option<LayoutType>,

    pub padding_left: Option<MarginUnits>,
    pub padding_right: Option<MarginUnits>,
    pub padding_top: Option<MarginUnits>,
    pub padding_bottom: Option<MarginUnits>,

    pub row_between: Option<MarginUnits>,
    pub col_between: Option<MarginUnits>,
    pub children_halign: Option<Alignment>,
    pub children_valign: Option<Alignment>,
}

impl PanelStyle {
    pub fn merge(&mut self, other: &Self, inheriting: bool) {

        // is_viewport does not inherit, but it still needs to merge when serializing/deserializing
        if !inheriting {
            self.is_viewport = other.is_viewport;
        }

        self.background_color = other.background_color.or(self.background_color);
        self.background_alpha = other.background_alpha.or(self.background_alpha);
        self.layout_type = other.layout_type.or(self.layout_type);
        self.padding_left = other.padding_left.or(self.padding_left);
        self.padding_right = other.padding_right.or(self.padding_right);
        self.padding_top = other.padding_top.or(self.padding_top);
        self.padding_bottom = other.padding_bottom.or(self.padding_bottom);
        self.row_between = other.row_between.or(self.row_between);
        self.col_between = other.col_between.or(self.col_between);
        self.children_halign = other.children_halign.or(self.children_halign);
        self.children_valign = other.children_valign.or(self.children_valign);
    }
}

impl PanelStyle {
    pub fn empty() -> Self {
        Self {
            is_viewport: false,

            background_color: None,
            background_alpha: None,

            layout_type: None,

            padding_left: None,
            padding_right: None,
            padding_top: None,
            padding_bottom: None,

            row_between: None,
            col_between: None,
            children_halign: None,
            children_valign: None,
        }
    }

    pub fn background_alpha(&self) -> Option<f32> {
        self.background_alpha
    }

    pub fn set_background_alpha(&mut self, val: f32) {
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
