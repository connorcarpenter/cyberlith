use std::collections::HashMap;
use bevy_ecs::change_detection::ResMut;
use bevy_ecs::component::Component;
use morphorm::{LayoutType, PositionType, Units};
use render_api::components::RenderLayer;
use render_api::resources::RenderFrame;

use crate::{Panel, Style};
use crate::uiid::UiId;

#[derive(Component)]
pub struct Ui {
    next_uiid: UiId,
    width_px: f32,
    height_px: f32,
    panels: HashMap<UiId, Panel>,
}

impl Ui {

    const ROOT_PANEL_ID: UiId = UiId::new(0);

    pub fn new() -> Self {

        let mut me = Self {
            next_uiid: Self::ROOT_PANEL_ID,
            width_px: 0.0,
            height_px: 0.0,
            panels: HashMap::new(),
        };

        let panel = Panel::new();
        let panel_id = Self::ROOT_PANEL_ID;
        me.panels.insert(panel_id, panel);
        me.next_uiid.increment();

        me
    }

    pub fn root(&mut self) -> PanelContext {
        PanelContext::new(self, Self::ROOT_PANEL_ID)
    }

    pub fn draw(&self, render_frame: &mut RenderFrame, render_layer_opt: Option<&RenderLayer>) {
        todo!();
    }

    pub(crate) fn next_id(&mut self) -> UiId {
        let output = self.next_uiid;
        self.next_uiid.increment();
        output
    }

    pub(crate) fn create_panel(&mut self) -> UiId {
        let panel = Panel::new();
        let panel_id = self.next_id();
        self.panels.insert(panel_id, panel);
        panel_id
    }

    pub(crate) fn panel_ref(&self, uiid: &UiId) -> Option<&Panel> {
        self.panels.get(&uiid)
    }

    pub(crate) fn panel_mut(&mut self, uiid: &UiId) -> Option<&mut Panel> {
        self.panels.get_mut(&uiid)
    }
}

// used just for adding children
pub struct InsidePanelContext<'a> {
    ui: &'a mut Ui,
    panel_id: UiId,
}

impl<'a> InsidePanelContext<'a> {
    pub(crate) fn new(ui: &'a mut Ui, panel_id: UiId) -> Self {
        Self {
            ui,
            panel_id,
        }
    }

    pub fn panel<'b>(self: &'b mut InsidePanelContext<'a>) -> PanelContext<'b> {
        // creates a new panel, returning a context for it
        let new_panel_id = self.ui.create_panel();
        PanelContext::<'b>::new(self.ui, new_panel_id)
    }
}

pub struct PanelContext<'a> {
    ui: &'a mut Ui,
    panel_id: UiId,
}

impl<'a> PanelContext<'a> {
    pub(crate) fn new(ui: &'a mut Ui, panel_id: UiId) -> Self {
        Self {
            ui,
            panel_id,
        }
    }

    pub fn inside(&'a mut self, inner_fn: impl FnOnce(InsidePanelContext)) -> &mut Self {
        let context = InsidePanelContext::new(self.ui, self.panel_id);
        inner_fn(context);
        self
    }

    pub fn set_visible(&mut self, visible: bool) -> &mut Self {
        if let Some(panel) = self.ui.panel_mut(&self.panel_id) {
            panel.visible = visible;
        }
        self
    }

    pub fn get_style(&self) -> Style {
        self.ui.panel_ref(&self.panel_id).unwrap().style
    }

    pub fn set_style(&mut self, style: Style) -> &mut Self {
        if let Some(panel) = self.ui.panel_mut(&self.panel_id) {
            panel.style = style;
        }
        self
    }

    pub fn style(&mut self, inner_fn: impl FnOnce(&mut Style)) -> &mut Self {
        if let Some(panel) = self.ui.panel_mut(&self.panel_id) {
            inner_fn(&mut panel.style);
        }
        self
    }
}
