use crate::components::VertexTypeValue;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum FileExtension {
    Unknown,
    Skel,
    Mesh,
    Mask,
}

impl FileExtension {
    pub fn from_file_name(file_name: &str) -> Self {
        // split file name by '.'
        let split: Vec<_> = file_name.split('.').collect();
        let ext: &str = split.last().unwrap();

        // match file extension to enum
        match ext {
            "skel" => FileExtension::Skel,
            "mesh" => FileExtension::Mesh,
            "mask" => FileExtension::Mask,
            _ => FileExtension::Unknown,
        }
    }

    pub fn can_io(&self) -> bool {
        match self {
            FileExtension::Skel | FileExtension::Mesh => true,
            _ => false,
        }
    }

    pub fn vertex_type(&self) -> Option<VertexTypeValue> {
        match self {
            FileExtension::Skel => Some(VertexTypeValue::Skel),
            FileExtension::Mesh => Some(VertexTypeValue::Mesh),
            _ => None,
        }
    }
}
