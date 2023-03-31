use std::default::Default;

use bevy_app::{App, Plugin};

use crate::{EguiContext, EguiUserTextures};

// Plugin
pub struct RenderEguiPlugin;

impl Plugin for RenderEguiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(EguiContext::default())
            .insert_resource(EguiUserTextures::default());
    }
}
