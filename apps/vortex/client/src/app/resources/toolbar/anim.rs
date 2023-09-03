use bevy_ecs::world::World;

use render_egui::egui::Ui;

pub struct AnimationToolbar;

impl AnimationToolbar {
    pub(crate) fn render(&mut self, ui: &mut Ui, world: &mut World) {
        // skeleton file name visibility toggle
        let _response = ui.button("🔍").on_hover_text("Show skeleton file name");

        // new frame
        let _response = ui.button("➕").on_hover_text("New frame");

        // delete frame
        let _response = ui.button("🗑").on_hover_text("Delete frame");
    }
}