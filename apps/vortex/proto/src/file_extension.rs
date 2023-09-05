use crate::components::FileTypeValue;

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

    pub fn to_file_type(&self) -> FileTypeValue {
        match self {
            FileExtension::Skel => FileTypeValue::Skel,
            FileExtension::Mesh => FileTypeValue::Mesh,
            _ => panic!(
                "FileExtension::to_file_type() called on non-io file extension!: {:?}",
                self
            ),
        }
    }
}
