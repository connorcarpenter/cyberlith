use std::{collections::HashMap, default::Default};

use bevy_app::{App, Plugin};
use bevy_ecs::{
    prelude::Resource,
    system::{ResMut, SystemParam},
};

use egui;

use render_api::{Handle, Image};

use crate::{EguiContext, EguiUserTextures};

// Plugin
pub struct RenderEguiPlugin;

impl Plugin for RenderEguiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(EguiContext::default())
            .insert_resource(EguiUserTextures::default());
    }
}
