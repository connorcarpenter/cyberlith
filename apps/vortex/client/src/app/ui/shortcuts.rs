use bevy_ecs::prelude::World;

use render_egui::{
    egui,
    egui::{KeyboardShortcut, Modifiers},
};

use crate::app::resources::action::{action_stack_redo, action_stack_undo};

pub const SHORTCUT_UNDO: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, egui::Key::Z);
pub const SHORTCUT_REDO: KeyboardShortcut = KeyboardShortcut::new(
    Modifiers {
        ctrl: true,
        shift: true,
        ..Modifiers::NONE
    },
    egui::Key::Z,
);

pub fn consume_shortcuts(context: &egui::Context, world: &mut World) {

    if context.input_mut(|i| i.consume_shortcut(&SHORTCUT_UNDO)) {
        action_stack_undo(world);
    }

    if context.input_mut(|i| i.consume_shortcut(&SHORTCUT_REDO)) {
        action_stack_redo(world);
    }
}
