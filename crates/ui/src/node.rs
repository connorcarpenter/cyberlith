use std::collections::HashMap;

use crate::{style::NodeStyle, NodeId, widget::Widget};

pub struct NodeStore {
    map: HashMap<NodeId, UiNode>,
}

impl NodeStore {
    pub(crate) fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub(crate) fn insert(&mut self, uiid: NodeId, panel: UiNode) {
        self.map.insert(uiid, panel);
    }

    pub(crate) fn get(&self, uiid: &NodeId) -> Option<&UiNode> {
        self.map.get(uiid)
    }

    pub(crate) fn get_mut(&mut self, uiid: &NodeId) -> Option<&mut UiNode> {
        self.map.get_mut(uiid)
    }

    pub(crate) fn keys(&self) -> impl Iterator<Item = &NodeId> {
        self.map.keys()
    }

    // pub(crate) fn iter(&self) -> impl Iterator<Item = (&NodeId, &UiNode)> {
    //     self.map.iter()
    // }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub(crate) enum NodeKind {
    Panel,
    Label,
}

#[derive(Clone)]
pub(crate) struct UiNode {
    pub(crate) kind: NodeKind,
    pub(crate) visible: bool,
    pub(crate) style: NodeStyle,
    pub(crate) widget: Box<dyn Widget>,
}

impl UiNode {
    pub(crate) fn new<W: Widget>(kind: &NodeKind, widget: W) -> Self {
        Self {
            visible: true,
            widget: Box::new(widget),
            kind: *kind,
            style: NodeStyle::default(),
        }
    }

    pub(crate) fn downcast_ref<T: Widget>(widget: &dyn Widget) -> Option<&T> {
        widget.as_any().downcast_ref()
    }

    // pub(crate) fn downcast_mut<T: Widget>(widget: &mut dyn Widget) -> Option<&mut T> {
    //     let widget_any: &mut dyn Any = widget.as_any_mut();
    //     widget_any.downcast_mut()
    // }
}