use bevy_app::{App, CoreSet, Plugin};
use bevy_ecs::schedule::{IntoSystemConfig, IntoSystemSetConfig};

use crate::{Input, InputSet, system};

// Plugin
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .insert_resource(Input::new())
            // System Sets
            .configure_set(
                InputSet::Update
                    .after(CoreSet::Last)
                    .before(CoreSet::LastFlush)
            )
            // Systems
            .add_system(system::run.in_base_set(InputSet::Update));
    }
}
