use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "read_bits")] {
        mod read;

        pub fn read_ui_bits(data: &[u8]) -> Ui {
            read::read_bits(data)
        }
    } else {}
}

cfg_if! {
    if #[cfg(feature = "write_bits")] {
        mod write;

        pub fn write_ui_bits(ui: &Ui) -> Vec<u8> {
            write::write_bits(ui)
        }
    } else {}
}

use naia_serde::{SerdeInternal as Serde, UnsignedInteger, UnsignedVariableInteger};

use asset_id::AssetId;
use ui::{NodeId, Ui, WidgetKind};

// Actions
#[derive(Clone)]
pub(crate) enum UiAction {
    // r, g, b
    TextColor(u8, u8, u8),
    // assetid
    TextIconAssetId(AssetId),
    // default button
    DefaultButton(Option<NodeId>),
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

impl UiStyleBits {
    pub(crate) fn widget_kind(&self) -> WidgetKind {
        match &self.widget_style {
            WidgetStyleBits::Panel(_) => WidgetKind::Panel,
            WidgetStyleBits::Text(_) => WidgetKind::Text,
            WidgetStyleBits::Button(_) => WidgetKind::Button,
            WidgetStyleBits::Textbox(_) => WidgetKind::Textbox,
        }
    }
}

#[derive(Serde, Clone, PartialEq)]
pub(crate) struct PanelStyleBits {
    background_color: Option<(u8, u8, u8)>,
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
    background_color: Option<(u8, u8, u8)>,
    background_alpha: Option<UnsignedInteger<4>>,
}

#[derive(Serde, Clone, PartialEq)]
pub(crate) struct ButtonStyleBits {
    panel: PanelStyleBits,
    hover_color: Option<(u8, u8, u8)>,
    down_color: Option<(u8, u8, u8)>,
}

#[derive(Serde, Clone, PartialEq)]
pub(crate) struct TextboxStyleBits {
    panel: PanelStyleBits,
    hover_color: Option<(u8, u8, u8)>,
    active_color: Option<(u8, u8, u8)>,
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
    Pixels(UnsignedVariableInteger<7>),
    Percent(UnsignedInteger<7>),
    Viewport(UnsignedInteger<7>),
    Auto,
}

#[derive(Serde, Clone, PartialEq)]
pub(crate) enum MarginUnitsBits {
    Pixels(UnsignedVariableInteger<7>), // TODO: is this a good value type for this?
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

// Node

#[derive(Serde, Clone, PartialEq)]
pub(crate) struct UiNodeBits {
    visible: bool,
    style_ids: Vec<u8>, // TODO: is this a good value type for this? how many styles are we likely to have?
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
    panel: PanelBits,
    id_str: String,
    navigation: NavigationBits,
}

#[derive(Serde, Clone, PartialEq)]
pub(crate) struct NavigationBits {
    up: Option<UnsignedVariableInteger<4>>,
    down: Option<UnsignedVariableInteger<4>>,
    left: Option<UnsignedVariableInteger<4>>,
    right: Option<UnsignedVariableInteger<4>>,
}
