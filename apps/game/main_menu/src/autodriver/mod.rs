use bevy_app::{App, Plugin, Startup};

use game_engine::logging::info;

pub(crate) struct AutodriverPlugin;

impl Plugin for AutodriverPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup);
    }
}

fn startup() {
    info!("MainMenu::Autodriver::startup()");
}