use bevy_app::{Plugin, App, PostUpdate};

use crate::{app_exit, AppExitAction};

pub struct KernelPlugin;

impl Plugin for KernelPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<AppExitAction>()
            .add_systems(PostUpdate, app_exit::process);
    }
}