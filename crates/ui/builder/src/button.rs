use render_api::base::Color;
use ui_layout::{Alignment, LayoutType, MarginUnits, PositionType, SizeUnits, Solid};
use ui_types::{Button, ButtonStyle, NodeId, NodeStyle, Panel, StyleId, Text, UiConfig, UiNode, UiStore, Widget, WidgetStyle};

use crate::{PanelMut, TextMut};

pub struct ButtonMut<'a> {
    ui_config: &'a mut UiConfig,
    node_id: NodeId,
}

impl<'a> ButtonMut<'a> {
    pub(crate) fn new(ui_config: &'a mut UiConfig, node_id: NodeId) -> Self {
        Self { ui_config, node_id }
    }

    pub fn set_as_first_input(&mut self) -> &mut Self {
        self.ui_config.set_first_input(self.node_id);
        self
    }

    pub fn add_style(&mut self, style_id: StyleId) -> &mut Self {
        let node = self.ui_config.node_mut(&self.node_id).unwrap();
        node.style_ids.push(style_id);
        self
    }

    pub fn contents(&'a mut self, inner_fn: impl FnOnce(&mut ButtonContentsMut)) -> &mut Self {
        let mut context = ButtonContentsMut::new(self.ui_config, self.node_id);
        inner_fn(&mut context);
        self
    }

    pub fn navigation(&'a mut self, inner_fn: impl FnOnce(&mut ButtonNavigationMut)) -> &mut Self {
        let mut context = ButtonNavigationMut::new(self.ui_config, self.node_id);
        inner_fn(&mut context);
        self
    }

    pub fn to_panel_mut(&mut self) -> PanelMut {
        PanelMut::new(self.ui_config, self.node_id)
    }
}

// only used for adding children
pub struct ButtonContentsMut<'a> {
    ui_config: &'a mut UiConfig,
    node_id: NodeId,
}

impl<'a> ButtonContentsMut<'a> {
    pub(crate) fn new(ui_config: &'a mut UiConfig, node_id: NodeId) -> Self {
        Self { ui_config, node_id }
    }

    fn get_mut(&mut self) -> &mut UiNode {
        self.ui_config.node_mut(&self.node_id).unwrap()
    }

    fn get_button_mut(&mut self) -> &mut Button {
        self.get_mut().widget_button_mut().unwrap()
    }

    pub fn add_panel<'b>(self: &'b mut ButtonContentsMut<'a>) -> PanelMut<'b> {
        // creates a new panel, returning a context for it
        let new_id = self.ui_config.create_node(Widget::Panel(Panel::new()));

        // add new panel to children
        self.get_button_mut().add_child(new_id);

        PanelMut::<'b>::new(self.ui_config, new_id)
    }

    pub fn add_text<'b>(self: &'b mut ButtonContentsMut<'a>, text: &str) -> TextMut<'b> {
        // creates a new panel, returning a context for it
        let new_id = self.ui_config.create_node(Widget::Text(Text::new(text)));

        // add base text style
        let node_mut = self.ui_config.node_mut(&new_id).unwrap();
        node_mut.style_ids.push(UiConfig::BASE_TEXT_STYLE_ID);

        // add new text widget to children
        self.get_button_mut().add_child(new_id);

        TextMut::<'b>::new(self.ui_config, new_id)
    }

    // no `add_button` for buttons-in-buttons ...
}

pub struct ButtonNavigationMut<'a> {
    ui_config: &'a mut UiConfig,
    node_id: NodeId,
}

impl<'a> ButtonNavigationMut<'a> {
    pub(crate) fn new(ui_config: &'a mut UiConfig, node_id: NodeId) -> Self {
        Self { ui_config, node_id }
    }

    fn get_mut(&mut self) -> &mut UiNode {
        self.ui_config.node_mut(&self.node_id).unwrap()
    }

    fn get_button_mut(&mut self) -> &mut Button {
        self.get_mut().widget_button_mut().unwrap()
    }

    pub fn left_goes_to(&mut self, name: &str) -> &mut Self {
        self.get_button_mut().navigation.left_goes_to = Some(name.to_string());
        self
    }

    pub fn right_goes_to(&mut self, name: &str) -> &mut Self {
        self.get_button_mut().navigation.right_goes_to = Some(name.to_string());
        self
    }

    pub fn up_goes_to(&mut self, name: &str) -> &mut Self {
        self.get_button_mut().navigation.up_goes_to = Some(name.to_string());
        self
    }

    pub fn down_goes_to(&mut self, name: &str) -> &mut Self {
        self.get_button_mut().navigation.down_goes_to = Some(name.to_string());
        self
    }

    pub fn tab_goes_to(&mut self, name: &str) -> &mut Self {
        self.get_button_mut().navigation.tab_goes_to = Some(name.to_string());
        self
    }
}

pub struct ButtonStyleRef<'a> {
    store: &'a UiStore,
    node_id: NodeId,
}

impl<'a> ButtonStyleRef<'a> {
    pub fn new(store: &'a UiStore, node_id: NodeId) -> Self {
        Self { store, node_id }
    }

    pub fn background_color(&self) -> Color {
        let mut output = Color::BLACK; // TODO: put into const var!

        self.store.for_each_button_style(&self.node_id, |style| {
            if let Some(color) = style.panel.background_color {
                output = color;
            }
        });

        output
    }

    pub fn background_alpha(&self) -> f32 {
        let mut output = 1.0; // TODO: put into const var!

        self.store.for_each_button_style(&self.node_id, |style| {
            if let Some(alpha) = style.panel.background_alpha {
                output = alpha;
            }
        });

        output
    }

    pub fn hover_color(&self) -> Color {
        let mut output = Color::BLACK; // TODO: put into const var!

        self.store.for_each_button_style(&self.node_id, |style| {
            if let Some(color) = style.hover_color {
                output = color;
            }
        });

        output
    }

    pub fn down_color(&self) -> Color {
        let mut output = Color::BLACK; // TODO: put into const var!

        self.store.for_each_button_style(&self.node_id, |style| {
            if let Some(color) = style.down_color {
                output = color;
            }
        });

        output
    }
}

pub struct ButtonStyleMut<'a> {
    ui_config: &'a mut UiConfig,
    style_id: StyleId,
}

impl<'a> ButtonStyleMut<'a> {
    pub(crate) fn new(ui_config: &'a mut UiConfig, style_id: StyleId) -> Self {
        Self { ui_config, style_id }
    }

    fn get_style_mut(&mut self) -> &mut NodeStyle {
        self.ui_config.style_mut(&self.style_id).unwrap()
    }

    fn get_button_style_mut(&mut self) -> &mut ButtonStyle {
        if let WidgetStyle::Button(button_style) = &mut self.get_style_mut().widget_style {
            button_style
        } else {
            panic!("StyleId does not reference a ButtonStyle");
        }
    }

    // setters

    pub fn set_hover_color(&mut self, color: Color) -> &mut Self {
        self.get_button_style_mut().set_hover_color(color);
        self
    }

    pub fn set_down_color(&mut self, color: Color) -> &mut Self {
        self.get_button_style_mut().set_down_color(color);
        self
    }

    pub fn set_background_color(&mut self, color: Color) -> &mut Self {
        self.get_button_style_mut().panel.background_color = Some(color);
        self
    }

    pub fn set_background_alpha(&mut self, alpha: f32) -> &mut Self {
        self.get_button_style_mut().set_background_alpha(alpha);
        self
    }

    pub fn set_horizontal(&mut self) -> &mut Self {
        self.get_button_style_mut().panel.layout_type = Some(LayoutType::Row);
        self
    }

    pub fn set_vertical(&mut self) -> &mut Self {
        self.get_button_style_mut().panel.layout_type = Some(LayoutType::Column);
        self
    }

    pub fn set_absolute(&mut self) -> &mut Self {
        self.get_style_mut().position_type = Some(PositionType::Absolute);
        self
    }

    pub fn set_relative(&mut self) -> &mut Self {
        self.get_style_mut().position_type = Some(PositionType::Relative);
        self
    }

    pub fn set_self_halign(&mut self, align: Alignment) -> &mut Self {
        self.get_style_mut().self_halign = Some(align);
        self
    }

    pub fn set_self_valign(&mut self, align: Alignment) -> &mut Self {
        self.get_style_mut().self_valign = Some(align);
        self
    }

    pub fn set_children_halign(&mut self, align: Alignment) -> &mut Self {
        self.get_button_style_mut().panel.children_halign = Some(align);
        self
    }

    pub fn set_children_valign(&mut self, align: Alignment) -> &mut Self {
        self.get_button_style_mut().panel.children_valign = Some(align);
        self
    }

    // set_width
    fn set_width_units(&mut self, width: SizeUnits) -> &mut Self {
        self.get_style_mut().width = Some(width);
        self
    }

    pub fn set_width_auto(&mut self) -> &mut Self {
        self.set_width_units(SizeUnits::Auto)
    }

    pub fn set_width_px(&mut self, width_px: f32) -> &mut Self {
        self.set_width_units(SizeUnits::Pixels(width_px))
    }

    pub fn set_width_pc(&mut self, width_pc: f32) -> &mut Self {
        self.set_width_units(SizeUnits::Percentage(width_pc))
    }

    pub fn set_width_vp(&mut self, width_vp: f32) -> &mut Self {
        self.set_width_units(SizeUnits::Viewport(width_vp))
    }

    // set height
    fn set_height_units(&mut self, height: SizeUnits) -> &mut Self {
        self.get_style_mut().height = Some(height);
        self
    }

    pub fn set_height_auto(&mut self) -> &mut Self {
        self.set_height_units(SizeUnits::Auto)
    }

    pub fn set_height_px(&mut self, width_px: f32) -> &mut Self {
        self.set_height_units(SizeUnits::Pixels(width_px))
    }

    pub fn set_height_pc(&mut self, width_pc: f32) -> &mut Self {
        self.set_height_units(SizeUnits::Percentage(width_pc))
    }

    pub fn set_height_vp(&mut self, width_vp: f32) -> &mut Self {
        self.set_height_units(SizeUnits::Viewport(width_vp))
    }

    // set size
    fn set_size_units(&mut self, width: SizeUnits, height: SizeUnits) -> &mut Self {
        self.set_width_units(width);
        self.set_height_units(height);
        self
    }

    pub fn set_size_auto(&mut self) -> &mut Self {
        self.set_size_units(SizeUnits::Auto, SizeUnits::Auto)
    }

    pub fn set_size_px(&mut self, width_px: f32, height_px: f32) -> &mut Self {
        self.set_size_units(SizeUnits::Pixels(width_px), SizeUnits::Pixels(height_px))
    }

    pub fn set_size_pc(&mut self, width_pc: f32, height_pc: f32) -> &mut Self {
        self.set_size_units(
            SizeUnits::Percentage(width_pc),
            SizeUnits::Percentage(height_pc),
        )
    }

    pub fn set_size_vp(&mut self, width_vp: f32, height_vp: f32) -> &mut Self {
        self.set_size_units(
            SizeUnits::Viewport(width_vp),
            SizeUnits::Viewport(height_vp),
        )
    }

    // set_width_min
    fn set_width_min_units(&mut self, min_width: SizeUnits) -> &mut Self {
        self.get_style_mut().width_min = Some(min_width);
        self
    }

    pub fn set_width_min_auto(&mut self) -> &mut Self {
        self.set_width_min_units(SizeUnits::Auto)
    }

    pub fn set_width_min_px(&mut self, min_width_px: f32) -> &mut Self {
        self.set_width_min_units(SizeUnits::Pixels(min_width_px))
    }

    pub fn set_width_min_pc(&mut self, min_width_pc: f32) -> &mut Self {
        self.set_width_min_units(SizeUnits::Percentage(min_width_pc))
    }

    pub fn set_width_min_vp(&mut self, min_width_vp: f32) -> &mut Self {
        self.set_width_min_units(SizeUnits::Viewport(min_width_vp))
    }

    // set_height_min
    fn set_height_min_units(&mut self, min_height: SizeUnits) -> &mut Self {
        self.get_style_mut().height_min = Some(min_height);
        self
    }

    pub fn set_height_min_auto(&mut self) -> &mut Self {
        self.set_height_min_units(SizeUnits::Auto)
    }

    pub fn set_height_min_px(&mut self, min_height_px: f32) -> &mut Self {
        self.set_height_min_units(SizeUnits::Pixels(min_height_px))
    }

    pub fn set_height_min_pc(&mut self, min_height_pc: f32) -> &mut Self {
        self.set_height_min_units(SizeUnits::Percentage(min_height_pc))
    }

    pub fn set_height_min_vp(&mut self, min_height_vp: f32) -> &mut Self {
        self.set_height_min_units(SizeUnits::Viewport(min_height_vp))
    }

    // set_size_min
    fn set_size_min_units(&mut self, min_width: SizeUnits, min_height: SizeUnits) -> &mut Self {
        self.set_width_min_units(min_width);
        self.set_height_min_units(min_height);
        self
    }

    pub fn set_size_min_auto(&mut self) -> &mut Self {
        self.set_size_min_units(SizeUnits::Auto, SizeUnits::Auto)
    }

    pub fn set_size_min_px(&mut self, min_width_px: f32, min_height_px: f32) -> &mut Self {
        self.set_size_min_units(
            SizeUnits::Pixels(min_width_px),
            SizeUnits::Pixels(min_height_px),
        )
    }

    pub fn set_size_min_pc(&mut self, min_width_pc: f32, min_height_pc: f32) -> &mut Self {
        self.set_size_min_units(
            SizeUnits::Percentage(min_width_pc),
            SizeUnits::Percentage(min_height_pc),
        )
    }

    pub fn set_size_min_vp(&mut self, min_width_vp: f32, min_height_vp: f32) -> &mut Self {
        self.set_size_min_units(
            SizeUnits::Viewport(min_width_vp),
            SizeUnits::Viewport(min_height_vp),
        )
    }

    // set_width_max
    fn set_width_max_units(&mut self, max_width: SizeUnits) -> &mut Self {
        self.get_style_mut().width_max = Some(max_width);
        self
    }

    pub fn set_width_max_auto(&mut self) -> &mut Self {
        self.set_width_max_units(SizeUnits::Auto)
    }

    pub fn set_width_max_px(&mut self, max_width_px: f32) -> &mut Self {
        self.set_width_max_units(SizeUnits::Pixels(max_width_px))
    }

    pub fn set_width_max_pc(&mut self, max_width_pc: f32) -> &mut Self {
        self.set_width_max_units(SizeUnits::Percentage(max_width_pc))
    }

    pub fn set_width_max_vp(&mut self, max_width_vp: f32) -> &mut Self {
        self.set_width_max_units(SizeUnits::Viewport(max_width_vp))
    }

    // set_height_max
    fn set_height_max_units(&mut self, max_height: SizeUnits) -> &mut Self {
        self.get_style_mut().height_max = Some(max_height);
        self
    }

    pub fn set_height_max_auto(&mut self) -> &mut Self {
        self.set_height_max_units(SizeUnits::Auto)
    }

    pub fn set_height_max_px(&mut self, max_height_px: f32) -> &mut Self {
        self.set_height_max_units(SizeUnits::Pixels(max_height_px))
    }

    pub fn set_height_max_pc(&mut self, max_height_pc: f32) -> &mut Self {
        self.set_height_max_units(SizeUnits::Percentage(max_height_pc))
    }

    pub fn set_height_max_vp(&mut self, max_height_vp: f32) -> &mut Self {
        self.set_height_max_units(SizeUnits::Viewport(max_height_vp))
    }

    // set_size_max
    fn set_size_max_units(&mut self, max_width: SizeUnits, max_height: SizeUnits) -> &mut Self {
        self.set_width_max_units(max_width);
        self.set_height_max_units(max_height);
        self
    }

    pub fn set_size_max_auto(&mut self) -> &mut Self {
        self.set_size_max_units(SizeUnits::Auto, SizeUnits::Auto)
    }

    pub fn set_size_max_px(&mut self, max_width_px: f32, max_height_px: f32) -> &mut Self {
        self.set_size_max_units(
            SizeUnits::Pixels(max_width_px),
            SizeUnits::Pixels(max_height_px),
        )
    }

    pub fn set_size_max_pc(&mut self, max_width_pc: f32, max_height_pc: f32) -> &mut Self {
        self.set_size_max_units(
            SizeUnits::Percentage(max_width_pc),
            SizeUnits::Percentage(max_height_pc),
        )
    }

    pub fn set_size_max_vp(&mut self, max_width_vp: f32, max_height_vp: f32) -> &mut Self {
        self.set_size_max_units(
            SizeUnits::Viewport(max_width_vp),
            SizeUnits::Viewport(max_height_vp),
        )
    }

    // set_left
    fn set_margin_left_units(&mut self, left: MarginUnits) -> &mut Self {
        self.get_style_mut().margin_left = Some(left);
        self
    }

    pub fn set_margin_left_px(&mut self, left_px: f32) -> &mut Self {
        self.set_margin_left_units(MarginUnits::Pixels(left_px))
    }

    pub fn set_margin_left_pc(&mut self, left_pc: f32) -> &mut Self {
        self.set_margin_left_units(MarginUnits::Percentage(left_pc))
    }

    pub fn set_margin_left_vp(&mut self, left_vp: f32) -> &mut Self {
        self.set_margin_left_units(MarginUnits::Viewport(left_vp))
    }

    // set_right
    fn set_margin_right_units(&mut self, right: MarginUnits) -> &mut Self {
        self.get_style_mut().margin_right = Some(right);
        self
    }

    pub fn set_margin_right_px(&mut self, right_px: f32) -> &mut Self {
        self.set_margin_right_units(MarginUnits::Pixels(right_px))
    }

    pub fn set_margin_right_pc(&mut self, right_pc: f32) -> &mut Self {
        self.set_margin_right_units(MarginUnits::Percentage(right_pc))
    }

    pub fn set_margin_right_vp(&mut self, right_vp: f32) -> &mut Self {
        self.set_margin_right_units(MarginUnits::Viewport(right_vp))
    }

    // set_top
    fn set_margin_top_units(&mut self, top: MarginUnits) -> &mut Self {
        self.get_style_mut().margin_top = Some(top);
        self
    }

    pub fn set_margin_top_px(&mut self, top_px: f32) -> &mut Self {
        self.set_margin_top_units(MarginUnits::Pixels(top_px))
    }

    pub fn set_margin_top_pc(&mut self, top_pc: f32) -> &mut Self {
        self.set_margin_top_units(MarginUnits::Percentage(top_pc))
    }

    pub fn set_margin_top_vp(&mut self, top_vp: f32) -> &mut Self {
        self.set_margin_top_units(MarginUnits::Viewport(top_vp))
    }

    // set_bottom
    fn set_margin_bottom_units(&mut self, bottom: MarginUnits) -> &mut Self {
        self.get_style_mut().margin_bottom = Some(bottom);
        self
    }

    pub fn set_margin_bottom_px(&mut self, bottom_px: f32) -> &mut Self {
        self.set_margin_bottom_units(MarginUnits::Pixels(bottom_px))
    }

    pub fn set_margin_bottom_pc(&mut self, bottom_pc: f32) -> &mut Self {
        self.set_margin_bottom_units(MarginUnits::Percentage(bottom_pc))
    }

    pub fn set_margin_bottom_vp(&mut self, bottom_vp: f32) -> &mut Self {
        self.set_margin_bottom_units(MarginUnits::Viewport(bottom_vp))
    }

    // set_margin

    pub fn set_margin_px(&mut self, left: f32, right: f32, top: f32, bottom: f32) -> &mut Self {
        self.set_margin_left_px(left)
            .set_margin_right_px(right)
            .set_margin_top_px(top)
            .set_margin_bottom_px(bottom)
    }

    pub fn set_margin_pc(&mut self, left: f32, right: f32, top: f32, bottom: f32) -> &mut Self {
        self.set_margin_left_pc(left)
            .set_margin_right_pc(right)
            .set_margin_top_pc(top)
            .set_margin_bottom_pc(bottom)
    }

    pub fn set_margin_vp(&mut self, left: f32, right: f32, top: f32, bottom: f32) -> &mut Self {
        self.set_margin_left_vp(left)
            .set_margin_right_vp(right)
            .set_margin_top_vp(top)
            .set_margin_bottom_vp(bottom)
    }

    // set_padding_left
    fn set_padding_left_units(&mut self, child_left: SizeUnits) -> &mut Self {
        self.get_button_style_mut().panel.padding_left = Some(child_left);
        self
    }

    pub fn set_padding_left_auto(&mut self) -> &mut Self {
        self.set_padding_left_units(SizeUnits::Auto)
    }

    pub fn set_padding_left_px(&mut self, child_left_px: f32) -> &mut Self {
        self.set_padding_left_units(SizeUnits::Pixels(child_left_px))
    }

    pub fn set_padding_left_pc(&mut self, child_left_pc: f32) -> &mut Self {
        self.set_padding_left_units(SizeUnits::Percentage(child_left_pc))
    }

    pub fn set_padding_left_vp(&mut self, child_left_vp: f32) -> &mut Self {
        self.set_padding_left_units(SizeUnits::Viewport(child_left_vp))
    }

    // set_padding_right
    fn set_padding_right_units(&mut self, child_right: SizeUnits) -> &mut Self {
        self.get_button_style_mut().panel.padding_right = Some(child_right);
        self
    }

    pub fn set_padding_right_auto(&mut self) -> &mut Self {
        self.set_padding_right_units(SizeUnits::Auto)
    }

    pub fn set_padding_right_px(&mut self, child_right_px: f32) -> &mut Self {
        self.set_padding_right_units(SizeUnits::Pixels(child_right_px))
    }

    pub fn set_padding_right_pc(&mut self, child_right_pc: f32) -> &mut Self {
        self.set_padding_right_units(SizeUnits::Percentage(child_right_pc))
    }

    pub fn set_padding_right_vp(&mut self, child_right_vp: f32) -> &mut Self {
        self.set_padding_right_units(SizeUnits::Viewport(child_right_vp))
    }

    // set_padding_top
    fn set_padding_top_units(&mut self, child_top: SizeUnits) -> &mut Self {
        self.get_button_style_mut().panel.padding_top = Some(child_top);
        self
    }

    pub fn set_padding_top_auto(&mut self) -> &mut Self {
        self.set_padding_top_units(SizeUnits::Auto)
    }

    pub fn set_padding_top_px(&mut self, child_top_px: f32) -> &mut Self {
        self.set_padding_top_units(SizeUnits::Pixels(child_top_px))
    }

    pub fn set_padding_top_pc(&mut self, child_top_pc: f32) -> &mut Self {
        self.set_padding_top_units(SizeUnits::Percentage(child_top_pc))
    }

    pub fn set_padding_top_vp(&mut self, child_top_vp: f32) -> &mut Self {
        self.set_padding_top_units(SizeUnits::Viewport(child_top_vp))
    }

    // set_padding_bottom
    fn set_padding_bottom_units(&mut self, child_bottom: SizeUnits) -> &mut Self {
        self.get_button_style_mut().panel.padding_bottom = Some(child_bottom);
        self
    }

    pub fn set_padding_bottom_auto(&mut self) -> &mut Self {
        self.set_padding_bottom_units(SizeUnits::Auto)
    }

    pub fn set_padding_bottom_px(&mut self, child_bottom_px: f32) -> &mut Self {
        self.set_padding_bottom_units(SizeUnits::Pixels(child_bottom_px))
    }

    pub fn set_padding_bottom_pc(&mut self, child_bottom_pc: f32) -> &mut Self {
        self.set_padding_bottom_units(SizeUnits::Percentage(child_bottom_pc))
    }

    pub fn set_padding_bottom_vp(&mut self, child_bottom_vp: f32) -> &mut Self {
        self.set_padding_bottom_units(SizeUnits::Viewport(child_bottom_vp))
    }

    // set_padding
    pub fn set_padding_auto(&mut self) -> &mut Self {
        self.set_padding_left_auto()
            .set_padding_right_auto()
            .set_padding_top_auto()
            .set_padding_bottom_auto()
    }

    pub fn set_padding_px(&mut self, left: f32, right: f32, top: f32, bottom: f32) -> &mut Self {
        self.set_padding_left_px(left)
            .set_padding_right_px(right)
            .set_padding_top_px(top)
            .set_padding_bottom_px(bottom)
    }

    pub fn set_padding_pc(&mut self, left: f32, right: f32, top: f32, bottom: f32) -> &mut Self {
        self.set_padding_left_pc(left)
            .set_padding_right_pc(right)
            .set_padding_top_pc(top)
            .set_padding_bottom_pc(bottom)
    }

    pub fn set_padding_vp(&mut self, left: f32, right: f32, top: f32, bottom: f32) -> &mut Self {
        self.set_padding_left_vp(left)
            .set_padding_right_vp(right)
            .set_padding_top_vp(top)
            .set_padding_bottom_vp(bottom)
    }

    // set_row_between
    fn set_row_between_units(&mut self, row_between: SizeUnits) -> &mut Self {
        self.get_button_style_mut().panel.row_between = Some(row_between);
        self
    }

    pub fn set_row_between_auto(&mut self) -> &mut Self {
        self.set_row_between_units(SizeUnits::Auto)
    }

    pub fn set_row_between_px(&mut self, row_between_px: f32) -> &mut Self {
        self.set_row_between_units(SizeUnits::Pixels(row_between_px))
    }

    pub fn set_row_between_pc(&mut self, row_between_pc: f32) -> &mut Self {
        self.set_row_between_units(SizeUnits::Percentage(row_between_pc))
    }

    pub fn set_row_between_vp(&mut self, row_between_vp: f32) -> &mut Self {
        self.set_row_between_units(SizeUnits::Viewport(row_between_vp))
    }

    // set_col_between
    fn set_col_between_units(&mut self, column_between: SizeUnits) -> &mut Self {
        self.get_button_style_mut().panel.col_between = Some(column_between);
        self
    }

    pub fn set_col_between_auto(&mut self) -> &mut Self {
        self.set_col_between_units(SizeUnits::Auto)
    }

    pub fn set_col_between_px(&mut self, column_between_px: f32) -> &mut Self {
        self.set_col_between_units(SizeUnits::Pixels(column_between_px))
    }

    pub fn set_col_between_pc(&mut self, column_between_pc: f32) -> &mut Self {
        self.set_col_between_units(SizeUnits::Percentage(column_between_pc))
    }

    pub fn set_col_between_vp(&mut self, column_between_vp: f32) -> &mut Self {
        self.set_col_between_units(SizeUnits::Viewport(column_between_vp))
    }

    // solid stuff

    pub fn set_solid_fit(&mut self) -> &mut Self {
        self.get_style_mut().solid_override = Some(Solid::Fit);
        self
    }

    pub fn set_solid_fill(&mut self) -> &mut Self {
        self.get_style_mut().solid_override = Some(Solid::Fill);
        self
    }

    pub fn set_aspect_ratio(&mut self, width: f32, height: f32) -> &mut Self {
        self.get_style_mut().set_aspect_ratio(width, height);
        self
    }
}