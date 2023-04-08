use render_egui::egui;

pub fn center_panel(
    context: &egui::Context,
) {
    egui::CentralPanel::default()
        .show(context, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Tabs?");
            });

            ui.separator();

            ui.vertical_centered(|ui| {
                ui.label("content");
            });
        });
}