use bevy_ecs::{change_detection::ResMut, component::Component, system::Query};

use asset_id::AssetId;
use render_api::{
    base::{Color, CpuMaterial, CpuMesh},
    components::{Viewport},
    shapes::UnitSquare,
};
use storage::{Handle, Storage};
use ui_layout::{Node, SizeUnits};

use crate::{
    cache::LayoutCache,
    node::{UiNode, UiStore},
    node_id::NodeId,
    panel::{Panel, PanelMut, PanelStyle, PanelStyleMut},
    style::{NodeStyle, StyleId, WidgetStyle},
    text::{TextStyle, TextStyleMut},
    widget::{Widget, WidgetKind},
};

#[derive(Component)]
pub struct Ui {
    pub globals: Globals,
    pending_mat_handles: Vec<NodeId>,
    pub cache: LayoutCache,
    pub store: UiStore,
    recalc_layout: bool,
    last_viewport: Viewport,
}

impl Ui {
    pub const ROOT_NODE_ID: NodeId = NodeId::new(0);
    // pub(crate) const ROOT_STYLE_ID: StyleId = StyleId::new(0);
    pub const BASE_TEXT_STYLE_ID: StyleId = StyleId::new(0);

    pub fn new() -> Self {
        let mut me = Self {
            globals: Globals::new(),
            pending_mat_handles: Vec::new(),
            cache: LayoutCache::new(),
            store: UiStore::new(),
            recalc_layout: false,
            last_viewport: Viewport::new_at_origin(0, 0),
        };

        // Root Node
        let root_panel_id = me.create_node(Widget::Panel(Panel::new()));
        if root_panel_id != Self::ROOT_NODE_ID {
            panic!("root panel id is not 0");
        }

        // Root Style
        // let root_panel_style_id = me.create_style(NodeStyle::empty(WidgetStyle::Panel(PanelStyle::empty())));
        // if root_panel_style_id != Self::ROOT_STYLE_ID {
        //     panic!("root panel style id is {:?}, not 0!", root_panel_style_id);
        // }
        // me.node_mut(&root_panel_id).unwrap().style_ids.push(root_panel_style_id);

        // Base Text Style
        let base_text_style_id =
            me.create_style(NodeStyle::empty(WidgetStyle::Text(TextStyle::empty())));
        if base_text_style_id != Self::BASE_TEXT_STYLE_ID {
            panic!("base text style id is {:?}, not 1!", base_text_style_id);
        }
        me.style_mut(&base_text_style_id).unwrap().width = Some(SizeUnits::Percentage(100.0));
        me.style_mut(&base_text_style_id).unwrap().height = Some(SizeUnits::Percentage(100.0));

        me
    }

    // called as as system

    pub fn update(
        mut meshes: ResMut<Storage<CpuMesh>>,
        mut materials: ResMut<Storage<CpuMaterial>>,
        mut ui_q: Query<&mut Ui>,
    ) {
        for mut ui in ui_q.iter_mut() {
            if ui.needs_box_handle() {
                ui.set_box_handle(&mut meshes);
            }
            if ui.needs_color_handles() {
                ui.set_color_handles(&mut materials);
            }
            if ui.needs_text_color_handle() {
                ui.set_text_color_handle(&mut materials);
            }
        }
    }

    // system methods

    fn needs_box_handle(&self) -> bool {
        self.globals.box_mesh_handle_opt.is_none()
    }

    fn set_box_handle(&mut self, meshes: &mut Storage<CpuMesh>) {
        let mesh_handle = meshes.add(UnitSquare);
        self.globals.box_mesh_handle_opt = Some(mesh_handle);
    }

    fn needs_text_color_handle(&self) -> bool {
        self.globals.text_color_handle_opt.is_none()
    }

    fn set_text_color_handle(&mut self, materials: &mut Storage<CpuMaterial>) {
        let mat_handle = materials.add(self.globals.text_color);
        self.globals.text_color_handle_opt = Some(mat_handle);
    }

    fn needs_color_handles(&self) -> bool {
        !self.pending_mat_handles.is_empty()
    }

    fn set_color_handles(&mut self, materials: &mut Storage<CpuMaterial>) {
        let ids = std::mem::take(&mut self.pending_mat_handles);
        for id in ids {
            let node_ref = self.node_ref(&id).unwrap();
            if node_ref.widget_kind() != WidgetKind::Panel {
                continue;
            }

            let panel_style_ref = self.store.panel_style_ref(&id);
            let color = panel_style_ref.background_color();

            let panel_mut = self.panel_mut(&id).unwrap();

            let mat_handle = materials.add(color);
            panel_mut.background_color_handle = Some(mat_handle);
        }
    }

    // interface

    pub fn get_text_icon_asset_id(&self) -> &AssetId {
        self.globals.text_icon_asset_id_opt.as_ref().unwrap()
    }

    pub fn set_text_icon_asset_id(&mut self, text_icon_asset_id: &AssetId) -> &mut Self {
        self.globals.text_icon_asset_id_opt = Some(text_icon_asset_id.clone());
        self
    }

    pub fn get_text_color(&self) -> Color {
        self.globals.text_color
    }

    pub fn set_text_color(&mut self, text_color: Color) -> &mut Self {
        self.globals.set_text_color(text_color);
        self
    }

    pub fn root_mut(&mut self) -> PanelMut {
        self.queue_recalculate_layout();
        PanelMut::new(self, Self::ROOT_NODE_ID)
    }

    pub fn create_panel_style(&mut self, inner_fn: impl FnOnce(&mut PanelStyleMut)) -> StyleId {
        let new_style_id =
            self.create_style(NodeStyle::empty(WidgetStyle::Panel(PanelStyle::empty())));
        let mut panel_style_mut = PanelStyleMut::new(self, new_style_id);
        inner_fn(&mut panel_style_mut);
        new_style_id
    }

    pub fn create_text_style(&mut self, inner_fn: impl FnOnce(&mut TextStyleMut)) -> StyleId {
        let new_style_id =
            self.create_style(NodeStyle::empty(WidgetStyle::Text(TextStyle::empty())));
        let mut text_style_mut = TextStyleMut::new(self, new_style_id);
        inner_fn(&mut text_style_mut);
        new_style_id
    }

    pub fn update_viewport(&mut self, viewport: &Viewport) {
        let viewport = *viewport;
        if self.last_viewport == viewport {
            return;
        }
        self.last_viewport = viewport;
        self.queue_recalculate_layout();
    }

    fn queue_recalculate_layout(&mut self) {
        self.recalc_layout = true;
    }

    pub fn recalculate_layout_if_needed(&mut self) {
        if self.recalc_layout {
            self.recalc_layout = false;
            self.recalculate_layout();
        }
    }

    fn recalculate_layout(&mut self) {
        //info!("recalculating layout. viewport_width: {:?}, viewport_height: {:?}", self.viewport.width, self.viewport.height);

        let last_viewport_width: f32 = self.last_viewport.width as f32;
        let last_viewport_height: f32 = self.last_viewport.height as f32;

        let root_panel_style_id = *self
            .node_ref(&Self::ROOT_NODE_ID)
            .unwrap()
            .style_ids
            .last()
            .unwrap();
        let root_panel_style = self.style_mut(&root_panel_style_id).unwrap();
        root_panel_style.width = Some(SizeUnits::Pixels(last_viewport_width));
        root_panel_style.height = Some(SizeUnits::Pixels(last_viewport_height));

        let panels_ref = &self.store;
        let cache_mut = &mut self.cache;

        // this calculates all the rects in cache_mut
        Self::ROOT_NODE_ID.layout(cache_mut, panels_ref, panels_ref);

        // print_node(&Self::ROOT_PANEL_ID, &self.cache, &self.panels, true, false, "".to_string());

        // now go get all the queued color handles
        // happens each time there's a recalculation of layout ... should actually just happen whenever new elements are added
        self.collect_color_handles();
    }

    fn collect_color_handles(&mut self) {
        let mut pending_mat_handles = Vec::new();
        for id in self.store.node_ids() {
            let Some(panel_ref) = self.panel_ref(&id) else {
                continue;
            };
            if panel_ref.background_color_handle.is_none() {
                pending_mat_handles.push(id);
            }
        }
        self.pending_mat_handles = pending_mat_handles;
    }

    pub(crate) fn create_node(&mut self, widget: Widget) -> NodeId {
        self.store.insert_node(UiNode::new(widget))
    }

    pub(crate) fn node_ref(&self, id: &NodeId) -> Option<&UiNode> {
        self.store.get_node(&id)
    }

    pub(crate) fn node_mut(&mut self, id: &NodeId) -> Option<&mut UiNode> {
        self.queue_recalculate_layout();
        self.store.get_node_mut(&id)
    }

    pub(crate) fn panel_ref(&self, id: &NodeId) -> Option<&Panel> {
        let node_ref = self.node_ref(id)?;
        let panel_ref = node_ref.widget_panel_ref()?;
        Some(panel_ref)
    }

    pub(crate) fn panel_mut(&mut self, id: &NodeId) -> Option<&mut Panel> {
        let node_mut = self.node_mut(id)?;
        let panel_mut = node_mut.widget_panel_mut()?;
        Some(panel_mut)
    }

    pub(crate) fn style_mut(&mut self, id: &StyleId) -> Option<&mut NodeStyle> {
        self.store.get_style_mut(&id)
    }

    pub(crate) fn create_style(&mut self, style: NodeStyle) -> StyleId {
        self.store.insert_style(style)
    }
}

pub struct Globals {
    box_mesh_handle_opt: Option<Handle<CpuMesh>>,
    text_icon_asset_id_opt: Option<AssetId>,
    text_color: Color,
    text_color_handle_opt: Option<Handle<CpuMaterial>>,
}

impl Globals {
    pub(crate) fn new() -> Self {
        Self {
            box_mesh_handle_opt: None,
            text_icon_asset_id_opt: None,
            text_color: Color::BLACK,
            text_color_handle_opt: None,
        }
    }

    pub fn get_text_icon_handle(&self) -> Option<&AssetId> {
        self.text_icon_asset_id_opt.as_ref()
    }

    pub fn get_text_color_handle(&self) -> Option<&Handle<CpuMaterial>> {
        self.text_color_handle_opt.as_ref()
    }

    pub fn get_box_mesh_handle(&self) -> Option<&Handle<CpuMesh>> {
        self.box_mesh_handle_opt.as_ref()
    }

    pub fn set_text_color(&mut self, color: Color) {
        if color == self.text_color {
            return;
        }
        self.text_color = color;
        self.text_color_handle_opt = None;
    }
}