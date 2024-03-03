use bevy_app::{App, Plugin, Update};

use crate::Ui;

// Plugin
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            // AssetManager
            .add_systems(Update, Ui::update)
        ;
    }
}
