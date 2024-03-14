use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "read_json")] {
        mod read;

        pub fn read_json(data: Vec<u8>) -> Ui {
            let ui_json: UiJson = serde_json::from_slice(data.as_slice()).unwrap();
            ui_json.to_ui()
        }
    } else {}
}

cfg_if! {
    if #[cfg(feature = "write_json")] {
        mod write;

        pub fn write_json(ui: &Ui) -> Vec<u8> {
            let ui_json = UiJson::from_ui(ui);
            serde_json::to_vec_pretty(&ui_json).unwrap()
        }
    } else {}
}

///

use serde::{Deserialize, Serialize};
use asset_id::AssetId;

use ui::{Ui, WidgetKind};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct UiJson {
    text_color: ColorJson,
    text_icon_asset_id: String,
    styles: Vec<UiStyleJson>,
    nodes: Vec<UiNodeJson>,
}

impl UiJson {
    const CURRENT_SCHEMA_VERSION: u32 = 0;

    fn dependencies(&self) -> Vec<AssetId> {
        todo!()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct UiStyleJson {

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
    aspect_ratio_w_over_h: Option<f32>,

    self_halign: Option<AlignmentJson>,
    self_valign: Option<AlignmentJson>,
}

impl UiStyleJson {
    pub(crate) fn widget_kind(&self) -> WidgetKind {
        match &self.widget_style {
            WidgetStyleJson::Panel(_) => WidgetKind::Panel,
            WidgetStyleJson::Text(_) => WidgetKind::Text,
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

}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) enum WidgetStyleJson {
    Panel(PanelStyleJson),
    Text(TextStyleJson),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) enum PositionTypeJson {
    Absolute,
    Relative,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) enum SizeUnitsJson {
    Pixels(f32),
    Percentage(f32),
    Auto,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) enum MarginUnitsJson {
    Pixels(f32),
    Percentage(f32),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) enum SolidJson {
    Fit,
    Fill,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) enum AlignmentJson {
    Start,
    Center,
    End,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct UiNodeJson {
    visible: bool,
    style_ids: Vec<usize>,
    widget: WidgetJson,
}

impl UiNodeJson {
    fn widget_kind(&self) -> WidgetKind {
        match &self.widget {
            WidgetJson::Panel(_) => WidgetKind::Panel,
            WidgetJson::Text(_) => WidgetKind::Text,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) enum WidgetJson {
    Panel(PanelJson),
    Text(TextJson),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct PanelJson {
    children: Vec<usize>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct TextJson {
    text: String,
}