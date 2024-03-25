use std::collections::HashMap;

use naia_serde::{FileBitWriter, SerdeInternal as Serde, UnsignedInteger, UnsignedVariableInteger};

use ui::{Alignment, Button, ButtonStyle, LayoutType, MarginUnits, NodeStyle, Panel, PanelStyle, PositionType, SizeUnits, Solid, StyleId, Text, TextStyle, Ui, UiNode, Widget, WidgetStyle, Navigation, TextboxStyle, Textbox};

use crate::bits::{AlignmentBits, ButtonBits, NavigationBits, ButtonStyleBits, LayoutTypeBits, MarginUnitsBits, PanelBits, PanelStyleBits, PositionTypeBits, SizeUnitsBits, SolidBits, TextBits, TextStyleBits, UiAction, UiActionType, UiNodeBits, UiStyleBits, WidgetBits, WidgetStyleBits, TextboxStyleBits, TextboxBits};

pub fn write_bits(ui: &Ui) -> Vec<u8> {
    let actions = convert_ui_to_actions(ui);
    actions_to_bytes(actions)
}

fn convert_ui_to_actions(ui: &Ui) -> Vec<UiAction> {
    let mut output = Vec::new();

    // write text color
    let text_color = ui.get_text_color();
    output.push(UiAction::TextColor(
        text_color.r,
        text_color.g,
        text_color.b,
    ));

    // write text icon AssetId
    let text_icon_asset_id = ui.get_text_icon_asset_id();
    output.push(UiAction::TextIconAssetId(*text_icon_asset_id));

    // write default button
    let default_button_id = ui.get_default_button();
    output.push(UiAction::DefaultButton(default_button_id));

    // write styles

    let mut style_id_to_index = HashMap::new();
    let mut style_count = 0;

    for (style_id, style) in ui.store.styles.iter().enumerate() {
        let style_id = StyleId::new(style_id as u32);
        if style_id == Ui::BASE_TEXT_STYLE_ID {
            continue;
        }
        let next_index = style_count;
        if style_count == u8::MAX {
            panic!("Too many styles, max is {}", u8::MAX);
        }
        style_count += 1;
        style_id_to_index.insert(style_id, next_index);

        output.push(UiAction::Style(UiStyleBits::from_style(style)));
    }

    // write nodes
    for node in ui.store.nodes.iter() {
        output.push(UiAction::Node(UiNodeBits::from_node(
            ui,
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
            UiAction::TextColor(r, g, b) => {
                UiActionType::TextColor.ser(&mut bit_writer);
                r.ser(&mut bit_writer);
                g.ser(&mut bit_writer);
                b.ser(&mut bit_writer);
            }
            UiAction::TextIconAssetId(asset_id) => {
                UiActionType::TextIconAssetId.ser(&mut bit_writer);
                asset_id.as_u32().ser(&mut bit_writer);
            }
            UiAction::DefaultButton(button_id_opt) => {
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
impl UiStyleBits {
    pub(crate) fn from_style(style: &NodeStyle) -> Self {
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
            widget_style: WidgetStyleBits::from_style(&style.widget_style),

            position_type: style
                .position_type
                .map(PositionTypeBits::from_position_type),

            width: style.width.map(SizeUnitsBits::from_size_units),
            height: style.height.map(SizeUnitsBits::from_size_units),
            width_min: style.width_min.map(SizeUnitsBits::from_size_units),
            width_max: style.width_max.map(SizeUnitsBits::from_size_units),
            height_min: style.height_min.map(SizeUnitsBits::from_size_units),
            height_max: style.height_max.map(SizeUnitsBits::from_size_units),

            margin_left: style.margin_left.map(MarginUnitsBits::from_margin_units),
            margin_right: style.margin_right.map(MarginUnitsBits::from_margin_units),
            margin_top: style.margin_top.map(MarginUnitsBits::from_margin_units),
            margin_bottom: style.margin_bottom.map(MarginUnitsBits::from_margin_units),

            solid_override: style.solid_override.map(SolidBits::from_solid),
            aspect_ratio,

            self_halign: style.self_halign.map(AlignmentBits::from_alignment),
            self_valign: style.self_valign.map(AlignmentBits::from_alignment),
        }
    }
}

impl WidgetStyleBits {
    fn from_style(style: &WidgetStyle) -> Self {
        match style {
            WidgetStyle::Panel(panel) => Self::Panel(PanelStyleBits::from_panel_style(panel)),
            WidgetStyle::Text(text) => Self::Text(TextStyleBits::from_text_style(text)),
            WidgetStyle::Button(button) => Self::Button(ButtonStyleBits::from_button_style(button)),
            WidgetStyle::Textbox(textbox) => Self::Textbox(TextboxStyleBits::from_textbox_style(textbox)),
        }
    }
}

impl PanelStyleBits {
    fn from_panel_style(style: &PanelStyle) -> Self {
        Self {
            background_color: style.background_color.map(|val| (val.r, val.g, val.b)),
            background_alpha: style.background_alpha().map(|val| {
                let val = (val * 10.0) as u8;
                UnsignedInteger::<4>::new(val)
            }),

            layout_type: style.layout_type.map(LayoutTypeBits::from_layout_type),

            padding_left: style.padding_left.map(SizeUnitsBits::from_size_units),
            padding_right: style.padding_right.map(SizeUnitsBits::from_size_units),
            padding_top: style.padding_top.map(SizeUnitsBits::from_size_units),
            padding_bottom: style.padding_bottom.map(SizeUnitsBits::from_size_units),

            row_between: style.row_between.map(SizeUnitsBits::from_size_units),
            col_between: style.col_between.map(SizeUnitsBits::from_size_units),
            children_halign: style.children_halign.map(AlignmentBits::from_alignment),
            children_valign: style.children_valign.map(AlignmentBits::from_alignment),
        }
    }
}

impl TextStyleBits {
    fn from_text_style(style: &TextStyle) -> Self {
        Self {
            background_color: style.background_color.map(|val| (val.r, val.g, val.b)),
            background_alpha: style.background_alpha().map(|val| {
                let val = (val * 10.0) as u8;
                UnsignedInteger::<4>::new(val)
            }),
        }
    }
}

impl ButtonStyleBits {
    fn from_button_style(style: &ButtonStyle) -> Self {
        Self {
            panel: PanelStyleBits::from_panel_style(&style.panel),
            hover_color: style.hover_color.map(|val| (val.r, val.g, val.b)),
            down_color: style.down_color.map(|val| (val.r, val.g, val.b)),
        }
    }
}

impl TextboxStyleBits {
    fn from_textbox_style(style: &TextboxStyle) -> Self {
        Self {
            panel: PanelStyleBits::from_panel_style(&style.panel),
            hover_color: style.hover_color.map(|val| (val.r, val.g, val.b)),
            active_color: style.active_color.map(|val| (val.r, val.g, val.b)),
        }
    }
}

impl PositionTypeBits {
    fn from_position_type(position_type: PositionType) -> Self {
        match position_type {
            PositionType::Absolute => Self::Absolute,
            PositionType::Relative => Self::Relative,
        }
    }
}

impl SizeUnitsBits {
    fn from_size_units(size_units: SizeUnits) -> Self {
        match size_units {
            SizeUnits::Pixels(val) => {
                // validate
                if val.fract() != 0.0 {
                    panic!(
                        "SizeUnits::Pixels value must be a whole number, got: {}",
                        val
                    );
                }
                if val < 0.0 {
                    panic!("SizeUnits::Pixels value must be positive, got: {}", val);
                }

                let val = val as u64;
                let val = UnsignedVariableInteger::<7>::new(val);

                Self::Pixels(val)
            }
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
                if val.fract() != 0.0 {
                    panic!(
                        "SizeUnits::Viewport value must be a whole number, got: {}",
                        val
                    );
                }

                let val = val as u64;
                let val = UnsignedInteger::<7>::new(val);

                Self::Viewport(val)
            }
            SizeUnits::Auto => Self::Auto,
        }
    }
}

impl MarginUnitsBits {
    fn from_margin_units(margin_units: MarginUnits) -> Self {
        match margin_units {
            MarginUnits::Pixels(val) => {
                // validate
                if val.fract() != 0.0 {
                    panic!(
                        "SizeUnits::Pixels value must be a whole number, got: {}",
                        val
                    );
                }
                if val < 0.0 {
                    panic!("SizeUnits::Pixels value must be positive, got: {}", val);
                }

                let val = val as u64;
                let val = UnsignedVariableInteger::<7>::new(val);

                Self::Pixels(val)
            }
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
                if val.fract() != 0.0 {
                    panic!(
                        "SizeUnits::Viewport value must be a whole number, got: {}",
                        val
                    );
                }

                let val = val as u64;
                let val = UnsignedInteger::<7>::new(val);

                Self::Viewport(val)
            }
        }
    }
}

impl SolidBits {
    fn from_solid(solid: Solid) -> Self {
        match solid {
            Solid::Fit => Self::Fit,
            Solid::Fill => Self::Fill,
        }
    }
}

impl AlignmentBits {
    fn from_alignment(alignment: Alignment) -> Self {
        match alignment {
            Alignment::Start => Self::Start,
            Alignment::Center => Self::Center,
            Alignment::End => Self::End,
        }
    }
}

impl LayoutTypeBits {
    fn from_layout_type(layout_type: LayoutType) -> Self {
        match layout_type {
            LayoutType::Row => Self::Row,
            LayoutType::Column => Self::Column,
        }
    }
}

impl UiNodeBits {
    fn from_node(ui: &Ui, style_id_to_index: &HashMap<StyleId, u8>, node: &UiNode) -> Self {
        let mut me = Self {
            visible: node.visible,
            style_ids: Vec::new(),
            widget: WidgetBits::from_widget(ui, &node.widget),
        };

        for style_id in &node.style_ids {
            if style_id == &Ui::BASE_TEXT_STYLE_ID {
                continue;
            }
            let style_index = *style_id_to_index.get(style_id).unwrap();
            me.style_ids.push(style_index);
        }

        me
    }
}

impl WidgetBits {
    fn from_widget(ui: &Ui, widget: &Widget) -> Self {
        match widget {
            Widget::Panel(panel) => Self::Panel(PanelBits::from_panel(panel)),
            Widget::Text(text) => Self::Text(TextBits::from_text(text)),
            Widget::Button(button) => Self::Button(ButtonBits::from_button(ui, button)),
            Widget::Textbox(textbox) => Self::Textbox(TextboxBits::from_textbox(ui, textbox)),
        }
    }
}

impl PanelBits {
    fn from_panel(panel: &Panel) -> Self {
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

impl TextBits {
    fn from_text(text: &Text) -> Self {
        Self {
            text: text.inner_text().to_string(),
        }
    }
}

impl ButtonBits {
    fn from_button(ui: &Ui, button: &Button) -> Self {
        let panel_bits = PanelBits::from_panel(&button.panel);
        let nav_bits = NavigationBits::from_navigation(ui, &button.navigation);
        Self {
            panel: panel_bits,
            id_str: button.id_str.clone(),
            navigation: nav_bits,
        }
    }
}

impl TextboxBits {
    fn from_textbox(ui: &Ui, textbox: &Textbox) -> Self {
        let panel_bits = PanelBits::from_panel(&textbox.panel);
        let nav_bits = NavigationBits::from_navigation(ui, &textbox.navigation);
        Self {
            panel: panel_bits,
            id_str: textbox.id_str.clone(),
            navigation: nav_bits,
        }
    }
}

impl NavigationBits {
    fn from_navigation(ui: &Ui, navigation: &Navigation) -> Self {
        let Navigation {
            up_goes_to,
            down_goes_to,
            left_goes_to,
            right_goes_to,
        } = navigation;

        let up = get_nav_id(ui, up_goes_to.as_ref().map(|s| s.as_str()));
        let down = get_nav_id(ui, down_goes_to.as_ref().map(|s| s.as_str()));
        let left = get_nav_id(ui, left_goes_to.as_ref().map(|s| s.as_str()));
        let right = get_nav_id(ui, right_goes_to.as_ref().map(|s| s.as_str()));

        Self {
            up,
            down,
            left,
            right,
        }
    }
}

fn get_nav_id(ui: &Ui, id_str: Option<&str>) -> Option<UnsignedVariableInteger<4>> {
    let id_str = id_str?;
    let id = ui.get_node_id_by_id_str(id_str)?;
    let id = id.as_usize();
    Some(UnsignedVariableInteger::<4>::new(id as i128))
}