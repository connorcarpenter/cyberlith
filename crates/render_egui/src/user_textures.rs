use std::collections::{HashMap, HashSet};

use bevy_ecs::system::Resource;

use render_api::{base::CpuTexture2D, Handle};

/// A resource for storing `bevy_egui` user textures.
#[derive(Clone, Resource, Default)]
pub struct EguiUserTextures {
    added_textures: HashSet<Handle<CpuTexture2D>>,
    removed_textures: HashSet<Handle<CpuTexture2D>>,
    changed_textures: HashSet<Handle<CpuTexture2D>>,
    textures: HashMap<Handle<CpuTexture2D>, u64>,
}

impl EguiUserTextures {

    pub fn must_process(&self) -> bool {
        !self.added_textures.is_empty()
            || !self.removed_textures.is_empty()
            || !self.changed_textures.is_empty()
    }

    pub fn add_texture(&mut self, texture_handle: &Handle<CpuTexture2D>) {
        self.added_textures.insert(texture_handle.clone());
    }

    pub fn remove_texture(&mut self, texture_handle: &Handle<CpuTexture2D>) {
        self.removed_textures.insert(texture_handle.clone());
    }

    pub fn mark_texture_changed(&mut self, texture_handle: &Handle<CpuTexture2D>) {
        self.changed_textures.insert(texture_handle.clone());
    }

    pub fn added_textures(&self) -> Vec<Handle<CpuTexture2D>> {
        self.added_textures.iter().copied().collect()
    }

    pub fn removed_textures(&self) -> Vec<Handle<CpuTexture2D>> {
        self.removed_textures.iter().copied().collect()
    }

    pub fn changed_textures(&self) -> Vec<Handle<CpuTexture2D>> {
        self.changed_textures.iter().copied().collect()
    }

    pub fn flush_added_texture(&mut self, handle: &Handle<CpuTexture2D>) {
        self.added_textures.remove(handle);
    }

    pub fn flush_removed_texture(&mut self, handle: &Handle<CpuTexture2D>) {
        self.removed_textures.remove(handle);
    }

    pub fn flush_changed_texture(&mut self, handle: &Handle<CpuTexture2D>) {
        self.changed_textures.remove(handle);
    }

    pub fn register_texture(&mut self, handle: Handle<CpuTexture2D>, id: egui::TextureId) {
        if self.textures.contains_key(&handle) {
            panic!("Texture {:?} is already registered", handle.id);
        }
        if let egui::TextureId::User(inner) = id {
            self.textures.insert(handle, inner);
        } else {
            panic!("Texture {:?} is not a user texture", handle.id);
        }
    }

    pub fn deregister_texture(&mut self, handle: &Handle<CpuTexture2D>) -> egui::TextureId {
        if let Some(id) = self.textures.remove(handle) {
            egui::TextureId::User(id)
        } else {
            panic!("Texture {:?} is not registered", handle.id);
        }
    }

    pub fn texture_id(&self, texture_handle: &Handle<CpuTexture2D>) -> Option<egui::TextureId> {
        self.textures
            .get(texture_handle)
            .map(|&id| egui::TextureId::User(id))
    }
}
