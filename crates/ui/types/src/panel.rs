use render_api::base::Color;
use ui_layout::{Alignment, LayoutType, SizeUnits};

use crate::NodeId;

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

    // returns whether or not mouse is inside the rect
    pub fn mouse_is_inside(
        layout: (f32, f32, f32, f32),
        mouse_x: f32,
        mouse_y: f32,
    ) -> bool {
        let (width, height, posx, posy) = layout;

        mouse_x >= posx && mouse_x <= posx + width + 1.0 && mouse_y >= posy && mouse_y <= posy + height + 1.0
    }
}

#[derive(Clone, Copy)]
pub struct PanelStyle {
    pub background_color: Option<Color>,
    pub background_alpha: Option<f32>,

    pub layout_type: Option<LayoutType>,

    pub padding_left: Option<SizeUnits>,
    pub padding_right: Option<SizeUnits>,
    pub padding_top: Option<SizeUnits>,
    pub padding_bottom: Option<SizeUnits>,

    pub row_between: Option<SizeUnits>,
    pub col_between: Option<SizeUnits>,
    pub children_halign: Option<Alignment>,
    pub children_valign: Option<Alignment>,
}

impl PanelStyle {
    pub fn empty() -> Self {
        Self {
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