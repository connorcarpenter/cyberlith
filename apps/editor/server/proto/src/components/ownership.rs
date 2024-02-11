use bevy_ecs::component::Component;

use naia_bevy_shared::{EntityProperty, Property, Protocol, ProtocolPlugin, Replicate, Serde};

pub struct OwnershipComponentsPlugin;

impl ProtocolPlugin for OwnershipComponentsPlugin {
    fn build(&self, protocol: &mut Protocol) {
        protocol
            .add_component::<OwnedByFile>()
            .add_component::<FileType>();
    }
}

// FileOwnership
#[derive(Component, Replicate)]
pub struct OwnedByFile {
    pub file_entity: EntityProperty,
}

impl OwnedByFile {
    pub fn new() -> Self {
        Self::new_complete()
    }
}

// FileType
#[derive(Serde, Copy, Clone, PartialEq, Debug, Hash, Eq)]
pub enum FileExtension {
    Skel,
    Mesh,
    Anim,
    Palette,
    Skin,
    Model,
    Scene,
    Icon,
    Unknown,
}

impl From<&str> for FileExtension {
    fn from(file_name: &str) -> Self {
        // split file name by '.'
        let split: Vec<_> = file_name.split('.').collect();
        let mut ext: &str = split.last().unwrap();

        if ext == "json" {
            ext = split.get(split.len() - 2).unwrap();
        }

        //info!("file_name: {}, ext: {}", file_name, ext);

        // match file extension to enum
        match ext {
            "skeleton" => FileExtension::Skel,
            "mesh" => FileExtension::Mesh,
            "animation" => FileExtension::Anim,
            "palette" => FileExtension::Palette,
            "skin" => FileExtension::Skin,
            "model" => FileExtension::Model,
            "scene" => FileExtension::Scene,
            "icon" => FileExtension::Icon,
            _ => FileExtension::Unknown,
        }
    }
}

impl FileExtension {
    pub fn can_io(&self) -> bool {
        match self {
            FileExtension::Skel
            | FileExtension::Mesh
            | FileExtension::Anim
            | FileExtension::Palette
            | FileExtension::Skin
            | FileExtension::Model
            | FileExtension::Scene
            | FileExtension::Icon => true,
            _ => false,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            FileExtension::Skel => ".skel".to_string(),
            FileExtension::Mesh => ".mesh".to_string(),
            FileExtension::Anim => ".anim".to_string(),
            FileExtension::Palette => ".palette".to_string(),
            FileExtension::Skin => ".skin".to_string(),
            FileExtension::Model => ".model".to_string(),
            FileExtension::Scene => ".scene".to_string(),
            FileExtension::Icon => ".icon".to_string(),
            FileExtension::Unknown => "<UNKNOWN>".to_string(),
        }
    }
}

#[derive(Component, Replicate)]
pub struct FileType {
    pub value: Property<FileExtension>,
}

impl FileType {
    pub fn new(value: FileExtension) -> Self {
        Self::new_complete(value)
    }
}
