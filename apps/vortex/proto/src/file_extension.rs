#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum FileExtension {
    Unknown,
    Skel,
    Mesh,
    Skin,
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
            "skin" => FileExtension::Skin,
            "mask" => FileExtension::Mask,
            _ => FileExtension::Unknown,
        }
    }

    pub fn can_io(&self) -> bool {
        match self {
            FileExtension::Skel => true,
            _ => false,
        }
    }
}