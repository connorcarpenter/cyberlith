use bevy_ecs::world::World;

use render_egui::egui::Ui;

use crate::app::resources::toolbar::Toolbar;

pub struct MeshToolbar;

impl MeshToolbar {
    pub(crate) fn render(ui: &mut Ui, _world: &mut World) {
        // insert vertex
        let _response = Toolbar::button(ui, "🔼", "Insert vertex", true);

        // delete selected
        let _response = Toolbar::button(ui, "🗑", "Delete selected shape", true);

        // toggle normals visibility
        let _response = Toolbar::button(ui, "🔍", "Show/hide normals", true);

        // swap normals
        let _response = Toolbar::button(ui, "🔄", "Swap face normals", true);
    }
}
