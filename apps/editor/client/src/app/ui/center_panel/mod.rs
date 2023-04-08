
mod tab_bar;
use tab_bar::tab_bar;

mod workspace;
use workspace::workspace;

use render_egui::egui;

pub fn center_panel(
    context: &egui::Context,
) {
    egui::CentralPanel::default()
        .show(context, |ui| {
            tab_bar(ui);
            workspace(ui);
        });
}