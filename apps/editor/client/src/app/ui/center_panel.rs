use render_egui::egui;

use crate::app::ui::tab_bar;

pub fn center_panel(
    context: &egui::Context,
) {
    egui::CentralPanel::default()
        .show(context, |ui| {
            tab_bar(ui);
        });
}