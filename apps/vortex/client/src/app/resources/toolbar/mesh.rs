use bevy_ecs::world::World;

use render_egui::egui::Ui;

use crate::app::resources::toolbar::Toolbar;

pub struct MeshToolbar;

impl Default for MeshToolbar {
    fn default() -> Self {
        Self
    }
}

impl MeshToolbar {
    pub(crate) fn render(&mut self, ui: &mut Ui, _world: &mut World) {
        // insert vertex
        let _response = Toolbar::button(ui, "🔼", "Insert vertex");

        // delete selected
        let _response = Toolbar::button(ui, "🗑", "Delete selected shape");

        // toggle normals visibility
        let _response = Toolbar::button(ui, "🔍", "Show/hide normals");

        // swap normals
        let _response = Toolbar::button(ui, "🔄", "Swap face normals");
    }
}