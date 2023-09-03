use bevy_ecs::prelude::World;

use render_egui::egui::Ui;

use crate::app::resources::toolbar::Toolbar;

pub struct SkeletonToolbar;

impl Default for SkeletonToolbar {
    fn default() -> Self {
        Self
    }
}

impl SkeletonToolbar {
    pub(crate) fn render(&mut self, ui: &mut Ui, _world: &mut World) {
        // delete selected vertex
        let _response = Toolbar::button(ui, "🗑", "Delete vertex");

        // name selected shape
        let _response = Toolbar::button(ui, "🔍", "Name shape");
    }
}