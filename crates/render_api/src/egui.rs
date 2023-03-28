use std::{collections::HashMap, default::Default};

use bevy_app::{App, Plugin};
use bevy_ecs::{
    prelude::Resource,
    system::{ResMut, SystemParam},
};
use three_d::egui;

use crate::assets::{Handle, Image};

// Plugin
pub struct EguiPlugin;

impl Plugin for EguiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(EguiContext::default())
            .insert_resource(EguiUserTextures::default());
    }
}

// Contexts
#[derive(SystemParam)]
pub struct EguiContexts<'w> {
    context: ResMut<'w, EguiContext>,
    user_textures: ResMut<'w, EguiUserTextures>,
}

impl<'w> EguiContexts<'w> {
    pub fn image_id(&self, image: &Handle<Image>) -> Option<egui::TextureId> {
        self.user_textures.image_id(image)
    }

    pub fn ctx_mut(&mut self) -> &mut egui::Context {
        &mut self.context.0
    }
}

// Context
#[derive(Default, Resource)]
pub struct EguiContext(egui::Context);

// User Textures
#[derive(Default, Resource)]
pub struct EguiUserTextures {
    textures: HashMap<Handle<Image>, u64>,
    last_texture_id: u64,
}

impl EguiUserTextures {
    pub fn add_image(&mut self, image: Handle<Image>) -> egui::TextureId {
        let id = *self.textures.entry(image.clone()).or_insert_with(|| {
            let id = self.last_texture_id;
            self.last_texture_id += 1;
            id
        });
        egui::TextureId::User(id)
    }
    pub fn image_id(&self, image: &Handle<Image>) -> Option<egui::TextureId> {
        self.textures
            .get(image)
            .map(|&id| egui::TextureId::User(id))
    }
}
