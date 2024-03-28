use input::Modifiers;
use render_api::base::{Color, CpuMaterial};
use storage::Handle;
use ui_layout::{Alignment, MarginUnits, PositionType, SizeUnits, TextMeasurer};

use crate::{button::NodeActiveState, node::UiNode, store::UiStore, style::{NodeStyle, StyleId, WidgetStyle}, NodeId, Panel, PanelMut, PanelStyle, Ui, Navigation, Text};
use crate::input::UiInputEvent;

#[derive(Clone)]
pub struct Textbox {
    pub panel: Panel,
    pub id_str: String,
    pub navigation: Navigation,

    pub text: String,
    pub carat_index: usize,
    pub select_index: Option<usize>,

    hover_color_handle: Option<Handle<CpuMaterial>>,
    active_color_handle: Option<Handle<CpuMaterial>>,
    select_color_handle: Option<Handle<CpuMaterial>>,
}

impl Textbox {
    pub fn new(id_str: &str) -> Self {
        Self {
            panel: Panel::new(),
            id_str: id_str.to_string(),
            navigation: Navigation::new(),
            text: String::new(),
            carat_index: 0,
            select_index: None,
            hover_color_handle: None,
            active_color_handle: None,
            select_color_handle: None,
        }
    }

    // returns whether or not mouse is inside the textbox
    pub fn mouse_is_inside(
        &mut self,
        layout: (f32, f32, f32, f32),
        mouse_x: f32,
        mouse_y: f32,
    ) -> bool {
        let (width, height, posx, posy) = layout;

        mouse_x >= posx && mouse_x <= posx + width + 1.0 && mouse_y >= posy && mouse_y <= posy + height + 1.0
    }

    pub fn needs_color_handle(&self) -> bool {
        self.panel.background_color_handle.is_none() || self.hover_color_handle.is_none() || self.active_color_handle.is_none() || self.select_color_handle.is_none()
    }

    pub fn current_color_handle(&self, state: NodeActiveState) -> Option<Handle<CpuMaterial>> {
        match state {
            NodeActiveState::Normal => self.panel.background_color_handle,
            NodeActiveState::Hover => self.hover_color_handle,
            NodeActiveState::Active => self.active_color_handle,
        }
    }

    pub fn set_hover_color_handle(&mut self, val: Handle<CpuMaterial>) {
        self.hover_color_handle = Some(val);
    }

    pub fn set_active_color_handle(&mut self, val: Handle<CpuMaterial>) {
        self.active_color_handle = Some(val);
    }

    pub fn get_selection_color_handle(&self) -> Option<Handle<CpuMaterial>> {
        self.select_color_handle
    }

    pub fn set_selection_color_handle(&mut self, val: Handle<CpuMaterial>) {
        self.select_color_handle = Some(val);
    }

    pub fn recv_input(&mut self, event: UiInputEvent) {
        match event {
            UiInputEvent::Left(modifiers) => {
                match (modifiers.shift, modifiers.ctrl) {
                    (false, false) => {
                        if self.carat_index > 0 {
                            self.carat_index -= 1;
                        }
                        self.select_index = None;
                    }
                    (true, false) => {
                        if self.carat_index > 0 {
                            if self.select_index.is_none() {
                                // if there is no current selection, set it to the current carat index
                                self.select_index = Some(self.carat_index);
                            }
                            self.carat_index -= 1;
                            if self.carat_index == self.select_index.unwrap() {
                                self.select_index = None;
                            }
                        }
                    }
                    (_, _) => todo!(),
                }
            },
            UiInputEvent::Right(modifiers) => {
                match (modifiers.shift, modifiers.ctrl) {
                    (false, false) => {
                        if self.carat_index < self.text.len() {
                            self.carat_index += 1;
                        }
                        self.select_index = None;
                    }
                    (true, false) => {
                        if self.carat_index < self.text.len() {
                            if self.select_index.is_none() {
                                // if there is no current selection, set it to the current carat index
                                self.select_index = Some(self.carat_index);
                            }
                            self.carat_index += 1;
                            if self.carat_index == self.select_index.unwrap() {
                                self.select_index = None;
                            }
                        }
                    }
                    _ => {}
                }
            },
            UiInputEvent::Text(new_char) => {
                self.text.insert(self.carat_index, new_char);
                self.carat_index += 1;
            },
            UiInputEvent::Backspace(modifiers) => {
                if modifiers.ctrl {
                    todo!()
                } else {
                    if self.carat_index > 0 {
                        self.text.remove(self.carat_index - 1);
                        self.carat_index -= 1;
                    }
                }
            },
            UiInputEvent::Delete(modifiers) => {
                if modifiers.ctrl {
                    todo!()
                } else {
                    if self.carat_index < self.text.len() {
                        self.text.remove(self.carat_index);
                    }
                }
            },
            UiInputEvent::Home(modifiers) => {
                if modifiers.shift {
                    if self.select_index.is_none() {
                        self.select_index = Some(self.carat_index);
                    }
                    self.carat_index = 0;
                    if self.carat_index == self.select_index.unwrap() {
                        self.select_index = None;
                    }
                } else {
                    self.carat_index = 0;
                    self.select_index = None;
                }
            },
            UiInputEvent::End(modifiers) => {
                if modifiers.shift {
                    if self.select_index.is_none() {
                        self.select_index = Some(self.carat_index);
                    }
                    self.carat_index = self.text.len();
                    if self.carat_index == self.select_index.unwrap() {
                        self.select_index = None;
                    }
                } else {
                    self.carat_index = self.text.len();
                    self.select_index = None;
                }
            },
            UiInputEvent::Paste(text) => {
                // TODO: validate pasted text? I did panic at some point here.
                self.text.insert_str(self.carat_index, &text);
                self.carat_index += text.len();
            }
            _ => panic!("Unhandled input event for textbox: {:?}", event),
        }
    }

    pub fn recv_click(&mut self, text_measurer: &dyn TextMeasurer, click_x: f32, position_x: f32, height: f32, modifiers: &Modifiers) {

        if !modifiers.shift {
            self.select_index = None;
        } else {
            if self.select_index.is_none() {
                self.select_index = Some(self.carat_index);
            }
        }

        let click_x = click_x - position_x;

        let mut closest_x: f32 = f32::MAX;
        let mut closest_index: usize = usize::MAX;

        let subimage_indices = Text::get_subimage_indices(&self.text);
        let (x_positions, text_height) = Text::get_raw_text_rects(text_measurer, &subimage_indices);
        let scale = height / text_height;

        for (char_index, x_position) in x_positions.iter().enumerate() {

            let index_x = 8.0 + (x_position * scale);
            let dist = (click_x - index_x).abs();
            if dist < closest_x {
                closest_x = dist;
                closest_index = char_index;
            } else {
                // dist is increasing ... we can break
                self.carat_index = closest_index;
                if let Some(select_index) = self.select_index {
                    if self.carat_index == select_index {
                        self.select_index = None;
                    }
                }
                return;
            }
        }

        self.carat_index = closest_index;
        if let Some(select_index) = self.select_index {
            if self.carat_index == select_index {
                self.select_index = None;
            }
        }
    }
}

#[derive(Clone, Copy)]
pub struct TextboxStyle {
    pub panel: PanelStyle,

    pub hover_color: Option<Color>,
    pub active_color: Option<Color>,
    pub select_color: Option<Color>,
}

impl TextboxStyle {
    pub(crate) fn empty() -> Self {
        Self {
            panel: PanelStyle::empty(),

            hover_color: None,
            active_color: None,
            select_color: None,
        }
    }

    pub fn background_alpha(&self) -> Option<f32> {
        self.panel.background_alpha()
    }

    pub(crate) fn set_background_alpha(&mut self, val: f32) {
        self.panel.set_background_alpha(val);
    }

    pub fn hover_color(&self) -> Option<Color> {
        self.hover_color
    }

    pub(crate) fn set_hover_color(&mut self, val: Color) {
        self.hover_color = Some(val);
    }

    pub fn active_color(&self) -> Option<Color> {
        self.active_color
    }

    pub(crate) fn set_active_color(&mut self, val: Color) {
        self.active_color = Some(val);
    }

    pub fn selection_color(&self) -> Option<Color> {
        self.select_color
    }

    pub(crate) fn set_selection_color(&mut self, val: Color) {
        self.select_color = Some(val);
    }
}

pub struct TextboxMut<'a> {
    ui: &'a mut Ui,
    node_id: NodeId,
}

impl<'a> TextboxMut<'a> {
    pub(crate) fn new(ui: &'a mut Ui, node_id: NodeId) -> Self {
        Self { ui, node_id }
    }

    pub fn set_visible(&mut self, visible: bool) -> &mut Self {
        if let Some(node) = self.ui.node_mut(&self.node_id) {
            node.visible = visible;
        }
        self
    }

    pub fn set_as_first_input(&mut self) -> &mut Self {
        self.ui.set_first_input(self.node_id);
        self
    }

    pub fn add_style(&mut self, style_id: StyleId) -> &mut Self {
        let node = self.ui.node_mut(&self.node_id).unwrap();
        node.style_ids.push(style_id);
        self
    }

    pub fn navigation(&'a mut self, inner_fn: impl FnOnce(&mut TextboxNavigationMut)) -> &mut Self {
        let mut context = TextboxNavigationMut::new(self.ui, self.node_id);
        inner_fn(&mut context);
        self
    }

    pub fn to_panel_mut(&mut self) -> PanelMut {
        PanelMut::new(self.ui, self.node_id)
    }
}

pub struct TextboxNavigationMut<'a> {
    ui: &'a mut Ui,
    node_id: NodeId,
}

impl<'a> TextboxNavigationMut<'a> {
    pub(crate) fn new(ui: &'a mut Ui, node_id: NodeId) -> Self {
        Self { ui, node_id }
    }

    fn get_mut(&mut self) -> &mut UiNode {
        self.ui.node_mut(&self.node_id).unwrap()
    }

    fn get_textbox_mut(&mut self) -> &mut Textbox {
        self.get_mut().widget_textbox_mut().unwrap()
    }

    pub fn left_goes_to(&mut self, name: &str) -> &mut Self {
        self.get_textbox_mut().navigation.left_goes_to = Some(name.to_string());
        self
    }

    pub fn right_goes_to(&mut self, name: &str) -> &mut Self {
        self.get_textbox_mut().navigation.right_goes_to = Some(name.to_string());
        self
    }

    pub fn up_goes_to(&mut self, name: &str) -> &mut Self {
        self.get_textbox_mut().navigation.up_goes_to = Some(name.to_string());
        self
    }

    pub fn down_goes_to(&mut self, name: &str) -> &mut Self {
        self.get_textbox_mut().navigation.down_goes_to = Some(name.to_string());
        self
    }
}

pub struct TextboxStyleRef<'a> {
    store: &'a UiStore,
    node_id: NodeId,
}

impl<'a> TextboxStyleRef<'a> {
    pub(crate) fn new(store: &'a UiStore, node_id: NodeId) -> Self {
        Self { store, node_id }
    }

    pub fn background_color(&self) -> Color {
        let mut output = Color::BLACK; // TODO: put into const var!

        self.store.for_each_textbox_style(&self.node_id, |style| {
            if let Some(color) = style.panel.background_color {
                output = color;
            }
        });

        output
    }

    pub fn background_alpha(&self) -> f32 {
        let mut output = 1.0; // TODO: put into const var!

        self.store.for_each_textbox_style(&self.node_id, |style| {
            if let Some(alpha) = style.panel.background_alpha {
                output = alpha;
            }
        });

        output
    }

    pub fn hover_color(&self) -> Color {
        let mut output = Color::BLACK; // TODO: put into const var!

        self.store.for_each_textbox_style(&self.node_id, |style| {
            if let Some(color) = style.hover_color {
                output = color;
            }
        });

        output
    }

    pub fn active_color(&self) -> Color {
        let mut output = Color::BLACK; // TODO: put into const var!

        self.store.for_each_textbox_style(&self.node_id, |style| {
            if let Some(color) = style.active_color {
                output = color;
            }
        });

        output
    }

    pub fn selection_color(&self) -> Color {
        let mut output = Color::BLACK; // TODO: put into const var!

        self.store.for_each_textbox_style(&self.node_id, |style| {
            if let Some(color) = style.select_color {
                output = color;
            }
        });

        output
    }
}

pub struct TextboxStyleMut<'a> {
    ui: &'a mut Ui,
    style_id: StyleId,
}

impl<'a> TextboxStyleMut<'a> {
    pub(crate) fn new(ui: &'a mut Ui, style_id: StyleId) -> Self {
        Self { ui, style_id }
    }

    fn get_style_mut(&mut self) -> &mut NodeStyle {
        self.ui.style_mut(&self.style_id).unwrap()
    }

    fn get_textbox_style_mut(&mut self) -> &mut TextboxStyle {
        if let WidgetStyle::Textbox(textbox_style) = &mut self.get_style_mut().widget_style {
            textbox_style
        } else {
            panic!("StyleId does not reference a TextboxStyle");
        }
    }

    // setters

    pub fn set_background_color(&mut self, color: Color) -> &mut Self {
        self.get_textbox_style_mut().panel.background_color = Some(color);
        self
    }

    pub fn set_background_alpha(&mut self, alpha: f32) -> &mut Self {
        self.get_textbox_style_mut().set_background_alpha(alpha);
        self
    }

    pub fn set_hover_color(&mut self, color: Color) -> &mut Self {
        self.get_textbox_style_mut().set_hover_color(color);
        self
    }

    pub fn set_active_color(&mut self, color: Color) -> &mut Self {
        self.get_textbox_style_mut().set_active_color(color);
        self
    }

    pub fn set_selection_color(&mut self, color: Color) -> &mut Self {
        self.get_textbox_style_mut().set_selection_color(color);
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
}
