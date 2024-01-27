use render_egui::egui::Color32;

// File Row Colors
pub struct FileRowColors {
    pub available: Option<Color32>,
    pub requested: Color32,
    pub granted: Color32,
    pub denied: Color32,
}

pub const FILE_ROW_COLORS_UNSELECTED: FileRowColors = FileRowColors {
    available: None,
    requested: Color32::from_rgb(16, 40, 48),
    granted: Color32::from_rgb(0, 48, 64),
    denied: Color32::from_rgb(64, 0, 0),
};

pub const FILE_ROW_COLORS_HOVER: FileRowColors = FileRowColors {
    available: Some(Color32::from_gray(12)),
    requested: Color32::from_rgb(16, 52, 64),
    granted: Color32::from_rgb(0, 72, 96),
    denied: Color32::from_rgb(96, 0, 0),
};

pub const FILE_ROW_COLORS_SELECTED: FileRowColors = FileRowColors {
    available: Some(Color32::from_gray(72)),
    requested: Color32::from_rgb(16, 64, 80),
    granted: Color32::from_rgb(0, 96, 128),
    denied: Color32::from_rgb(128, 0, 0),
};

// Text Colors
pub struct TextColors {
    pub default: Color32,
    pub disabled: Color32,
    pub modified: Color32,
    pub created: Color32,
    pub deleted: Color32,
}

pub const TEXT_COLORS_UNSELECTED: TextColors = TextColors {
    default: Color32::from_gray(140),
    disabled: Color32::from_gray(100),
    modified: Color32::from_rgb(80, 120, 160),
    created: Color32::from_rgb(80, 160, 80),
    deleted: Color32::from_rgb(160, 80, 80),
};

pub const TEXT_COLORS_HOVER: TextColors = TextColors {
    default: Color32::from_gray(140),
    disabled: Color32::from_gray(120),
    modified: Color32::from_rgb(110, 130, 150),
    created: Color32::from_rgb(110, 150, 110),
    deleted: Color32::from_rgb(150, 110, 110),
};

pub const TEXT_COLORS_SELECTED: TextColors = TextColors {
    default: Color32::from_gray(140),
    disabled: Color32::from_gray(140),
    modified: Color32::from_gray(140),
    created: Color32::from_gray(140),
    deleted: Color32::from_gray(140),
};
