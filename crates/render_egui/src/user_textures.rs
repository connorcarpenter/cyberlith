use std::collections::{HashMap, HashSet};

use bevy_ecs::system::Resource;

use render_api::{base::Texture2D, Handle};

/// A resource for storing `bevy_egui` user textures.
#[derive(Clone, Resource, Default)]
pub struct EguiUserTextures {
    added_textures: HashSet<Handle<Texture2D>>,
    removed_textures: HashSet<Handle<Texture2D>>,
    textures: HashMap<Handle<Texture2D>, u64>,
}

impl EguiUserTextures {
    pub fn add_texture(&mut self, texture_handle: &Handle<Texture2D>) {
        self.added_textures.insert(texture_handle.clone());
    }

    pub fn remove_texture(&mut self, texture_handle: &Handle<Texture2D>) {
        self.removed_textures.insert(texture_handle.clone());
    }

    pub fn flush_added_textures(&mut self) -> Vec<Handle<Texture2D>> {
        self.added_textures.drain().collect()
    }

    pub fn flush_removed_textures(&mut self) -> Vec<Handle<Texture2D>> {
        self.removed_textures.drain().collect()
    }

    pub fn register_texture(&mut self, handle: Handle<Texture2D>, id: egui::TextureId) {
        if self.textures.contains_key(&handle) {
            panic!("Texture {:?} is already registered", handle.id);
        }
        if let egui::TextureId::User(inner) = id {
            self.textures.insert(handle, inner);
        } else {
            panic!("Texture {:?} is not a user texture", handle.id);
        }
    }

    pub fn deregister_texture(&mut self, handle: &Handle<Texture2D>) {
        if self.textures.remove(handle).is_none() {
            panic!("Texture {:?} is not registered", handle.id);
        }
    }

    pub fn texture_id(&self, texture_handle: &Handle<Texture2D>) -> Option<egui::TextureId> {
        self.textures
            .get(texture_handle)
            .map(|&id| egui::TextureId::User(id))
    }
}
