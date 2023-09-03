use bevy_ecs::world::World;

use render_egui::egui::Ui;

use crate::app::resources::toolbar::Toolbar;

pub struct AnimationToolbar;

impl Default for AnimationToolbar {
    fn default() -> Self {
        Self
    }
}

impl AnimationToolbar {
    pub(crate) fn render(&mut self, ui: &mut Ui, _world: &mut World) {
        // skeleton file name visibility toggle
        let _response = Toolbar::button(ui, "🔍", "Show skeleton file name");

        // new frame
        let _response = Toolbar::button(ui, "➕", "New frame");

        // delete frame
        let _response = Toolbar::button(ui, "🗑", "Delete frame");
    }
}