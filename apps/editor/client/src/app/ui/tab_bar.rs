use render_egui::{egui, egui::{Ui, Modifiers}};

pub fn tab_bar(
    ui: &mut egui::Ui,
) {
    egui::TopBottomPanel::top("tab_bar").show_inside(ui, |ui| {
        egui::menu::bar(ui, |ui| {
            ui.add(egui::Button::new("Tab 1"));
            ui.add(egui::Button::new("Tab 2"));
            ui.add(egui::Button::new("Tab 3"));
        });
    });
}