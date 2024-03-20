
use render_api::{
    base::{Color, CpuMaterial},
};
use storage::Handle;
use ui_layout::{Alignment, LayoutType, MarginUnits, PositionType, SizeUnits, Solid};

use crate::{store::UiStore, node::{UiNode}, style::{NodeStyle, StyleId, WidgetStyle}, text::{Text, TextMut}, NodeId, Ui, Widget, Button, ButtonMut};

#[derive(Clone)]
pub struct Panel {
    pub children: Vec<NodeId>,
    pub background_color_handle: Option<Handle<CpuMaterial>>,
}

impl Panel {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            background_color_handle: None,
        }
    }

    pub fn add_child(&mut self, child_id: NodeId) {
        self.children.push(child_id);
    }
}

#[derive(Clone, Copy)]
pub struct PanelStyle {
    pub background_color: Option<Color>,
    pub(crate) background_alpha: Option<f32>,

    pub layout_type: Option<LayoutType>,

    pub padding_left: Option<SizeUnits>,
    pub padding_right: Option<SizeUnits>,
    pub padding_top: Option<SizeUnits>,
    pub padding_bottom: Option<SizeUnits>,

    pub row_between: Option<SizeUnits>,
    pub col_between: Option<SizeUnits>,
    pub children_halign: Option<Alignment>,
    pub children_valign: Option<Alignment>,
}

impl PanelStyle {
    pub(crate) fn empty() -> Self {
        Self {
            background_color: None,
            background_alpha: None,

            layout_type: None,

            padding_left: None,
            padding_right: None,
            padding_top: None,
            padding_bottom: None,

            row_between: None,
            col_between: None,
            children_halign: None,
            children_valign: None,
        }
    }

    pub fn background_alpha(&self) -> Option<f32> {
        self.background_alpha
    }

    pub(crate) fn set_background_alpha(&mut self, val: f32) {
        // validate
        if val < 0.0 || val > 1.0 {
            panic!("background_alpha must be between 0.0 and 1.0");
        }
        if (val * 10.0).fract() != 0.0 {
            panic!("background_alpha must be a multiple of 0.1");
        }

        self.background_alpha = Some(val);
    }
}

pub struct PanelMut<'a> {
    ui: &'a mut Ui,
    node_id: NodeId,
}

impl<'a> PanelMut<'a> {
    pub(crate) fn new(ui: &'a mut Ui, node_id: NodeId) -> Self {
        Self { ui, node_id }
    }

    pub fn set_visible(&mut self, visible: bool) -> &mut Self {
        if let Some(panel) = self.ui.node_mut(&self.node_id) {
            panel.visible = visible;
        }
        self
    }

    // pub fn style(&mut self, inner_fn: impl FnOnce(&mut PanelStyleMut)) -> &mut Self {
    //     let mut style_mut = PanelStyleMut::new(self.ui, self.node_id);
    //     inner_fn(&mut style_mut);
    //     self
    // }

    pub fn add_style(&mut self, style_id: StyleId) -> &mut Self {
        let node = self.ui.node_mut(&self.node_id).unwrap();
        node.style_ids.push(style_id);
        self
    }

    pub fn contents(&'a mut self, inner_fn: impl FnOnce(&mut PanelContentsMut)) -> &mut Self {
        let mut context = PanelContentsMut::new(self.ui, self.node_id);
        inner_fn(&mut context);
        self
    }
}

// only used for adding children
pub struct PanelContentsMut<'a> {
    ui: &'a mut Ui,
    node_id: NodeId,
}

impl<'a> PanelContentsMut<'a> {
    pub(crate) fn new(ui: &'a mut Ui, node_id: NodeId) -> Self {
        Self { ui, node_id }
    }

    fn get_mut(&mut self) -> &mut UiNode {
        self.ui.node_mut(&self.node_id).unwrap()
    }

    fn get_panel_mut(&mut self) -> &mut Panel {
        self.get_mut().widget_panel_mut().unwrap()
    }

    pub fn add_panel<'b>(self: &'b mut PanelContentsMut<'a>) -> PanelMut<'b> {
        // creates a new panel, returning a context for it
        let new_id = self.ui.create_node(Widget::Panel(Panel::new()));

        // add new panel to children
        self.get_panel_mut().add_child(new_id);

        PanelMut::<'b>::new(self.ui, new_id)
    }

    pub fn add_text<'b>(self: &'b mut PanelContentsMut<'a>, text: &str) -> TextMut<'b> {
        // creates a new panel, returning a context for it
        let new_id = self.ui.create_node(Widget::Text(Text::new(text)));

        // add base text style
        let node_mut = self.ui.node_mut(&new_id).unwrap();
        node_mut.style_ids.push(Ui::BASE_TEXT_STYLE_ID);

        // add new panel to children
        self.get_panel_mut().add_child(new_id);

        TextMut::<'b>::new(self.ui, new_id)
    }

    pub fn add_button<'b>(self: &'b mut PanelContentsMut<'a>, button_id_str: &str) -> ButtonMut<'b> {
        // creates a new button, returning a context for it
        let new_id = self.ui.create_node(Widget::Button(Button::new(button_id_str)));

        // add new panel to children
        self.get_panel_mut().add_child(new_id);

        ButtonMut::<'b>::new(self.ui, new_id)
    }
}

pub struct PanelStyleRef<'a> {
    store: &'a UiStore,
    node_id: NodeId,
}

impl<'a> PanelStyleRef<'a> {
    pub(crate) fn new(store: &'a UiStore, node_id: NodeId) -> Self {
        Self { store, node_id }
    }

    pub fn background_color(&self) -> Color {
        let mut output = Color::BLACK; // TODO: put into const var!

        self.store.for_each_panel_style(&self.node_id, |style| {
            if let Some(color) = style.background_color {
                output = color;
            }
        });

        output
    }

    pub fn background_alpha(&self) -> f32 {
        let mut output = 1.0; // TODO: put into const var!

        self.store.for_each_panel_style(&self.node_id, |style| {
            if let Some(alpha) = style.background_alpha {
                output = alpha;
            }
        });

        output
    }
}

pub struct PanelStyleMut<'a> {
    ui: &'a mut Ui,
    style_id: StyleId,
}

impl<'a> PanelStyleMut<'a> {
    pub(crate) fn new(ui: &'a mut Ui, style_id: StyleId) -> Self {
        Self { ui, style_id }
    }

    fn get_style_mut(&mut self) -> &mut NodeStyle {
        self.ui.style_mut(&self.style_id).unwrap()
    }

    fn get_panel_style_mut(&mut self) -> &mut PanelStyle {
        if let WidgetStyle::Panel(panel_style) = &mut self.get_style_mut().widget_style {
            panel_style
        } else {
            panic!("StyleId does not reference a PanelStyle");
        }
    }

    // setters

    pub fn set_background_color(&mut self, color: Color) -> &mut Self {
        self.get_panel_style_mut().background_color = Some(color);
        self
    }

    pub fn set_background_alpha(&mut self, alpha: f32) -> &mut Self {
        self.get_panel_style_mut().set_background_alpha(alpha);
        self
    }

    pub fn set_horizontal(&mut self) -> &mut Self {
        self.get_panel_style_mut().layout_type = Some(LayoutType::Row);
        self
    }

    pub fn set_vertical(&mut self) -> &mut Self {
        self.get_panel_style_mut().layout_type = Some(LayoutType::Column);
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
        self.get_panel_style_mut().children_halign = Some(align);
        self
    }

    pub fn set_children_valign(&mut self, align: Alignment) -> &mut Self {
        self.get_panel_style_mut().children_valign = Some(align);
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

    // set_padding_left
    fn set_padding_left_units(&mut self, child_left: SizeUnits) -> &mut Self {
        self.get_panel_style_mut().padding_left = Some(child_left);
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

    // set_padding_right
    fn set_padding_right_units(&mut self, child_right: SizeUnits) -> &mut Self {
        self.get_panel_style_mut().padding_right = Some(child_right);
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

    // set_padding_top
    fn set_padding_top_units(&mut self, child_top: SizeUnits) -> &mut Self {
        self.get_panel_style_mut().padding_top = Some(child_top);
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

    // set_padding_bottom
    fn set_padding_bottom_units(&mut self, child_bottom: SizeUnits) -> &mut Self {
        self.get_panel_style_mut().padding_bottom = Some(child_bottom);
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

    // set_row_between
    fn set_row_between_units(&mut self, row_between: SizeUnits) -> &mut Self {
        self.get_panel_style_mut().row_between = Some(row_between);
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

    // set_col_between
    fn set_col_between_units(&mut self, column_between: SizeUnits) -> &mut Self {
        self.get_panel_style_mut().col_between = Some(column_between);
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
