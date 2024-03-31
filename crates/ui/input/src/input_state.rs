
use input::CursorIcon;
use instant::Instant;
use math::Vec2;

use ui_types::{NodeId, UiConfig};
use ui_layout::TextMeasurer;
use ui_state::{NodeActiveState, UiState};

use crate::{input::ui_receive_input, UiGlobalEvent, UiInputEvent, UiNodeEvent};

pub struct UiInputState {
    global_events: Vec<UiGlobalEvent>,
    node_events: Vec<(NodeId, UiNodeEvent)>,
    hovering_node: Option<NodeId>,
    selected_node: Option<NodeId>,
    cursor_icon: CursorIcon,
    interact_timer: Instant,

    pub carat_index: usize,
    pub select_index: Option<usize>,
}

impl UiInputState {

    pub fn from_ui_config() -> Self {
        Self {
            global_events: Vec::new(),
            node_events: Vec::new(),
            hovering_node: None,
            selected_node: None,
            cursor_icon: CursorIcon::Default,
            interact_timer: Instant::now(),

            carat_index: 0,
            select_index: None,
        }
    }

    // events
    pub fn receive_input(&mut self, ui_config: &UiConfig, ui_state: &mut UiState, text_measurer: &dyn TextMeasurer, mouse_position: Option<Vec2>, events: Vec<UiInputEvent>) {
        ui_receive_input(ui_config, ui_state, self, text_measurer, mouse_position, events);
    }

    pub fn get_cursor_icon(&self) -> CursorIcon {
        self.cursor_icon
    }

    pub fn set_cursor_icon(&mut self, cursor_icon: CursorIcon) {
        self.cursor_icon = cursor_icon;
    }

    pub fn get_hover(&self) -> Option<NodeId> {
        self.hovering_node
    }

    pub fn receive_hover(&mut self, id: &NodeId) {
        self.hovering_node = Some(*id);
    }

    pub fn get_active_state(&self, id: &NodeId) -> NodeActiveState {
        if let Some(select_id) = self.selected_node {
            if select_id == *id {
                return NodeActiveState::Active;
            }
        }

        if let Some(hover_id) = self.hovering_node {
            if hover_id == *id {
                return NodeActiveState::Hover;
            }
        };

        return NodeActiveState::Normal;
    }

    pub fn clear_hover(&mut self) {
        self.hovering_node = None;
    }

    pub fn get_active_node(&self) -> Option<NodeId> {
        self.selected_node
    }

    pub fn set_active_node(&mut self, id_opt: Option<NodeId>) {
        self.selected_node = id_opt;
    }

    pub fn emit_global_event(&mut self, event: UiGlobalEvent) {
        self.global_events.push(event);
    }

    pub fn take_global_events(&mut self) -> Vec<UiGlobalEvent> {
        std::mem::take(&mut self.global_events)
    }

    pub fn emit_node_event(&mut self, node_id: &NodeId, event: UiNodeEvent) {
        self.node_events.push((*node_id, event));
    }

    pub fn take_node_events(&mut self) -> Vec<(NodeId, UiNodeEvent)> {
        std::mem::take(&mut self.node_events)
    }

    pub fn reset_interact_timer(&mut self) {
        self.interact_timer = Instant::now();
    }

    pub fn interact_timer_was_recent(&self) -> bool {
        self.interact_timer.elapsed().as_secs_f32() < 1.0
    }
}