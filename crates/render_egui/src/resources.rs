use std::{collections::HashMap, default::Default};

use bevy_ecs::{
    prelude::Resource,
    system::{ResMut, SystemParam},
};

use egui;

use render_api::{base::Texture2D, Handle};

// Contexts
#[derive(SystemParam)]
pub struct EguiContexts<'w> {
    context: ResMut<'w, EguiContext>,
    user_textures: ResMut<'w, EguiUserTextures>,
}

impl<'w> EguiContexts<'w> {
    pub fn image_id(&self, texture_handle: &Handle<Texture2D>) -> Option<egui::TextureId> {
        self.user_textures.image_id(texture_handle)
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
    textures: HashMap<Handle<Texture2D>, u64>,
    last_texture_id: u64,
}

impl EguiUserTextures {
    pub fn add_image(&mut self, image: Handle<Texture2D>) -> egui::TextureId {
        let id = *self.textures.entry(image.clone()).or_insert_with(|| {
            let id = self.last_texture_id;
            self.last_texture_id += 1;
            id
        });
        egui::TextureId::User(id)
    }
    pub fn image_id(&self, texture_handle: &Handle<Texture2D>) -> Option<egui::TextureId> {
        self.textures
            .get(texture_handle)
            .map(|&id| egui::TextureId::User(id))
    }
}
