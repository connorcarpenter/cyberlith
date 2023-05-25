use bevy_ecs::prelude::World;
use render_egui::{egui, egui::{Modifiers, KeyboardShortcut}};
use crate::app::resources::action_stack::ActionStack;

pub const SHORTCUT_UNDO: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, egui::Key::Z);
pub const SHORTCUT_REDO: KeyboardShortcut = KeyboardShortcut::new(Modifiers {
    ctrl: true,
    shift: true,
    ..Modifiers::NONE
}, egui::Key::Z);

pub fn consume_shortcuts(context: &egui::Context, world: &mut World) {
    let mut action_stack = world.get_resource_mut::<ActionStack>().unwrap();

    if action_stack.has_undo() {
        if context.input_mut(|i| i.consume_shortcut(&SHORTCUT_UNDO)) {
            action_stack.undo_action();
        }
    }
    if action_stack.has_redo() {
        if context.input_mut(|i| i.consume_shortcut(&SHORTCUT_REDO)) {
            let mut action_stack = world.get_resource_mut::<ActionStack>().unwrap();
            action_stack.redo_action();
        }
    }
}