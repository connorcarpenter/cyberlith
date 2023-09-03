use bevy_ecs::world::World;

use render_egui::egui::Ui;

pub struct MeshToolbar;

impl MeshToolbar {
    pub(crate) fn render(&mut self, ui: &mut Ui, world: &mut World) {
        // insert vertex
        let _response = ui.button("🔼").on_hover_text("Insert vertex");

        // delete selected
        let _response = ui.button("🗑").on_hover_text("Delete selected shape");

        // toggle normals visibility
        let _response = ui.button("🔍").on_hover_text("Show/hide normals");

        // swap normals
        let _response = ui.button("🔄").on_hover_text("Swap face normals");
    }
}