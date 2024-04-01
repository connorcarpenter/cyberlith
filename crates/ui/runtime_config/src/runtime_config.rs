use std::collections::HashMap;
use std::slice::Iter;

use render_api::base::Color;
use ui_types::{BaseNodeStyle, ButtonStyle, Navigation, PanelStyle, StyleId, TextboxStyle, UiConfig, WidgetKind, WidgetStyle};

use crate::{UiId, panel::PanelR, node::UiNodeR, button::ButtonR, runtime_store::UiRuntimeStore, text::TextR};

pub struct UiRuntimeConfig {

    store: UiRuntimeStore,
    text_color: Color,
    first_input: UiId,
    id_str_to_node_id_map: HashMap<String, UiId>,
}

impl UiRuntimeConfig {

    pub const ROOT_NODE_ID: UiId = UiId::new(0);

    pub fn new(ui_config: UiConfig) -> Self {

        let (store, globals, node_map) = ui_config.decompose();
        let node_map = node_map.into_iter().map(|(k, v)| (k.to_string(), v.into())).collect();

        Self {
            store: UiRuntimeStore::new(store),
            text_color: globals.get_text_color(),
            first_input: globals.get_first_input_node_id().into(),
            id_str_to_node_id_map: node_map,
        }
    }

    pub fn get_text_color(&self) -> Color {
        self.text_color
    }

    pub fn get_first_input(&self) -> UiId {
        self.first_input
    }

    // nodes

    pub fn nodes_len(&self) -> usize {
        self.store.nodes_len()
    }

    pub fn nodes_iter(&self) -> Iter<'_, UiNodeR> {
        self.store.nodes_iter()
    }

    pub fn get_node_id_by_id_str(&self, id_str: &str) -> Option<UiId> {
        self.id_str_to_node_id_map.get(id_str).cloned()
    }

    pub fn get_node(&self, id: &UiId) -> Option<&UiNodeR> {
        self.store.get_node(&id)
    }

    pub fn node_kind(&self, node_id: &UiId) -> WidgetKind {
        self.get_node(node_id).unwrap().widget_kind()
    }

    pub fn panel_ref(&self, node_id: &UiId) -> Option<&PanelR> {
        let node = self.get_node(node_id)?;
        if node.widget_kind() == WidgetKind::Panel {
            return node.widget_panel_ref();
        }
        None
    }

    pub fn text_ref(&self, node_id: &UiId) -> Option<&TextR> {
        let node = self.get_node(node_id)?;
        if node.widget_kind() == WidgetKind::Text {
            return node.widget_text_ref();
        }
        None
    }

    pub fn button_ref(&self, node_id: &UiId) -> Option<&ButtonR> {
        let node = self.get_node(node_id)?;
        if node.widget_kind() == WidgetKind::Button {
            return node.widget_button_ref();
        }
        None
    }

    // styles

    pub fn style_ref(&self, id: &StyleId) -> Option<&BaseNodeStyle> {
        self.store.get_style(&id)
    }

    pub fn node_background_color(&self, node_id: &UiId) -> Option<&Color> {
        self.store.node_background_color(node_id)
    }

    pub fn get_style(&self, style_id: &StyleId) -> Option<&BaseNodeStyle> {
        self.store.get_style(style_id)
    }

    pub fn node_style(&self, node_id: &UiId) -> Option<&BaseNodeStyle> {
        let node = self.get_node(node_id)?;
        node.style_id().map(|style_id| self.get_style(&style_id)).flatten()
    }

    pub fn panel_style(&self, node_id: &UiId) -> Option<&PanelStyle> {
        let widget_style = self.widget_style(node_id)?;
        match widget_style {
            WidgetStyle::Panel(panel_style) => Some(panel_style),
            WidgetStyle::Button(button_style) => Some(&button_style.panel),
            _ => None,
        }
    }

    fn widget_style(&self, node_id: &UiId) -> Option<&WidgetStyle> {
        let style = self.node_style(node_id)?;
        Some(&style.widget_style)
    }

    pub fn button_style(&self, id: &UiId) -> Option<&ButtonStyle> {
        self.store.button_style(id)
    }

    pub fn textbox_style(&self, id: &UiId) -> Option<&TextboxStyle> {
        self.store.textbox_style(id)
    }

    // navigation
    pub fn nav_get_up_id(&self, id: &UiId) -> Option<UiId> {
        let nav = self.get_node_nav(id)?;
        let up_str: &str = nav.up_goes_to.as_ref()?;
        self.get_node_id_by_id_str(up_str)
    }

    pub fn nav_get_down_id(&self, id: &UiId) -> Option<UiId> {
        let nav = self.get_node_nav(id)?;
        let down_str: &str = nav.down_goes_to.as_ref()?;
        self.get_node_id_by_id_str(down_str)
    }

    pub fn nav_get_left_id(&self, id: &UiId) -> Option<UiId> {
        let nav = self.get_node_nav(id)?;
        let left_str: &str = nav.left_goes_to.as_ref()?;
        self.get_node_id_by_id_str(left_str)
    }

    pub fn nav_get_right_id(&self, id: &UiId) -> Option<UiId> {
        let nav = self.get_node_nav(id)?;
        let right_str: &str = nav.right_goes_to.as_ref()?;
        self.get_node_id_by_id_str(right_str)
    }

    pub fn nav_get_tab_id(&self, id: &UiId) -> Option<UiId> {
        let nav = self.get_node_nav(id)?;
        let tab_str: &str = nav.tab_goes_to.as_ref()?;
        self.get_node_id_by_id_str(tab_str)
    }

    fn get_node_nav(&self, id: &UiId) -> Option<&Navigation> {
        let node = self.get_node(id)?;
        match node.widget_kind() {
            WidgetKind::Button => Some(&node.widget_button_ref()?.navigation),
            WidgetKind::Textbox => Some(&node.widget_textbox_ref()?.navigation),
            _ => None,
        }
    }

    pub fn get_style_background_alpha(&self, id: &UiId) -> f32 {

        match self.get_node(id).unwrap().widget_kind() {
            WidgetKind::Panel => {
                let mut output = 1.0;
                if let Some(panel_style) = self.store.panel_style(id) {
                    if let Some(alpha) = panel_style.background_alpha {
                        output = alpha;
                    }
                }
                output
            }
            WidgetKind::Text => {
                let mut output = 0.0;
                if let Some(text_style) = self.store.text_style(id) {
                    if let Some(alpha) = text_style.background_alpha {
                        output = alpha;
                    }
                }
                output
            }
            WidgetKind::Button => {
                let mut output = 1.0;
                if let Some(panel_style) = self.store.panel_style(id) {
                    if let Some(alpha) = panel_style.background_alpha {
                        output = alpha;
                    }
                }
                output
            }
            WidgetKind::Textbox => {
                let mut output = 1.0;
                if let Some(textbox_style) = self.store.textbox_style(id) {
                    if let Some(alpha) = textbox_style.background_alpha {
                        output = alpha;
                    }
                }
                output
            }
        }
    }
}