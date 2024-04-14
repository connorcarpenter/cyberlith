use bevy_app::{Plugin, App};

use crate::AppExitAction;

pub struct KernelPlugin;

impl Plugin for KernelPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<AppExitAction>();
    }
}