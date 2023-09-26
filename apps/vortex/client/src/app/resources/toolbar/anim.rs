use bevy_ecs::world::World;

use render_egui::egui::Ui;

use crate::app::resources::toolbar::{shared_buttons::button_toggle_edge_angle_visibility, Toolbar};

pub struct AnimationToolbar;

impl AnimationToolbar {
    pub(crate) fn render(ui: &mut Ui, world: &mut World) {
        // skeleton file name visibility toggle
        let _response = Toolbar::button(ui, "🔍", "Show skeleton file name", true);

        // new frame
        let _response = Toolbar::button(ui, "➕", "New frame", true);

        // delete frame
        let _response = Toolbar::button(ui, "🗑", "Delete frame", true);

        button_toggle_edge_angle_visibility(ui, world);
    }
}
