use bevy_app::{App, Plugin, Startup};

use crate::odst::startup_system::startup;

pub(crate) struct OdstPlugin;

impl Plugin for OdstPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup);
    }
}
