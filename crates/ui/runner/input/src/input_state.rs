use asset_id::AssetId;
use input::{CursorIcon, Modifiers};
use instant::Instant;
use ui_runner_config::{NodeId, UiRuntimeConfig, WidgetKind};
use ui_state::NodeActiveState;

use crate::{UiGlobalEvent, UiInputEvent, UiNodeEvent};

pub struct UiInputState {
    global_events: Vec<UiGlobalEvent>,
    node_events: Vec<(AssetId, NodeId, UiNodeEvent)>,

    hovering_node: Option<(AssetId, NodeId)>,
    selected_node: Option<(AssetId, NodeId)>,
    cursor_icon: CursorIcon,

    // for textbox
    pub carat_index: usize,
    pub select_index: Option<usize>,
    interact_timer: Instant,
    carat_held: bool,
    left_pressed: Option<Modifiers>,
    right_pressed: Option<Modifiers>,
}

impl UiInputState {
    pub fn generate_new_inputs(
        &mut self,
        ui_config: &UiRuntimeConfig,
        node_id: &NodeId,
        next_inputs: &mut Vec<UiInputEvent>,
    ) {
        if WidgetKind::Textbox != ui_config.get_node(&node_id).unwrap().widget_kind() {
            return;
        }

        if !self.carat_held {
            if self.left_pressed.is_some() || self.right_pressed.is_some() {
                if self.interact_timer_elapsed_seconds(0.5) {
                    self.carat_held = true;
                }
            }
        } else {
            if self.left_pressed.is_none() && self.right_pressed.is_none() {
                self.carat_held = false;
            }
        }
        if self.carat_held {
            if self.interact_timer_elapsed_seconds(0.05) {
                if let Some(modifiers) = self.left_pressed {
                    next_inputs.push(UiInputEvent::LeftHeld(modifiers));
                } else if let Some(modifiers) = self.right_pressed {
                    next_inputs.push(UiInputEvent::RightHeld(modifiers));
                }
            }
        }
    }
}

impl UiInputState {
    pub(crate) fn set_left_pressed(&mut self, modifiers: Modifiers) {
        self.left_pressed = Some(modifiers);
    }

    pub(crate) fn set_left_released(&mut self) {
        self.left_pressed = None;
    }

    pub(crate) fn set_right_pressed(&mut self, modifiers: Modifiers) {
        self.right_pressed = Some(modifiers);
    }

    pub(crate) fn set_right_released(&mut self) {
        self.right_pressed = None;
    }
}

impl UiInputState {
    pub fn new() -> Self {
        Self {
            global_events: Vec::new(),
            node_events: Vec::new(),
            hovering_node: None,
            selected_node: None,
            cursor_icon: CursorIcon::Default,
            interact_timer: Instant::now(),

            carat_index: 0,
            select_index: None,
            carat_held: false,
            left_pressed: None,
            right_pressed: None,
        }
    }

    // events

    pub fn get_cursor_icon(&self) -> CursorIcon {
        self.cursor_icon
    }

    pub fn set_cursor_icon(&mut self, cursor_icon: CursorIcon) {
        self.cursor_icon = cursor_icon;
    }

    pub fn get_hover(&self) -> Option<(AssetId, NodeId)> {
        self.hovering_node
    }

    pub fn receive_hover(&mut self, asset_id: &AssetId, node_id: &NodeId) {
        self.hovering_node = Some((*asset_id, *node_id));
    }

    pub fn clear_hover(&mut self) {
        self.hovering_node = None;
    }

    pub fn get_active_state(&self, asset_id: &AssetId, node_id: &NodeId) -> NodeActiveState {
        if let Some((selected_asset_id, selected_node_id)) = self.selected_node {
            if selected_asset_id == *asset_id && selected_node_id == *node_id {
                return NodeActiveState::Active;
            }
        }

        if let Some((hover_asset_id, hover_node_id)) = self.hovering_node {
            if hover_asset_id == *asset_id && hover_node_id == *node_id {
                return NodeActiveState::Hover;
            }
        };

        return NodeActiveState::Normal;
    }

    pub fn get_active_node(&self) -> Option<(AssetId, NodeId)> {
        self.selected_node
    }

    pub fn set_active_node(&mut self, id_opt: Option<(AssetId, NodeId)>) {
        self.selected_node = id_opt;
    }

    pub fn emit_global_event(&mut self, event: UiGlobalEvent) {
        self.global_events.push(event);
    }

    pub fn take_global_events(&mut self) -> Vec<UiGlobalEvent> {
        std::mem::take(&mut self.global_events)
    }

    pub fn emit_node_event(&mut self, asset_id: &AssetId, node_id: &NodeId, event: UiNodeEvent) {
        self.node_events.push((*asset_id, *node_id, event));
    }

    pub fn take_node_events(&mut self) -> Vec<(AssetId, NodeId, UiNodeEvent)> {
        std::mem::take(&mut self.node_events)
    }

    pub fn reset_interact_timer(&mut self) {
        self.interact_timer = Instant::now();
    }

    pub fn interact_timer_within_seconds(&self, secs: f32) -> bool {
        let now = Instant::now();
        self.interact_timer.elapsed(&now).as_secs_f32() < secs
    }

    pub fn interact_timer_elapsed_seconds(&self, secs: f32) -> bool {
        !self.interact_timer_within_seconds(secs)
    }
}
