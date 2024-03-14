use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "read_json")] {
        mod read;

        pub fn read_json(data: Vec<u8>) -> Ui {
            let ui_serde: UiSerde = serde_json::from_slice(data.as_slice()).unwrap();
            ui_serde.to_ui()
        }
    } else {}
}

cfg_if! {
    if #[cfg(feature = "write_json")] {
        mod write;

        pub fn write_json(ui: &Ui) -> Vec<u8> {
            let ui_serde = UiSerde::from_ui(ui);
            serde_json::to_vec_pretty(&ui_serde).unwrap()
        }
    } else {}
}

///

use serde::{Deserialize, Serialize};
use asset_id::AssetId;

use ui::{Ui, WidgetKind};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct UiSerde {
    text_color: ColorSerde,
    text_icon_asset_id: String,
    styles: Vec<UiStyleSerde>,
    nodes: Vec<UiNodeSerde>,
}

impl UiSerde {
    const CURRENT_SCHEMA_VERSION: u32 = 0;

    fn dependencies(&self) -> Vec<AssetId> {
        todo!()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct UiStyleSerde {

    widget_style: WidgetStyleSerde,

    position_type: Option<PositionTypeSerde>,

    width: Option<SizeUnitsSerde>,
    height: Option<SizeUnitsSerde>,
    width_min: Option<SizeUnitsSerde>,
    width_max: Option<SizeUnitsSerde>,
    height_min: Option<SizeUnitsSerde>,
    height_max: Option<SizeUnitsSerde>,

    margin_left: Option<MarginUnitsSerde>,
    margin_right: Option<MarginUnitsSerde>,
    margin_top: Option<MarginUnitsSerde>,
    margin_bottom: Option<MarginUnitsSerde>,

    solid_override: Option<SolidSerde>,
    aspect_ratio_w_over_h: Option<f32>,

    self_halign: Option<AlignmentSerde>,
    self_valign: Option<AlignmentSerde>,
}

impl UiStyleSerde {
    pub(crate) fn widget_kind(&self) -> WidgetKind {
        match &self.widget_style {
            WidgetStyleSerde::Panel(_) => WidgetKind::Panel,
            WidgetStyleSerde::Text(_) => WidgetKind::Text,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct PanelStyleSerde {
    background_color: Option<ColorSerde>,
    background_alpha: Option<f32>,

    layout_type: Option<LayoutTypeSerde>,

    padding_left: Option<SizeUnitsSerde>,
    padding_right: Option<SizeUnitsSerde>,
    padding_top: Option<SizeUnitsSerde>,
    padding_bottom: Option<SizeUnitsSerde>,

    row_between: Option<SizeUnitsSerde>,
    col_between: Option<SizeUnitsSerde>,
    children_halign: Option<AlignmentSerde>,
    children_valign: Option<AlignmentSerde>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct TextStyleSerde {

}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) enum WidgetStyleSerde {
    Panel(PanelStyleSerde),
    Text(TextStyleSerde),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) enum PositionTypeSerde {
    Absolute,
    Relative,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) enum SizeUnitsSerde {
    Pixels(f32),
    Percentage(f32),
    Auto,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) enum MarginUnitsSerde {
    Pixels(f32),
    Percentage(f32),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) enum SolidSerde {
    Fit,
    Fill,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) enum AlignmentSerde {
    Start,
    Center,
    End,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) enum LayoutTypeSerde {
    Row,
    Column,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct ColorSerde {
    r: u8,
    g: u8,
    b: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct UiNodeSerde {
    visible: bool,
    style_ids: Vec<usize>,
    widget: WidgetSerde,
}

impl UiNodeSerde {
    fn widget_kind(&self) -> WidgetKind {
        match &self.widget {
            WidgetSerde::Panel(_) => WidgetKind::Panel,
            WidgetSerde::Text(_) => WidgetKind::Text,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) enum WidgetSerde {
    Panel(PanelSerde),
    Text(TextSerde),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct PanelSerde {
    children: Vec<usize>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct TextSerde {
    text: String,
}