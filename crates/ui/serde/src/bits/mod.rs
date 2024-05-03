use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "read_bits")] {
        mod read;

        pub fn read_ui_bits(data: &[u8]) -> UiConfig {
            read::read_bits(data)
        }
    } else {}
}

cfg_if! {
    if #[cfg(feature = "write_bits")] {
        mod write;

        pub fn write_ui_bits(ui_config: &UiConfig) -> Vec<u8> {
            write::write_bits(ui_config)
        }
    } else {}
}

use naia_serde::{SerdeInternal as Serde, UnsignedInteger, UnsignedVariableInteger};

use asset_id::AssetId;
use ui_builder_config::{NodeId, UiConfig, WidgetKind};

// Actions
#[derive(Clone)]
pub(crate) enum UiAction {
    // rgb
    TextColor(ColorBits),
    // assetid
    TextIconAssetId(AssetId),
    // default button
    FirstInput(Option<NodeId>),
    // style
    Style(UiStyleBits),
    // node
    Node(UiNodeBits),
}

#[derive(Serde, Clone, PartialEq)]
pub(crate) enum UiActionType {
    TextColor,
    TextIconAssetId,
    DefaultButton,
    Style,
    Node,

    None,
}

// Style
#[derive(Serde, Clone, PartialEq)]
pub(crate) struct UiStyleBits {
    parent_style: Option<u8>, // TODO: is this a good value type for this? how many styles are we likely to have?
    widget_style: WidgetStyleBits,

    position_type: Option<PositionTypeBits>,

    width: Option<SizeUnitsBits>,
    height: Option<SizeUnitsBits>,
    width_min: Option<SizeUnitsBits>,
    width_max: Option<SizeUnitsBits>,
    height_min: Option<SizeUnitsBits>,
    height_max: Option<SizeUnitsBits>,

    margin_left: Option<MarginUnitsBits>,
    margin_right: Option<MarginUnitsBits>,
    margin_top: Option<MarginUnitsBits>,
    margin_bottom: Option<MarginUnitsBits>,

    solid_override: Option<SolidBits>,
    aspect_ratio: Option<(u8, u8)>,

    self_halign: Option<AlignmentBits>,
    self_valign: Option<AlignmentBits>,
}

#[derive(Serde, Clone, PartialEq)]
pub(crate) struct PanelStyleBits {
    background_color: Option<ColorBits>,
    background_alpha: Option<UnsignedInteger<4>>,

    layout_type: Option<LayoutTypeBits>,

    padding_left: Option<SizeUnitsBits>,
    padding_right: Option<SizeUnitsBits>,
    padding_top: Option<SizeUnitsBits>,
    padding_bottom: Option<SizeUnitsBits>,

    row_between: Option<SizeUnitsBits>,
    col_between: Option<SizeUnitsBits>,
    children_halign: Option<AlignmentBits>,
    children_valign: Option<AlignmentBits>,
}

#[derive(Serde, Clone, PartialEq)]
pub(crate) struct TextStyleBits {
    background_color: Option<ColorBits>,
    background_alpha: Option<UnsignedInteger<4>>,
}

#[derive(Serde, Clone, PartialEq)]
pub(crate) struct ButtonStyleBits {
    panel: PanelStyleBits,
    hover_color: Option<ColorBits>,
    down_color: Option<ColorBits>,
}

#[derive(Serde, Clone, PartialEq)]
pub(crate) struct TextboxStyleBits {
    background_color: Option<ColorBits>,
    background_alpha: Option<UnsignedInteger<4>>,
    hover_color: Option<ColorBits>,
    active_color: Option<ColorBits>,
    select_color: Option<ColorBits>,
}

#[derive(Serde, Clone, PartialEq)]
pub(crate) enum WidgetStyleBits {
    Panel(PanelStyleBits),
    Text(TextStyleBits),
    Button(ButtonStyleBits),
    Textbox(TextboxStyleBits),
}

#[derive(Serde, Clone, PartialEq)]
pub(crate) enum PositionTypeBits {
    Absolute,
    Relative,
}

#[derive(Serde, Clone, PartialEq)]
pub(crate) enum SizeUnitsBits {
    Percent(UnsignedInteger<7>),
    Viewport(UnsignedInteger<10>),
    Auto,
}

#[derive(Serde, Clone, PartialEq)]
pub(crate) enum MarginUnitsBits {
    Percent(UnsignedInteger<7>),        // TODO: is this a good value type for this?
    Viewport(UnsignedInteger<7>),       // TODO: is this a good value type for this?
}

#[derive(Serde, Clone, PartialEq)]
pub(crate) enum SolidBits {
    Fit,
    Fill,
}

#[derive(Serde, Clone, PartialEq)]
pub(crate) enum AlignmentBits {
    Start,
    Center,
    End,
}

#[derive(Serde, Clone, PartialEq)]
pub(crate) enum LayoutTypeBits {
    Row,
    Column,
}

#[derive(Serde, Clone, PartialEq)]
pub(crate) struct ColorBits {
    r: u8,
    g: u8,
    b: u8,
}

// Node

#[derive(Serde, Clone, PartialEq)]
pub(crate) struct UiNodeBits {
    style_id: Option<u8>, // TODO: is this a good value type for this? how many styles are we likely to have?
    widget: WidgetBits,
}

impl UiNodeBits {
    pub(crate) fn widget_kind(&self) -> WidgetKind {
        match &self.widget {
            WidgetBits::Panel(_) => WidgetKind::Panel,
            WidgetBits::Text(_) => WidgetKind::Text,
            WidgetBits::Button(_) => WidgetKind::Button,
            WidgetBits::Textbox(_) => WidgetKind::Textbox,
        }
    }
}

#[derive(Serde, Clone, PartialEq)]
pub(crate) enum WidgetBits {
    Panel(PanelBits),
    Text(TextBits),
    Button(ButtonBits),
    Textbox(TextboxBits),
}

#[derive(Serde, Clone, PartialEq)]
pub(crate) struct PanelBits {
    children: Vec<u8>, // TODO: is this a good value type for this? how many nodes are each likely to have?
}

#[derive(Serde, Clone, PartialEq)]
pub(crate) struct TextBits {
    text: String,
}

#[derive(Serde, Clone, PartialEq)]
pub(crate) struct ButtonBits {
    panel: PanelBits,
    id_str: String,
    navigation: NavigationBits,
}

#[derive(Serde, Clone, PartialEq)]
pub(crate) struct TextboxBits {
    id_str: String,
    navigation: NavigationBits,
}

#[derive(Serde, Clone, PartialEq)]
pub(crate) struct NavigationBits {
    up: Option<UnsignedVariableInteger<4>>,
    down: Option<UnsignedVariableInteger<4>>,
    left: Option<UnsignedVariableInteger<4>>,
    right: Option<UnsignedVariableInteger<4>>,
    tab: Option<UnsignedVariableInteger<4>>,
}
