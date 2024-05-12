use std::collections::HashMap;

use naia_serde::{FileBitWriter, SerdeInternal as Serde, UnsignedInteger, UnsignedVariableInteger};

use render_api::base::Color;
use ui_builder_config::{
    Button, ButtonStyle, Navigation, NodeStyle, Panel, PanelStyle, StyleId, Text, TextStyle,
    Textbox, TextboxStyle, UiConfig, UiNode, Widget, WidgetStyle,
};
use ui_layout::{Alignment, LayoutType, MarginUnits, PositionType, SizeUnits, Solid};

use crate::bits::{
    AlignmentBits, ButtonBits, ButtonStyleBits, ColorBits, LayoutTypeBits, MarginUnitsBits,
    NavigationBits, PanelBits, PanelStyleBits, PositionTypeBits, SizeUnitsBits, SolidBits,
    TextBits, TextStyleBits, TextboxBits, TextboxStyleBits, UiAction, UiActionType, UiNodeBits,
    UiStyleBits, WidgetBits, WidgetStyleBits,
};

pub fn write_bits(ui_config: &UiConfig) -> Vec<u8> {
    let actions = convert_ui_to_actions(ui_config);
    actions_to_bytes(actions)
}

fn convert_ui_to_actions(ui_config: &UiConfig) -> Vec<UiAction> {
    let mut output = Vec::new();

    // write text color
    let text_color = ui_config.get_text_color();
    output.push(UiAction::TextColor(ColorBits::from(text_color)));

    // write text icon AssetId
    let text_icon_asset_id = ui_config.get_text_icon_asset_id();
    output.push(UiAction::TextIconAssetId(text_icon_asset_id));

    // write eye icon AssetId
    let eye_icon_asset_id = ui_config.get_eye_icon_asset_id();
    output.push(UiAction::EyeIconAssetId(eye_icon_asset_id));

    // write first input
    let first_input_id = ui_config.get_first_input();
    output.push(UiAction::FirstInput(first_input_id));

    // write styles

    let mut style_id_to_index = HashMap::new();
    let mut style_count = 0;

    for (style_id, style) in ui_config.styles_iter().enumerate() {
        let style_id = StyleId::new(style_id as u32);
        let next_index = style_count;
        if style_count == u8::MAX {
            panic!("Too many styles, max is {}", u8::MAX);
        }
        style_count += 1;
        style_id_to_index.insert(style_id, next_index);

        output.push(UiAction::Style(UiStyleBits::from(style)));
    }

    // write nodes
    for node in ui_config.nodes_iter() {
        output.push(UiAction::Node(UiNodeBits::from_node(
            ui_config,
            &style_id_to_index,
            node,
        )));
    }

    output
}

fn actions_to_bytes(actions: Vec<UiAction>) -> Vec<u8> {
    let mut bit_writer = FileBitWriter::new();

    for action in actions {
        match action {
            UiAction::TextColor(text_color) => {
                UiActionType::TextColor.ser(&mut bit_writer);
                text_color.ser(&mut bit_writer);
            }
            UiAction::TextIconAssetId(asset_id) => {
                UiActionType::TextIconAssetId.ser(&mut bit_writer);
                asset_id.as_u32().ser(&mut bit_writer);
            }
            UiAction::EyeIconAssetId(asset_id) => {
                UiActionType::EyeIconAssetId.ser(&mut bit_writer);
                asset_id.as_u32().ser(&mut bit_writer);
            }
            UiAction::FirstInput(button_id_opt) => {
                UiActionType::DefaultButton.ser(&mut bit_writer);
                let val_opt = button_id_opt.map(|id| {
                    let val = id.as_usize();
                    let val = UnsignedVariableInteger::<7>::new(val as i128);
                    val
                });
                val_opt.ser(&mut bit_writer);
            }
            UiAction::Style(style) => {
                UiActionType::Style.ser(&mut bit_writer);
                style.ser(&mut bit_writer);
            }
            UiAction::Node(node) => {
                UiActionType::Node.ser(&mut bit_writer);
                node.ser(&mut bit_writer);
            }
        }
    }

    // continue bit
    UiActionType::None.ser(&mut bit_writer);

    bit_writer.to_bytes().to_vec()
}

// conversion
impl From<&NodeStyle> for UiStyleBits {
    fn from(style: &NodeStyle) -> Self {
        let aspect_ratio = style.aspect_ratio().map(|(width, height)| {
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

            let width = width as u8;
            let height = height as u8;

            (width, height)
        });

        Self {
            parent_style: style.parent_style.map(|val| val.as_usize() as u8),
            widget_style: From::from(&style.base.widget_style),

            position_type: style.base.position_type.map(From::from),

            width: style.base.width.map(From::from),
            height: style.base.height.map(From::from),
            width_min: style.base.width_min.map(From::from),
            width_max: style.base.width_max.map(From::from),
            height_min: style.base.height_min.map(From::from),
            height_max: style.base.height_max.map(From::from),

            margin_left: style.base.margin_left.map(From::from),
            margin_right: style.base.margin_right.map(From::from),
            margin_top: style.base.margin_top.map(From::from),
            margin_bottom: style.base.margin_bottom.map(From::from),

            solid_override: style.base.solid_override.map(From::from),
            aspect_ratio,

            self_halign: style.base.self_halign.map(From::from),
            self_valign: style.base.self_valign.map(From::from),
        }
    }
}

impl From<&WidgetStyle> for WidgetStyleBits {
    fn from(style: &WidgetStyle) -> Self {
        match style {
            WidgetStyle::Panel(panel) => Self::Panel(From::from(panel)),
            WidgetStyle::Text(text) => Self::Text(From::from(text)),
            WidgetStyle::Button(button) => Self::Button(From::from(button)),
            WidgetStyle::Textbox(textbox) => Self::Textbox(From::from(textbox)),
        }
    }
}

impl From<&PanelStyle> for PanelStyleBits {
    fn from(style: &PanelStyle) -> Self {
        Self {
            is_viewport: style.is_viewport,
            
            background_color: style.background_color.map(From::from),
            background_alpha: style.background_alpha().map(bits_from_alpha),

            layout_type: style.layout_type.map(From::from),

            padding_left: style.padding_left.map(From::from),
            padding_right: style.padding_right.map(From::from),
            padding_top: style.padding_top.map(From::from),
            padding_bottom: style.padding_bottom.map(From::from),

            row_between: style.row_between.map(From::from),
            col_between: style.col_between.map(From::from),
            children_halign: style.children_halign.map(From::from),
            children_valign: style.children_valign.map(From::from),
        }
    }
}

impl From<&TextStyle> for TextStyleBits {
    fn from(style: &TextStyle) -> Self {
        Self {
            background_color: style.background_color.map(From::from),
            background_alpha: style.background_alpha().map(bits_from_alpha),
        }
    }
}

impl From<&ButtonStyle> for ButtonStyleBits {
    fn from(style: &ButtonStyle) -> Self {
        Self {
            panel: From::from(&style.panel),
            hover_color: style.hover_color.map(From::from),
            down_color: style.down_color.map(From::from),
        }
    }
}

impl From<&TextboxStyle> for TextboxStyleBits {
    fn from(style: &TextboxStyle) -> Self {
        Self {
            background_color: style.background_color.map(From::from),
            background_alpha: style.background_alpha().map(bits_from_alpha),
            hover_color: style.hover_color.map(From::from),
            active_color: style.active_color.map(From::from),
            select_color: style.select_color.map(From::from),
        }
    }
}

impl From<PositionType> for PositionTypeBits {
    fn from(position_type: PositionType) -> Self {
        match position_type {
            PositionType::Absolute => Self::Absolute,
            PositionType::Relative => Self::Relative,
        }
    }
}

impl From<SizeUnits> for SizeUnitsBits {
    fn from(size_units: SizeUnits) -> Self {
        match size_units {
            SizeUnits::Percentage(val) => {
                // validate
                if val < 0.0 || val > 100.0 {
                    panic!(
                        "SizeUnits::Percentage value must be between 0 and 100, got: {}",
                        val
                    );
                }
                if val.fract() != 0.0 {
                    panic!(
                        "SizeUnits::Percentage value must be a whole number, got: {}",
                        val
                    );
                }

                let val = val as u64;
                let val = UnsignedInteger::<7>::new(val);

                Self::Percent(val)
            }
            SizeUnits::Viewport(val) => {
                // validate
                if val < 0.0 || val > 100.0 {
                    panic!(
                        "SizeUnits::Viewport value must be between 0 and 100, got: {}",
                        val
                    );
                }
                if (val * 10.0).fract() != 0.0 {
                    panic!(
                        "SizeUnits::Viewport value must have at most 1 decimal spot, got: {}",
                        val
                    );
                }

                let val = (val*10.0) as u64;
                let val = UnsignedInteger::<10>::new(val);

                Self::Viewport(val)
            }
            SizeUnits::Auto => Self::Auto,
        }
    }
}

impl From<MarginUnits> for MarginUnitsBits {
    fn from(margin_units: MarginUnits) -> Self {
        match margin_units {
            MarginUnits::Percentage(val) => {
                // validate
                if val < 0.0 || val > 100.0 {
                    panic!(
                        "SizeUnits::Percentage value must be between 0 and 100, got: {}",
                        val
                    );
                }
                if val.fract() != 0.0 {
                    panic!(
                        "SizeUnits::Percentage value must be a whole number, got: {}",
                        val
                    );
                }

                let val = val as u64;
                let val = UnsignedInteger::<7>::new(val);

                Self::Percent(val)
            }
            MarginUnits::Viewport(val) => {
                // validate
                if val < 0.0 || val > 100.0 {
                    panic!(
                        "SizeUnits::Viewport value must be between 0 and 100, got: {}",
                        val
                    );
                }
                if (val * 10.0).fract() != 0.0 {
                    panic!(
                        "SizeUnits::Viewport value must be a whole number, got: {}",
                        val
                    );
                }

                let val = (val * 10.0) as u64;
                let val = UnsignedInteger::<10>::new(val);

                Self::Viewport(val)
            }
        }
    }
}

impl From<Solid> for SolidBits {
    fn from(solid: Solid) -> Self {
        match solid {
            Solid::Fit => Self::Fit,
            Solid::Fill => Self::Fill,
        }
    }
}

impl From<Alignment> for AlignmentBits {
    fn from(alignment: Alignment) -> Self {
        match alignment {
            Alignment::Start => Self::Start,
            Alignment::Center => Self::Center,
            Alignment::End => Self::End,
        }
    }
}

impl From<LayoutType> for LayoutTypeBits {
    fn from(layout_type: LayoutType) -> Self {
        match layout_type {
            LayoutType::Row => Self::Row,
            LayoutType::Column => Self::Column,
        }
    }
}

impl From<Color> for ColorBits {
    fn from(color: Color) -> Self {
        Self {
            r: color.r,
            g: color.g,
            b: color.b,
        }
    }
}

impl UiNodeBits {
    fn from_node(
        ui_config: &UiConfig,
        style_id_to_index: &HashMap<StyleId, u8>,
        node: &UiNode,
    ) -> Self {
        let mut me = Self {
            style_id: None,
            widget: WidgetBits::from_widget(ui_config, &node.widget),
        };

        if let Some(style_id) = &node.style_id() {
            let style_index = *style_id_to_index.get(style_id).unwrap();
            me.style_id = Some(style_index);
        }

        me
    }
}

impl WidgetBits {
    fn from_widget(ui_config: &UiConfig, widget: &Widget) -> Self {
        match widget {
            Widget::Panel(panel) => Self::Panel(From::from(panel)),
            Widget::Text(text) => Self::Text(From::from(text)),
            Widget::Button(button) => Self::Button(ButtonBits::from_button(ui_config, button)),
            Widget::Textbox(textbox) => {
                Self::Textbox(TextboxBits::from_textbox(ui_config, textbox))
            }
        }
    }
}

impl From<&Panel> for PanelBits {
    fn from(panel: &Panel) -> Self {
        let mut me = Self {
            children: Vec::new(),
        };
        for child_id in panel.children.iter() {
            let child_id = child_id.as_usize();
            if child_id >= u8::MAX as usize {
                panic!("Too many children nodes, max is {}", u8::MAX);
            }
            let child_id = child_id as u8;
            me.children.push(child_id);
        }
        me
    }
}

impl From<&Text> for TextBits {
    fn from(text: &Text) -> Self {
        Self {
            text: text.inner_text().to_string(),
        }
    }
}

impl ButtonBits {
    fn from_button(ui_config: &UiConfig, button: &Button) -> Self {
        let panel_bits = From::from(&button.panel);
        let nav_bits = NavigationBits::from_navigation(ui_config, &button.navigation);
        Self {
            panel: panel_bits,
            id_str: button.id_str.clone(),
            navigation: nav_bits,
        }
    }
}

impl TextboxBits {
    fn from_textbox(ui_config: &UiConfig, textbox: &Textbox) -> Self {
        Self {
            id_str: textbox.id_str.clone(),
            navigation: NavigationBits::from_navigation(ui_config, &textbox.navigation),
            is_password: textbox.is_password,
        }
    }
}

impl NavigationBits {
    fn from_navigation(ui_config: &UiConfig, navigation: &Navigation) -> Self {
        let Navigation {
            up_goes_to,
            down_goes_to,
            left_goes_to,
            right_goes_to,
            tab_goes_to,
        } = navigation;

        let up = get_nav_id(ui_config, up_goes_to.as_ref().map(|s| s.as_str()));
        let down = get_nav_id(ui_config, down_goes_to.as_ref().map(|s| s.as_str()));
        let left = get_nav_id(ui_config, left_goes_to.as_ref().map(|s| s.as_str()));
        let right = get_nav_id(ui_config, right_goes_to.as_ref().map(|s| s.as_str()));
        let tab = get_nav_id(ui_config, tab_goes_to.as_ref().map(|s| s.as_str()));

        Self {
            up,
            down,
            left,
            right,
            tab,
        }
    }
}

fn get_nav_id(ui_config: &UiConfig, id_str: Option<&str>) -> Option<UnsignedVariableInteger<4>> {
    let id_str = id_str?;
    let id = ui_config.get_node_id_by_id_str(id_str)?;
    let id = id.as_usize();
    Some(UnsignedVariableInteger::<4>::new(id as i128))
}

fn bits_from_alpha(val: f32) -> UnsignedInteger<4> {
    let val = (val * 10.0) as u8;
    UnsignedInteger::<4>::new(val)
}
