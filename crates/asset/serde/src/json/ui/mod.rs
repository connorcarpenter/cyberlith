use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "read_json")] {
        mod read;

        pub fn read_ui_json(data: Vec<u8>) -> UiConfig {
            let ui_json: UiConfigJson = serde_json::from_slice(data.as_slice()).unwrap();
            ui_json.into()
        }
    } else {}
}

cfg_if! {
    if #[cfg(feature = "write_json")] {
        mod write;

        pub fn write_ui_json(ui_config: UiConfig) -> UiConfigJson {
            UiConfigJson::from(&ui_config)
        }
    } else {}
}

use asset_id::AssetId;
///
use serde::{Deserialize, Serialize};

use ui_types::{UiConfig, WidgetKind};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UiConfigJson {
    text_color: ColorJson,
    text_icon_asset_id: String,
    first_input: Option<usize>,
    styles: Vec<UiStyleJson>,
    nodes: Vec<UiNodeJson>,
}

impl UiConfigJson {
    pub const CURRENT_SCHEMA_VERSION: u32 = 0;

    pub fn dependencies(&self) -> Vec<AssetId> {
        let mut output = Vec::new();

        output.push(AssetId::from_str(&self.text_icon_asset_id).unwrap());

        output
    }
}

// Style

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct UiStyleJson {
    parent_style: Option<usize>,
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

impl UiStyleJson {
    pub(crate) fn widget_kind(&self) -> WidgetKind {
        match &self.widget_style {
            WidgetStyleJson::Panel(_) => WidgetKind::Panel,
            WidgetStyleJson::Text(_) => WidgetKind::Text,
            WidgetStyleJson::Button(_) => WidgetKind::Button,
            WidgetStyleJson::Textbox(_) => WidgetKind::Textbox,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct PanelStyleJson {
    background_color: Option<ColorJson>,
    background_alpha: Option<f32>,

    layout_type: Option<LayoutTypeJson>,

    padding_left: Option<SizeUnitsJson>,
    padding_right: Option<SizeUnitsJson>,
    padding_top: Option<SizeUnitsJson>,
    padding_bottom: Option<SizeUnitsJson>,

    row_between: Option<SizeUnitsJson>,
    col_between: Option<SizeUnitsJson>,
    children_halign: Option<AlignmentJson>,
    children_valign: Option<AlignmentJson>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct TextStyleJson {
    background_color: Option<ColorJson>,
    background_alpha: Option<f32>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct ButtonStyleJson {
    panel: PanelStyleJson,
    hover_color: Option<ColorJson>,
    down_color: Option<ColorJson>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct TextboxStyleJson {
    panel: PanelStyleJson,
    hover_color: Option<ColorJson>,
    active_color: Option<ColorJson>,
    select_color: Option<ColorJson>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub(crate) enum WidgetStyleJson {
    Panel(PanelStyleJson),
    Text(TextStyleJson),
    Button(ButtonStyleJson),
    Textbox(TextboxStyleJson),
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
    Pixels(f32),
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
}

impl UiNodeJson {
    fn widget_kind(&self) -> WidgetKind {
        match &self.widget {
            WidgetJson::Panel(_) => WidgetKind::Panel,
            WidgetJson::Text(_) => WidgetKind::Text,
            WidgetJson::Button(_) => WidgetKind::Button,
            WidgetJson::Textbox(_) => WidgetKind::Textbox,
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
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct PanelJson {
    children: Vec<usize>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct TextJson {
    text: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct ButtonJson {
    panel: PanelJson,
    id_str: String,
    navigation: NavigationJson,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct TextboxJson {
    panel: PanelJson,
    id_str: String,
    navigation: NavigationJson,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct NavigationJson {
    up: Option<String>,
    down: Option<String>,
    left: Option<String>,
    right: Option<String>,
    tab: Option<String>,
}
