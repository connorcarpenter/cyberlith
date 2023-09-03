use bevy_ecs::prelude::World;

use render_egui::egui::Ui;

pub struct SkeletonToolbar;

impl SkeletonToolbar {
    pub(crate) fn render(&mut self, ui: &mut Ui, world: &mut World) {
        // delete selected vertex
        let _response = ui.button("🗑").on_hover_text("Delete vertex");

        // name selected shape
        let _response = ui.button("🔍").on_hover_text("Name shape");
    }
}