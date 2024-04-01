use ui_builder_config::Panel;
use ui_layout::NodeId;

#[derive(Clone)]
pub struct PanelR {
    pub children: Vec<NodeId>,
}

impl From<Panel> for PanelR {
    fn from(panel: Panel) -> Self {
        Self {
            children: panel.children.iter().map(|id| (*id).into()).collect(),
        }
    }
}

impl PanelR {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
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