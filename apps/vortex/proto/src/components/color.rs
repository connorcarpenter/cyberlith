use bevy_ecs::component::Component;

use naia_bevy_shared::{
    EntityProperty, Property, Protocol, ProtocolPlugin, Replicate,
};

pub struct ColorComponentsPlugin;

impl ProtocolPlugin for ColorComponentsPlugin {
    fn build(&self, protocol: &mut Protocol) {
        protocol
            .add_component::<PaletteColor>()
            .add_component::<BackgroundSkinColor>()
            .add_component::<FaceColor>();
    }
}

// PaletteColor
#[derive(Component, Replicate)]
pub struct PaletteColor {
    pub owning_file_entity: EntityProperty,
    pub index: Property<u8>,
    pub r: Property<u8>,
    pub g: Property<u8>,
    pub b: Property<u8>,
}

impl PaletteColor {
    pub fn new(index: u8, r: u8, g: u8, b: u8) -> Self {
        Self::new_complete(index, r, g, b)
    }
}

// FaceColor
#[derive(Component, Replicate)]
pub struct FaceColor {
    pub owning_file_entity: EntityProperty,
    pub face_entity: EntityProperty,
    pub palette_color_entity: EntityProperty,
}

impl FaceColor {
    pub fn new() -> Self {
        Self::new_complete()
    }
}

// BackgroundSkinColor
#[derive(Component, Replicate)]
pub struct BackgroundSkinColor {
    pub owning_file_entity: EntityProperty,
    pub palette_color_entity: EntityProperty,
}

impl BackgroundSkinColor {
    pub fn new() -> Self {
        Self::new_complete()
    }
}