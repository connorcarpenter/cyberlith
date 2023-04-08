use render_egui::egui;

pub fn right_panel(
    context: &egui::Context,
) {
    egui::SidePanel::right("right_panel")
        .resizable(false)
        .default_width(60.0)
        .show(context, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Tools");
            });

            ui.separator();

            ui.vertical_centered(|ui| {
                ui.label("T1");
            });
        });
}