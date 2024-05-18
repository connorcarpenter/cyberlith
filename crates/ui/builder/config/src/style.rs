use ui_layout::{Alignment, MarginUnits, PositionType, SizeUnits, Solid};

use crate::{panel::PanelStyle, text::TextStyle, ButtonStyle, TextboxStyle, WidgetKind, SpinnerStyle};

#[derive(Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash, Debug, Default)]
pub struct StyleId(u32);

impl StyleId {
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    pub fn as_usize(&self) -> usize {
        self.0 as usize
    }
}

#[derive(Clone, Copy)]
pub enum WidgetStyle {
    Panel(PanelStyle),
    Text(TextStyle),
    Button(ButtonStyle),
    Textbox(TextboxStyle),
    Spinner(SpinnerStyle),
    UiContainer,
}

impl WidgetStyle {
    pub fn kind(&self) -> WidgetKind {
        match self {
            Self::Panel(_) => WidgetKind::Panel,
            Self::Text(_) => WidgetKind::Text,
            Self::Button(_) => WidgetKind::Button,
            Self::Textbox(_) => WidgetKind::Textbox,
            Self::Spinner(_) => WidgetKind::Spinner,
            Self::UiContainer => WidgetKind::UiContainer,
        }
    }

    pub fn merge(&mut self, other: &Self, inheriting: bool) {
        match (self, other) {
            (Self::Panel(style), Self::Panel(other_style)) => style.merge(other_style, inheriting),
            (Self::Text(style), Self::Text(other_style)) => style.merge(other_style),
            (Self::Button(style), Self::Button(other_style)) => style.merge(other_style),
            (Self::Textbox(style), Self::Textbox(other_style)) => style.merge(other_style),
            (Self::Spinner(style), Self::Spinner(other_style)) => style.merge(other_style),
            (Self::UiContainer, Self::UiContainer) => {}
            _ => panic!("Cannot merge different widget styles"),
        }
    }
}

#[derive(Clone, Copy)]
pub struct NodeStyle {
    pub parent_style: Option<StyleId>,
    pub base: BaseNodeStyle,
}

impl NodeStyle {
    pub fn empty(widget_style: WidgetStyle) -> Self {
        Self {
            parent_style: None,
            base: BaseNodeStyle::empty(widget_style),
        }
    }

    pub fn aspect_ratio(&self) -> Option<(f32, f32)> {
        self.base.aspect_ratio
    }

    pub fn set_aspect_ratio(&mut self, width: f32, height: f32) {
        // validate
        if width.fract() != 0.0 || height.fract() != 0.0 {
            panic!(
                "Aspect ratio must be a whole number, got: {} / {}",
                width, height
            );
        }
        if width < 0.0 || height < 0.0 {
            panic!(
                "Aspect ratio must be a positive number, got: {} / {}",
                width, height
            );
        }
        if width >= 256.0 || height >= 256.0 {
            panic!("Aspect ratio must be <= 256, got: {} / {}", width, height);
        }

        self.base.aspect_ratio = Some((width, height));
    }
}

#[derive(Clone, Copy)]
pub struct BaseNodeStyle {
    pub widget_style: WidgetStyle,

    pub position_type: Option<PositionType>,

    pub width: Option<SizeUnits>,
    pub height: Option<SizeUnits>,
    pub width_min: Option<SizeUnits>,
    pub width_max: Option<SizeUnits>,
    pub height_min: Option<SizeUnits>,
    pub height_max: Option<SizeUnits>,

    pub margin_left: Option<MarginUnits>,
    pub margin_right: Option<MarginUnits>,
    pub margin_top: Option<MarginUnits>,
    pub margin_bottom: Option<MarginUnits>,

    pub solid_override: Option<Solid>,
    pub aspect_ratio: Option<(f32, f32)>,

    pub self_halign: Option<Alignment>,
    pub self_valign: Option<Alignment>,
}

impl BaseNodeStyle {
    pub fn merge(&mut self, other: &Self, inheriting: bool) {
        self.widget_style.merge(&other.widget_style, inheriting);

        self.position_type = other.position_type.or(self.position_type);
        self.width = other.width.or(self.width);
        self.height = other.height.or(self.height);
        self.width_min = other.width_min.or(self.width_min);
        self.width_max = other.width_max.or(self.width_max);
        self.height_min = other.height_min.or(self.height_min);
        self.height_max = other.height_max.or(self.height_max);
        self.margin_left = other.margin_left.or(self.margin_left);
        self.margin_right = other.margin_right.or(self.margin_right);
        self.margin_top = other.margin_top.or(self.margin_top);
        self.margin_bottom = other.margin_bottom.or(self.margin_bottom);
        self.solid_override = other.solid_override.or(self.solid_override);
        self.aspect_ratio = other.aspect_ratio.or(self.aspect_ratio);
        self.self_halign = other.self_halign.or(self.self_halign);
        self.self_valign = other.self_valign.or(self.self_valign);
    }
}

impl BaseNodeStyle {
    pub fn empty(widget_style: WidgetStyle) -> Self {
        Self {
            widget_style,
            position_type: None,

            width: None,
            height: None,
            width_min: None,
            width_max: None,
            height_min: None,
            height_max: None,

            margin_left: None,
            margin_right: None,
            margin_top: None,
            margin_bottom: None,

            solid_override: None,
            aspect_ratio: None,

            self_halign: None,
            self_valign: None,
        }
    }

    pub fn aspect_ratio(&self) -> Option<(f32, f32)> {
        self.aspect_ratio
    }
}
