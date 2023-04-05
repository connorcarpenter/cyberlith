use std::collections::HashMap;

use bevy_ecs::{entity::Entity, system::Resource};

use render_api::{base::Texture2D, Handle};

/// Contains textures allocated and painted by Egui.
#[derive(Resource, Default)]
pub struct EguiManagedTextures(pub HashMap<(Entity, u64), EguiManagedTexture>);

/// Represents a texture allocated and painted by Egui.
pub struct EguiManagedTexture {
    /// Assets store handle.
    pub handle: Handle<Texture2D>,
    /// Stored in full so we can do partial updates (which bevy doesn't support).
    pub color_image: egui::ColorImage,
}
