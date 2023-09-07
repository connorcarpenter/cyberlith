use bevy_ecs::world::World;

use render_egui::{
    egui,
    egui::{Frame, Margin, Ui},
};

use crate::app::resources::toolbar::Toolbar;

pub fn render_tool_bar(ui: &mut Ui, world: &mut World) {
    egui::SidePanel::right("right_panel")
        .frame(Frame::side_top_panel(ui.style()).inner_margin(Margin {
            left: 3.0,
            right: 1.0,
            top: 2.0,
            bottom: 2.0,
        }))
        .resizable(false)
        .default_width(26.0)
        .show_inside(ui, |ui| {
            ui.style_mut().override_text_style = Some(egui::TextStyle::Heading);
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                let toolbar = world.get_resource::<Toolbar>().unwrap();
                if let Some(toolbar_kind) = toolbar.kind() {
                    toolbar_kind.render(ui, world);
                }
                ui.allocate_space(ui.available_size());
            });
            ui.allocate_space(ui.available_size());
        });
}
