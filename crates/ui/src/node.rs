
use crate::{text::TextStyleRef, style::{WidgetStyle, NodeStyle, StyleId}, widget::Widget, NodeId, panel::{Panel, PanelStyle, PanelStyleRef}};

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
        let index = self.nodes.len();
        self.nodes.push(node);
        NodeId::new(index as u32)
    }

    pub(crate) fn get_node(&self, node_id: &NodeId) -> Option<&UiNode> {
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

    pub(crate) fn panel_ref(&self, node_id: &NodeId) -> Option<&Panel> {
        let node = self.get_node(node_id)?;
        if node.kind == WidgetKind::Panel {
            return UiNode::downcast_ref::<Panel>(node.widget.as_ref());
        }
        None
    }

    fn node_style_ids(&self, node_id: &NodeId) -> &Vec<StyleId> {
        let node = self.get_node(node_id).unwrap();
        &node.style_ids
    }

    pub(crate) fn node_kind(&self, node_id: &NodeId) -> WidgetKind {
        self.get_node(node_id).unwrap().kind
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

    pub(crate) fn panel_style_ref(&self, node_id: &NodeId) -> PanelStyleRef {
        PanelStyleRef::new(self, *node_id)
    }

    pub fn text_style_ref(&self, node_id: &NodeId) -> TextStyleRef {
        TextStyleRef::new(self, *node_id)
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum WidgetKind {
    Panel,
    Text,
}

#[derive(Clone)]
pub struct UiNode {
    pub visible: bool,
    pub style_ids: Vec<StyleId>,
    pub kind: WidgetKind,
    pub widget: Box<dyn Widget>,
}

impl UiNode {
    pub(crate) fn new<W: Widget>(kind: &WidgetKind, widget: W) -> Self {
        Self {
            visible: true,
            style_ids: Vec::new(),
            widget: Box::new(widget),
            kind: *kind,
        }
    }

    pub fn downcast_ref<T: Widget>(widget: &dyn Widget) -> Option<&T> {
        widget.as_any().downcast_ref()
    }

    // pub(crate) fn downcast_mut<T: Widget>(widget: &mut dyn Widget) -> Option<&mut T> {
    //     let widget_any: &mut dyn Any = widget.as_any_mut();
    //     widget_any.downcast_mut()
    // }
}
