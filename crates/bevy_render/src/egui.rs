use bevy_ecs::prelude::Resource;

use crate::{Handle, Image};

#[derive(Resource)]
pub struct EguiContext(egui::Context);

impl EguiContext {

    pub fn ctx_mut(&mut self) -> &mut egui::Context {
        &mut self.0
    }

    pub fn image_id(&mut self, image: &Handle<Image>) -> Option<egui::TextureId> {
        None
    }
}

#[derive(Resource)]
pub struct EguiUserTextures {

}

impl EguiUserTextures {
    pub fn add_image(&mut self, image: Handle<Image>) {

    }
}