use render_egui::{egui, egui::{Ui, Modifiers}};

pub fn top_bar(
    context: &egui::Context,
) {
    egui::TopBottomPanel::top("top_bar").show(context, |ui| {
        egui::menu::bar(ui, |ui| {
            file_menu_button(ui);
            edit_menu_button(ui);
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
                egui::Button::new("Option 1")
                    .shortcut_text(ui.ctx().format_shortcut(&option_1_shortcut)),
            )
            .on_hover_text("option 1")
            .clicked()
        {
            // execute some logic 1

            ui.close_menu();
        }

        if ui
            .add(
                egui::Button::new("Option 2")
                    .shortcut_text(ui.ctx().format_shortcut(&option_2_shortcut)),
            )
            .on_hover_text("option 2")
            .clicked()
        {
            // execute some logic 2

            ui.close_menu();
        }
    });
}

fn edit_menu_button(ui: &mut Ui) {

    let option_1_shortcut =
        egui::KeyboardShortcut::new(Modifiers::CTRL | Modifiers::SHIFT, egui::Key::A);
    let option_2_shortcut =
        egui::KeyboardShortcut::new(Modifiers::CTRL | Modifiers::SHIFT, egui::Key::S);

    // NOTE: we must check the shortcuts OUTSIDE of the actual "File" menu,
    // or else they would only be checked if the "File" menu was actually open!

    ui.menu_button("Edit", |ui| {
        ui.set_min_width(220.0);
        ui.style_mut().wrap = Some(false);

        if ui
            .add(
                egui::Button::new("Option 1")
                    .shortcut_text(ui.ctx().format_shortcut(&option_1_shortcut)),
            )
            .on_hover_text("option 1")
            .clicked()
        {
            // execute some logic 1

            ui.close_menu();
        }

        if ui
            .add(
                egui::Button::new("Option 2")
                    .shortcut_text(ui.ctx().format_shortcut(&option_2_shortcut)),
            )
            .on_hover_text("option 2")
            .clicked()
        {
            // execute some logic 2

            ui.close_menu();
        }
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
                egui::Button::new("Option 1")
                    .shortcut_text(ui.ctx().format_shortcut(&option_1_shortcut)),
            )
            .on_hover_text("option 1")
            .clicked()
        {
            // execute some logic 1

            ui.close_menu();
        }

        if ui
            .add(
                egui::Button::new("Option 2")
                    .shortcut_text(ui.ctx().format_shortcut(&option_2_shortcut)),
            )
            .on_hover_text("option 2")
            .clicked()
        {
            // execute some logic 2

            ui.close_menu();
        }
    });
}