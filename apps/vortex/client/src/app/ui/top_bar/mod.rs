use bevy_ecs::{prelude::Mut, world::World};

use render_egui::{
    egui,
    egui::{Modifiers, Ui},
};

use crate::app::{
    resources::action_stack::ActionStack,
    ui::shortcuts::{SHORTCUT_REDO, SHORTCUT_UNDO},
};

pub fn top_bar(context: &egui::Context, world: &mut World) {
    egui::TopBottomPanel::top("top_bar").show(context, |ui| {
        egui::menu::bar(ui, |ui| {
            file_menu_button(ui);
            edit_menu_button(ui, world);
            git_menu_button(ui);
        });
    });
}

fn file_menu_button(ui: &mut Ui) {
    let option_1_shortcut =
        egui::KeyboardShortcut::new(Modifiers::CTRL | Modifiers::SHIFT, egui::Key::A);
    let option_2_shortcut =
        egui::KeyboardShortcut::new(Modifiers::CTRL | Modifiers::SHIFT, egui::Key::S);

    // NOTE: we must check the shortcuts OUTSIDE of the actual "File" menu,
    // or else they would only be checked if the "File" menu was actually open!

    if ui.input_mut(|i| i.consume_shortcut(&option_1_shortcut)) {
        // execute some logic 1
    }

    if ui.input_mut(|i| i.consume_shortcut(&option_2_shortcut)) {
        // execute some logic 2
    }

    ui.menu_button("File", |ui| {
        ui.set_min_width(220.0);
        ui.style_mut().wrap = Some(false);

        if ui
            .add(
                egui::Button::new("File Option 1")
                    .shortcut_text(ui.ctx().format_shortcut(&option_1_shortcut)),
            )
            .clicked()
        {
            // execute some logic 1

            ui.close_menu();
        }

        if ui
            .add(
                egui::Button::new("File Option 2")
                    .shortcut_text(ui.ctx().format_shortcut(&option_2_shortcut)),
            )
            .clicked()
        {
            // execute some logic 2

            ui.close_menu();
        }
    });
}

fn edit_menu_button(ui: &mut Ui, world: &mut World) {
    ui.menu_button("Edit", |ui| {
        world.resource_scope(|world, mut action_stack: Mut<ActionStack>| {
            ui.set_min_width(220.0);
            ui.style_mut().wrap = Some(false);

            if ui
                .add_enabled(
                    action_stack.has_undo(),
                    egui::Button::new("⟲ Undo")
                        .shortcut_text(ui.ctx().format_shortcut(&SHORTCUT_UNDO)),
                )
                .clicked()
            {
                action_stack.undo_action(world);
                ui.close_menu();
            }

            if ui
                .add_enabled(
                    action_stack.has_redo(),
                    egui::Button::new("⟳ Redo")
                        .shortcut_text(ui.ctx().format_shortcut(&SHORTCUT_REDO)),
                )
                .clicked()
            {
                action_stack.redo_action(world);
                ui.close_menu();
            }
        });
    });
}

fn git_menu_button(ui: &mut Ui) {
    let option_1_shortcut =
        egui::KeyboardShortcut::new(Modifiers::CTRL | Modifiers::SHIFT, egui::Key::A);
    let option_2_shortcut =
        egui::KeyboardShortcut::new(Modifiers::CTRL | Modifiers::SHIFT, egui::Key::S);

    // NOTE: we must check the shortcuts OUTSIDE of the actual "File" menu,
    // or else they would only be checked if the "File" menu was actually open!

    ui.menu_button("Git", |ui| {
        ui.set_min_width(220.0);
        ui.style_mut().wrap = Some(false);

        if ui
            .add(
                egui::Button::new("Git Option 1")
                    .shortcut_text(ui.ctx().format_shortcut(&option_1_shortcut)),
            )
            .clicked()
        {
            // execute some logic 1

            ui.close_menu();
        }

        if ui
            .add(
                egui::Button::new("Git Option 2")
                    .shortcut_text(ui.ctx().format_shortcut(&option_2_shortcut)),
            )
            .clicked()
        {
            // execute some logic 2

            ui.close_menu();
        }
    });
}
