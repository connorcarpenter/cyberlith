use bevy_ecs::{change_detection::ResMut, component::Component, system::Query};
use bevy_log::warn;

use morphorm::{Node, Units};

use asset_render::{AssetHandle, AssetManager, IconData};
use render_api::{
    base::{Color, CpuMaterial, CpuMesh},
    components::{RenderLayer, Transform, Viewport},
    resources::RenderFrame,
    shapes::UnitSquare,
};
use storage::{Handle, Storage};

use crate::{
    cache::LayoutCache,
    node::{NodeKind, UiNode, NodeStore},
    node_id::NodeId,
    panel::{Panel, PanelMut, PanelRef},
    widget::Widget,
};

#[derive(Component)]
pub struct Ui {
    globals: Globals,
    pending_mat_handles: Vec<NodeId>,
    next_node_id: NodeId,
    cache: LayoutCache,
    nodes: NodeStore,
    recalc_layout: bool,
    viewport: Viewport,
}

impl Ui {
    const ROOT_NODE_ID: NodeId = NodeId::new(0);

    pub fn new() -> Self {
        let mut me = Self {
            globals: Globals::new(),
            pending_mat_handles: Vec::new(),
            next_node_id: Self::ROOT_NODE_ID,
            cache: LayoutCache::new(),
            nodes: NodeStore::new(),
            recalc_layout: false,
            viewport: Viewport::new_at_origin(0, 0),
        };

        let panel_id = me.create_node(&NodeKind::Panel, Panel::new());
        if panel_id != Self::ROOT_NODE_ID {
            panic!("root panel id is not 0");
        }

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
        for uiid in self.pending_mat_handles.drain(..) {
            let Some(node_ref) = self.nodes.get(&uiid) else {
                continue;
            };
            let color = node_ref.style.background_color;
            let mat_handle = materials.add(color);
            let node = self.nodes.get_mut(&uiid).unwrap();
            node.style.background_color_handle = Some(mat_handle);
        }
    }

    // interface

    pub fn set_text_icon_handle(&mut self, text_handle: &AssetHandle<IconData>) -> &mut Self {
        self.globals.text_icon_handle_opt = Some(text_handle.clone());
        self
    }

    pub fn set_text_color(&mut self, text_color: Color) -> &mut Self {
        self.globals.set_text_color(text_color);
        self
    }

    pub fn root_ref(&self) -> PanelRef {
        PanelRef::new(self, Self::ROOT_NODE_ID)
    }

    pub fn root_mut(&mut self) -> PanelMut {
        self.queue_recalculate_layout();
        PanelMut::new(self, Self::ROOT_NODE_ID)
    }

    pub fn draw(&mut self, render_frame: &mut RenderFrame, render_layer_opt: Option<&RenderLayer>, asset_manager: &AssetManager) {
        let Some(viewport) = render_frame.get_camera_viewport(render_layer_opt) else {
            return;
        };
        self.update_viewport(&viewport);
        self.recalculate_layout_if_needed();

        draw_node(
            render_frame,
            render_layer_opt,
            asset_manager,
            &self.globals,
            &self.cache,
            &self.nodes,
            &Self::ROOT_NODE_ID,
            (0.0, 0.0, 0.0),
        );
    }

    fn update_viewport(&mut self, viewport: &Viewport) {
        let viewport = *viewport;
        if self.viewport == viewport {
            return;
        }
        self.viewport = viewport;
        self.queue_recalculate_layout();
    }

    fn queue_recalculate_layout(&mut self) {
        self.recalc_layout = true;
    }

    fn recalculate_layout_if_needed(&mut self) {
        if self.recalc_layout {
            self.recalc_layout = false;
            self.recalculate_layout();
        }
    }

    fn recalculate_layout(&mut self) {
        //info!("recalculating layout. viewport_width: {:?}, viewport_height: {:?}", self.viewport.width, self.viewport.height);

        let root_panel = self.nodes.get_mut(&Self::ROOT_NODE_ID).unwrap();
        root_panel.style.width = Units::Pixels(self.viewport.width as f32);
        root_panel.style.height = Units::Pixels(self.viewport.height as f32);

        let panels_ref = &self.nodes;
        let cache_mut = &mut self.cache;

        // this calculates all the rects in cache_mut
        Self::ROOT_NODE_ID.layout(cache_mut, panels_ref, panels_ref, &mut ());

        // print_node(&Self::ROOT_PANEL_ID, &self.cache, &self.panels, true, false, "".to_string());

        // now go get all the queued color handles
        // happens each time there's a recalculation of layout ... should actually just happen whenever new elements are added
        self.collect_color_handles();
    }

    fn collect_color_handles(&mut self) {
        let mut pending_mat_handles = Vec::new();
        for (uiid, panel) in self.nodes.iter() {
            if panel.style.background_color_handle.is_none() {
                pending_mat_handles.push(*uiid);
            }
        }
        self.pending_mat_handles = pending_mat_handles;
    }

    fn next_id(&mut self) -> NodeId {
        let output = self.next_node_id;
        self.next_node_id.increment();
        output
    }

    pub(crate) fn create_node<W: Widget>(&mut self, node_kind: &NodeKind, widget: W) -> NodeId {
        let panel = UiNode::new(node_kind, widget);
        let node_id = self.next_id();
        self.nodes.insert(node_id, panel);
        node_id
    }

    pub(crate) fn node_ref(&self, uiid: &NodeId) -> Option<&UiNode> {
        self.nodes.get(&uiid)
    }

    pub(crate) fn node_mut(&mut self, uiid: &NodeId) -> Option<&mut UiNode> {
        self.queue_recalculate_layout();
        self.nodes.get_mut(&uiid)
    }
}

pub(crate) struct Globals {
    box_mesh_handle_opt: Option<Handle<CpuMesh>>,
    text_icon_handle_opt: Option<AssetHandle<IconData>>,
    text_color: Color,
    text_color_handle_opt: Option<Handle<CpuMaterial>>,
}

impl Globals {
    pub(crate) fn new() -> Self {
        Self {
            box_mesh_handle_opt: None,
            text_icon_handle_opt: None,
            text_color: Color::BLACK,
            text_color_handle_opt: None,
        }
    }

    pub fn get_text_icon_handle(&self) -> Option<&AssetHandle<IconData>> {
        self.text_icon_handle_opt.as_ref()
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

pub(crate) fn draw_node(
    render_frame: &mut RenderFrame,
    render_layer_opt: Option<&RenderLayer>,
    asset_manager: &AssetManager,
    globals: &Globals,
    cache: &LayoutCache,
    store: &NodeStore,
    id: &NodeId,
    parent_position: (f32, f32, f32),
) {
    let Some((width, height, child_offset_x, child_offset_y)) = cache.bounds(id) else {
        warn!("no bounds for id: {:?}", id);
        return;
    };

    let Some(node) = store.get(&id) else {
        warn!("no panel for id: {:?}", id);
        return;
    };

    let mut transform = Transform::from_xyz(
        parent_position.0 + child_offset_x,
        parent_position.1 + child_offset_y,
        parent_position.2,
    );
    transform.scale.x = width;
    transform.scale.y = height;

    if node.visible {
        node.widget.draw(
            render_frame,
            render_layer_opt,
            asset_manager,
            globals,
            cache,
            store,
            &node.style,
            &transform,
        );
    }
}
