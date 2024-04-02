use input::{CursorIcon, InputEvent, Key, Modifiers};
use instant::Instant;
use math::Vec2;

use ui_runner_config::{NodeId, TextMeasurer, UiRuntimeConfig, WidgetKind};
use ui_state::{NodeActiveState, UiState};

use crate::{input::ui_receive_input, UiGlobalEvent, UiInputEvent, UiNodeEvent};

pub struct UiInputState {
    global_events: Vec<UiGlobalEvent>,
    node_events: Vec<(NodeId, UiNodeEvent)>,

    hovering_node: Option<NodeId>,
    selected_node: Option<NodeId>,
    cursor_icon: CursorIcon,

    // for textbox
    pub carat_index: usize,
    pub select_index: Option<usize>,
    interact_timer: Instant,
    left_pressed: Option<Modifiers>,
    right_pressed: Option<Modifiers>,
}

impl UiInputState {
    pub fn generate_new_inputs(&self, ui_config: &UiRuntimeConfig, next_inputs: &mut Vec<UiInputEvent>) {
        let Some(node_id) = self.get_active_node() else {
            return;
        };
        if WidgetKind::Textbox != ui_config.get_node(&node_id).unwrap().widget_kind() {
            return;
        }
        if let Some(modifiers) = self.left_pressed {
            next_inputs.push(UiInputEvent::LeftHeld(modifiers));
        } else if let Some(modifiers) = self.right_pressed {
            next_inputs.push(UiInputEvent::RightHeld(modifiers));
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
            left_pressed: None,
            right_pressed: None,
        }
    }

    // events
    pub fn receive_input(
        &mut self,
        ui_config: &UiRuntimeConfig,
        ui_state: &mut UiState,
        text_measurer: &dyn TextMeasurer,
        mouse_position: Option<Vec2>,
        events: Vec<UiInputEvent>,
    ) {
        ui_receive_input(
            ui_config,
            ui_state,
            self,
            text_measurer,
            mouse_position,
            events,
        );
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

    pub fn emit_node_event(&mut self, id: &NodeId, event: UiNodeEvent) {
        self.node_events.push((*id, event));
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
