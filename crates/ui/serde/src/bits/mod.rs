use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "read_bits")] {
        mod read;

        use naia_serde::SerdeErr;

        pub fn read_ui_bits(data: &[u8]) -> Result<UiConfig, SerdeErr> {
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

use ui_builder_config::{NodeId, UiConfig, WidgetKind};

// Actions
#[derive(Clone)]
pub(crate) enum UiAction {
    // default button
    FirstInput(Option<NodeId>),
    // style
    Style(UiStyleBits),
    // node
    Node(UiNodeBits),
}

#[derive(Serde, Clone, PartialEq)]
pub(crate) enum UiActionType {
    DefaultButton,
    Style,
    Node,

    None,
}

// Style
#[derive(Serde, Clone, PartialEq)]
pub(crate) struct UiStyleBits {
    parent_style: Option<u8>, // TODO: is this a good value type for this? how many styles are we likely to have?
    id_str: Option<String>,

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
    is_viewport: bool,

    background_color: Option<ColorBits>,
    background_alpha: Option<UnsignedInteger<4>>,

    layout_type: Option<LayoutTypeBits>,

    padding_left: Option<MarginUnitsBits>,
    padding_right: Option<MarginUnitsBits>,
    padding_top: Option<MarginUnitsBits>,
    padding_bottom: Option<MarginUnitsBits>,

    row_between: Option<MarginUnitsBits>,
    col_between: Option<MarginUnitsBits>,
    children_halign: Option<AlignmentBits>,
    children_valign: Option<AlignmentBits>,
}

#[derive(Serde, Clone, PartialEq)]
pub(crate) struct TextStyleBits {
    background_color: Option<ColorBits>,
    background_alpha: Option<UnsignedInteger<4>>,
    text_color: Option<ColorBits>,
}

#[derive(Serde, Clone, PartialEq)]
pub(crate) struct ButtonStyleBits {
    panel: PanelStyleBits,
    hover_color: Option<ColorBits>,
    down_color: Option<ColorBits>,
    disabled_color: Option<ColorBits>,
}

#[derive(Serde, Clone, PartialEq)]
pub(crate) struct TextboxStyleBits {
    background_color: Option<ColorBits>,
    background_alpha: Option<UnsignedInteger<4>>,
    text_color: Option<ColorBits>,
    hover_color: Option<ColorBits>,
    active_color: Option<ColorBits>,
    select_color: Option<ColorBits>,
}

#[derive(Serde, Clone, PartialEq)]
pub(crate) struct SpinnerStyleBits {
    background_color: Option<ColorBits>,
    background_alpha: Option<UnsignedInteger<4>>,
    spinner_color: Option<ColorBits>,
}

#[derive(Serde, Clone, PartialEq)]
pub(crate) enum WidgetStyleBits {
    Panel(PanelStyleBits),
    Text(TextStyleBits),
    Button(ButtonStyleBits),
    Textbox(TextboxStyleBits),
    Spinner(SpinnerStyleBits),
    UiContainer,
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
    Viewport(UnsignedInteger<10>),
    Auto,
}

#[derive(Serde, Clone, PartialEq)]
pub(crate) enum MarginUnitsBits {
    Percent(UnsignedInteger<7>), // TODO: is this a good value type for this?
    Viewport(UnsignedInteger<10>), // TODO: is this a good value type for this?
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

#[derive(Serde, Clone, PartialEq, Debug)]
pub(crate) struct UiNodeBits {
    style_id: Option<u8>, // TODO: is this a good value type for this? how many styles are we likely to have?
    widget: WidgetBits,
    init_visible: bool,
    id_str: Option<String>,
}

impl UiNodeBits {
    pub(crate) fn widget_kind(&self) -> WidgetKind {
        match &self.widget {
            WidgetBits::Panel(_) => WidgetKind::Panel,
            WidgetBits::Text(_) => WidgetKind::Text,
            WidgetBits::Button(_) => WidgetKind::Button,
            WidgetBits::Textbox(_) => WidgetKind::Textbox,
            WidgetBits::Spinner(_) => WidgetKind::Spinner,
            WidgetBits::UiContainer(_) => WidgetKind::UiContainer,
        }
    }
}

#[derive(Serde, Clone, PartialEq, Debug)]
pub(crate) enum WidgetBits {
    Panel(PanelBits),
    Text(TextBits),
    Button(ButtonBits),
    Textbox(TextboxBits),
    Spinner(SpinnerBits),
    UiContainer(UiContainerBits),
}

#[derive(Serde, Clone, PartialEq, Debug)]
pub(crate) struct PanelBits {
    children: Vec<u8>, // TODO: is this a good value type for this? how many nodes are each likely to have?
}

#[derive(Serde, Clone, PartialEq, Debug)]
pub(crate) struct TextBits {
    init_text: String,
}

#[derive(Serde, Clone, PartialEq, Debug)]
pub(crate) struct ButtonBits {
    panel: PanelBits,
    navigation: NavigationBits,
    enabled: bool,
}

#[derive(Serde, Clone, PartialEq, Debug)]
pub(crate) struct TextboxBits {
    navigation: NavigationBits,
    is_password: bool,
    validation: Option<ValidationBits>,
}

#[derive(Serde, Clone, PartialEq, Debug)]
pub(crate) struct NavigationBits {
    up: Option<UnsignedVariableInteger<4>>,
    down: Option<UnsignedVariableInteger<4>>,
    left: Option<UnsignedVariableInteger<4>>,
    right: Option<UnsignedVariableInteger<4>>,
    tab: Option<UnsignedVariableInteger<4>>,
}

#[derive(Serde, Clone, PartialEq, Copy, Debug)]
pub(crate) enum ValidationBits {
    Alphanumeric,
    Password,
    Email,
}

#[derive(Serde, Clone, PartialEq, Debug)]
pub(crate) struct SpinnerBits {}

#[derive(Serde, Clone, PartialEq, Debug)]
pub(crate) struct UiContainerBits {}
