use crate::{panel::{Panel, PanelStyle, PanelStyleRef}, style::{NodeStyle, StyleId, WidgetStyle}, widget::Widget, NodeId, Text};
use crate::widget::WidgetKind;

pub struct UiStore {
    pub styles: Vec<NodeStyle>,
    pub nodes: Vec<UiNode>,
}

impl UiStore {
    pub(crate) fn new() -> Self {
        Self {
            styles: Vec::new(),
            nodes: Vec::new(),
        }
    }

    // nodes
    pub(crate) fn insert_node(&mut self, node: UiNode) -> NodeId {
        if self.nodes.len() >= 255 {
            panic!("1 UI can only hold up to 255 nodes, too many nodes!");
        }
        let index = self.nodes.len();
        self.nodes.push(node);
        NodeId::new(index as u32)
    }

    pub fn get_node(&self, node_id: &NodeId) -> Option<&UiNode> {
        self.nodes.get(node_id.as_usize())
    }

    pub(crate) fn get_node_mut(&mut self, node_id: &NodeId) -> Option<&mut UiNode> {
        self.nodes.get_mut(node_id.as_usize())
    }

    pub(crate) fn node_ids(&self) -> Vec<NodeId> {
        let mut output = Vec::new();

        for i in 0..self.nodes.len() {
            output.push(NodeId::new(i as u32));
        }

        output
    }

    // styles

    pub(crate) fn insert_style(&mut self, style: NodeStyle) -> StyleId {
        if self.styles.len() >= 255 {
            panic!("1 UI can only hold up to 255 styles, too many styles!");
        }
        let index = self.styles.len();
        self.styles.push(style);
        StyleId::new(index as u32)
    }

    pub(crate) fn get_style(&self, style_id: &StyleId) -> Option<&NodeStyle> {
        self.styles.get(style_id.as_usize())
    }

    pub(crate) fn get_style_mut(&mut self, style_id: &StyleId) -> Option<&mut NodeStyle> {
        self.styles.get_mut(style_id.as_usize())
    }

    // pub(crate) fn iter(&self) -> impl Iterator<Item = (&NodeId, &UiNode)> {
    //     self.map.iter()
    // }

    // refs stuff

    pub fn panel_ref(&self, node_id: &NodeId) -> Option<&Panel> {
        let node = self.get_node(node_id)?;
        if node.widget_kind() == WidgetKind::Panel {
            return node.widget_panel_ref();
        }
        None
    }

    fn node_style_ids(&self, node_id: &NodeId) -> &Vec<StyleId> {
        let node = self.get_node(node_id).unwrap();
        &node.style_ids
    }

    pub(crate) fn node_kind(&self, node_id: &NodeId) -> WidgetKind {
        self.get_node(node_id).unwrap().widget_kind()
    }

    pub(crate) fn for_each_node_style(&self, node_id: &NodeId, mut func: impl FnMut(&NodeStyle)) {
        for style_id in self.node_style_ids(node_id) {
            let Some(style) = self.get_style(style_id) else {
                panic!("StyleId does not reference a Style");
            };
            func(style);
        }
    }

    pub(crate) fn for_each_panel_style(&self, node_id: &NodeId, mut func: impl FnMut(&PanelStyle)) {
        for style_id in self.node_style_ids(node_id) {
            let Some(style) = self.get_style(style_id) else {
                panic!("StyleId does not reference a Style");
            };
            let WidgetStyle::Panel(panel_style) = style.widget_style else {
                panic!("StyleId does not reference a PanelStyle");
            };
            func(&panel_style);
        }
    }

    pub fn panel_style_ref(&self, node_id: &NodeId) -> PanelStyleRef {
        PanelStyleRef::new(self, *node_id)
    }
}

#[derive(Clone)]
pub struct UiNode {
    pub visible: bool,
    pub style_ids: Vec<StyleId>,
    pub widget: Widget,
}

impl UiNode {
    pub(crate) fn new(widget: Widget) -> Self {
        Self {
            visible: true,
            style_ids: Vec::new(),
            widget,
        }
    }

    pub fn widget_kind(&self) -> WidgetKind {
        self.widget.kind()
    }

    pub fn widget_panel_ref(&self) -> Option<&Panel> {
        match &self.widget {
            Widget::Panel(panel) => Some(panel),
            _ => None,
        }
    }

    pub fn widget_panel_mut(&mut self) -> Option<&mut Panel> {
        match &mut self.widget {
            Widget::Panel(panel) => Some(panel),
            _ => None,
        }
    }

    pub fn widget_text_ref(&self) -> Option<&Text> {
        match &self.widget {
            Widget::Text(text) => Some(text),
            _ => None,
        }
    }

    pub fn widget_text_mut(&mut self) -> Option<&mut Text> {
        match &mut self.widget {
            Widget::Text(text) => Some(text),
            _ => None,
        }
    }
}
