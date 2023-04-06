use std::collections::HashMap;

use bevy_ecs::system::Resource;

use render_api::{base::Texture2D, Handle};

/// A resource for storing `bevy_egui` user textures.
#[derive(Clone, Resource, Default)]
pub struct EguiUserTextures {
    textures: HashMap<Handle<Texture2D>, u64>,
    last_texture_id: u64,
}

impl EguiUserTextures {
    /// Can accept either a strong or a weak handle.
    ///
    /// You may want to pass a weak handle if you control removing texture assets in your
    /// application manually and you don't want to bother with cleaning up textures in Egui.
    ///
    /// You'll want to pass a strong handle if a texture is used only in Egui and there are no
    /// handle copies stored anywhere else.
    pub fn add_image(&mut self, image: Handle<Texture2D>) -> egui::TextureId {
        let id = *self.textures.entry(image.clone()).or_insert_with(|| {
            let id = self.last_texture_id;
            self.last_texture_id += 1;
            id
        });
        egui::TextureId::User(id)
    }

    /// Removes the image handle and an Egui texture id associated with it.
    pub fn remove_image(&mut self, image: &Handle<Texture2D>) -> Option<egui::TextureId> {
        let id = self.textures.remove(image);
        id.map(egui::TextureId::User)
    }

    /// Returns an associated Egui texture id.
    #[must_use]
    pub fn image_id(&self, image: &Handle<Texture2D>) -> Option<egui::TextureId> {
        self.textures
            .get(image)
            .map(|&id| egui::TextureId::User(id))
    }
}
