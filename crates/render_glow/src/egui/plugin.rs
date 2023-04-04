use bevy_app::{App, Plugin};

use crate::egui::draw::draw;

pub struct EguiPlugin;

impl Plugin for EguiPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(draw);
    }
}