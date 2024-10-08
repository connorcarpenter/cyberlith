use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "read_json")] {
        mod read;
    } else {}
}

cfg_if! {
    if #[cfg(feature = "write_json")] {
        mod write;
    } else {}
}

use asset_id::AssetId;
///
use serde::{Deserialize, Serialize};

use ui_builder_config::WidgetKind;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UiConfigJson {
    first_input: Option<usize>,
    styles: Vec<UiStyleJson>,
    nodes: Vec<UiNodeJson>,
}

impl UiConfigJson {
    pub const CURRENT_SCHEMA_VERSION: u32 = 0;

    pub fn dependencies(&self) -> Vec<AssetId> {
        Vec::new()
    }
}

// Style

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct UiStyleJson {
    parent_style: Option<usize>,
    id_str: Option<String>,

    widget_style: WidgetStyleJson,

    position_type: Option<PositionTypeJson>,

    width: Option<SizeUnitsJson>,
    height: Option<SizeUnitsJson>,
    width_min: Option<SizeUnitsJson>,
    width_max: Option<SizeUnitsJson>,
    height_min: Option<SizeUnitsJson>,
    height_max: Option<SizeUnitsJson>,

    margin_left: Option<MarginUnitsJson>,
    margin_right: Option<MarginUnitsJson>,
    margin_top: Option<MarginUnitsJson>,
    margin_bottom: Option<MarginUnitsJson>,

    solid_override: Option<SolidJson>,
    aspect_ratio: Option<(f32, f32)>,

    self_halign: Option<AlignmentJson>,
    self_valign: Option<AlignmentJson>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct PanelStyleJson {
    is_viewport: bool,

    background_color: Option<ColorJson>,
    background_alpha: Option<f32>,

    layout_type: Option<LayoutTypeJson>,

    padding_left: Option<MarginUnitsJson>,
    padding_right: Option<MarginUnitsJson>,
    padding_top: Option<MarginUnitsJson>,
    padding_bottom: Option<MarginUnitsJson>,

    row_between: Option<MarginUnitsJson>,
    col_between: Option<MarginUnitsJson>,
    children_halign: Option<AlignmentJson>,
    children_valign: Option<AlignmentJson>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct TextStyleJson {
    background_color: Option<ColorJson>,
    background_alpha: Option<f32>,
    text_color: Option<ColorJson>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct ButtonStyleJson {
    panel: PanelStyleJson,
    hover_color: Option<ColorJson>,
    down_color: Option<ColorJson>,
    disabled_color: Option<ColorJson>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct TextboxStyleJson {
    background_color: Option<ColorJson>,
    background_alpha: Option<f32>,
    text_color: Option<ColorJson>,

    hover_color: Option<ColorJson>,
    active_color: Option<ColorJson>,
    select_color: Option<ColorJson>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct SpinnerStyleJson {
    background_color: Option<ColorJson>,
    background_alpha: Option<f32>,
    spinner_color: Option<ColorJson>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub(crate) enum WidgetStyleJson {
    Panel(PanelStyleJson),
    Text(TextStyleJson),
    Button(ButtonStyleJson),
    Textbox(TextboxStyleJson),
    Spinner(SpinnerStyleJson),
    UiContainer,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub(crate) enum PositionTypeJson {
    Absolute,
    Relative,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub(crate) enum SizeUnitsJson {
    Pixels(f32),
    Percentage(f32),
    Viewport(f32),
    Auto,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub(crate) enum MarginUnitsJson {
    Percentage(f32),
    Viewport(f32),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub(crate) enum SolidJson {
    Fit,
    Fill,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub(crate) enum AlignmentJson {
    Start,
    Center,
    End,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub(crate) enum LayoutTypeJson {
    Row,
    Column,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct ColorJson {
    r: u8,
    g: u8,
    b: u8,
}

// Node

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct UiNodeJson {
    style_id: Option<usize>,
    widget: WidgetJson,
    init_visible: bool,
    id_str: Option<String>,
}

impl UiNodeJson {
    fn widget_kind(&self) -> WidgetKind {
        match &self.widget {
            WidgetJson::Panel(_) => WidgetKind::Panel,
            WidgetJson::Text(_) => WidgetKind::Text,
            WidgetJson::Button(_) => WidgetKind::Button,
            WidgetJson::Textbox(_) => WidgetKind::Textbox,
            WidgetJson::Spinner(_) => WidgetKind::Spinner,
            WidgetJson::UiContainer(_) => WidgetKind::UiContainer,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub(crate) enum WidgetJson {
    Panel(PanelJson),
    Text(TextJson),
    Button(ButtonJson),
    Textbox(TextboxJson),
    Spinner(SpinnerJson),
    UiContainer(UiContainerJson),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct PanelJson {
    children: Vec<usize>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct TextJson {
    init_text: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct ButtonJson {
    panel: PanelJson,
    navigation: NavigationJson,
    enabled: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct TextboxJson {
    navigation: NavigationJson,
    is_password: bool,
    validation: Option<ValidationJson>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct SpinnerJson {}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct UiContainerJson {}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct NavigationJson {
    up: Option<String>,
    down: Option<String>,
    left: Option<String>,
    right: Option<String>,
    tab: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub(crate) enum ValidationJson {
    Alphanumeric,
    Password,
    Email,
}
