use bevy_ecs::{change_detection::ResMut, component::Component, system::Query};
use bevy_log::warn;

use morphorm::Node;

use render_api::{
    base::{CpuMaterial, CpuMesh},
    components::{RenderLayer, Transform, Viewport},
    resources::RenderFrame,
    shapes::UnitSquare,
};
use storage::{Handle, Storage};

use crate::{
    cache::LayoutCache,
    panel::{Panel, PanelStore},
    style::Style,
    uiid::UiId,
};

#[derive(Component)]
pub struct Ui {
    box_mesh_handle_opt: Option<Handle<CpuMesh>>,
    pending_mat_handles: Vec<UiId>,
    next_uiid: UiId,
    cache: LayoutCache,
    panels: PanelStore,
    recalc_layout: bool,
    viewport: Viewport,
}

impl Ui {
    const ROOT_PANEL_ID: UiId = UiId::new(0);

    pub fn new() -> Self {
        let mut me = Self {
            box_mesh_handle_opt: None,
            pending_mat_handles: Vec::new(),
            next_uiid: Self::ROOT_PANEL_ID,
            cache: LayoutCache::new(),
            panels: PanelStore::new(),
            recalc_layout: false,
            viewport: Viewport::new_at_origin(0, 0),
        };

        let panel_id = me.create_panel();
        if panel_id != Self::ROOT_PANEL_ID {
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
        }
    }

    fn needs_box_handle(&self) -> bool {
        self.box_mesh_handle_opt.is_none()
    }

    fn set_box_handle(&mut self, meshes: &mut Storage<CpuMesh>) {
        let mesh_handle = meshes.add(UnitSquare);
        self.box_mesh_handle_opt = Some(mesh_handle);
    }

    fn needs_color_handles(&self) -> bool {
        !self.pending_mat_handles.is_empty()
    }

    fn set_color_handles(&mut self, materials: &mut Storage<CpuMaterial>) {
        for uiid in self.pending_mat_handles.drain(..) {
            let Some(panel_ref) = self.panels.get(&uiid) else {
                continue;
            };
            let color = panel_ref.style.background_color();
            let mat_handle = materials.add(color);
            let panel = self.panels.get_mut(&uiid).unwrap();
            panel.style.set_background_color_handle(mat_handle);
        }
    }

    pub fn root_ref(&self) -> PanelRef {
        PanelRef::new(self, Self::ROOT_PANEL_ID)
    }

    pub fn root_mut(&mut self) -> PanelMut {
        self.queue_recalculate_layout();
        PanelMut::new(self, Self::ROOT_PANEL_ID)
    }

    pub fn draw(&mut self, render_frame: &mut RenderFrame, render_layer_opt: Option<&RenderLayer>) {
        let Some(viewport) = render_frame.get_camera_viewport(render_layer_opt) else {
            return;
        };
        self.update_viewport(&viewport);
        self.recalculate_layout_if_needed();

        draw_node(
            0.0,
            render_frame,
            render_layer_opt,
            self.box_mesh_handle_opt.as_ref().unwrap(),
            &self.cache,
            &self.panels,
            &Self::ROOT_PANEL_ID,
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

    pub fn recalculate_layout_if_needed(&mut self) {
        if self.recalc_layout {
            self.recalc_layout = false;
            self.recalculate_layout();
        }
    }

    fn recalculate_layout(&mut self) {
        let root_panel = self.panels.get_mut(&Self::ROOT_PANEL_ID).unwrap();
        root_panel.style.set_width_px(self.viewport.width as f32);
        root_panel.style.set_height_px(self.viewport.height as f32);

        let panels_ref = &self.panels;
        let cache_mut = &mut self.cache;

        // this calculates all the rects in cache_mut
        Self::ROOT_PANEL_ID.layout(cache_mut, panels_ref, panels_ref, &mut ());

        // now go get all the queued color handles
        self.collect_color_handles();
    }

    fn collect_color_handles(&mut self) {
        let mut pending_mat_handles = Vec::new();
        for (uiid, panel) in self.panels.iter() {
            if panel.style.background_color_handle().is_none() {
                pending_mat_handles.push(*uiid);
            }
        }
        self.pending_mat_handles = pending_mat_handles;
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
        self.queue_recalculate_layout();
        self.panels.get_mut(&uiid)
    }
}

// use for inspecting children?
pub struct PanelContentsRef<'a> {
    ui: &'a Ui,
    panel_id: UiId,
}

impl<'a> PanelContentsRef<'a> {
    pub(crate) fn new(ui: &'a Ui, panel_id: UiId) -> Self {
        Self { ui, panel_id }
    }
}

// only used for adding children
pub struct PanelContentsMut<'a> {
    ui: &'a mut Ui,
    panel_id: UiId,
}

impl<'a> PanelContentsMut<'a> {
    pub(crate) fn new(ui: &'a mut Ui, panel_id: UiId) -> Self {
        Self { ui, panel_id }
    }

    pub fn add_panel<'b>(self: &'b mut PanelContentsMut<'a>) -> PanelMut<'b> {
        // creates a new panel, returning a context for it
        let new_panel_id = self.ui.create_panel();
        self.ui
            .panel_mut(&self.panel_id)
            .unwrap()
            .children
            .push(new_panel_id);
        PanelMut::<'b>::new(self.ui, new_panel_id)
    }
}

pub struct PanelRef<'a> {
    ui: &'a Ui,
    panel_id: UiId,
}

impl<'a> PanelRef<'a> {
    pub(crate) fn new(ui: &'a Ui, panel_id: UiId) -> Self {
        Self { ui, panel_id }
    }

    pub fn inside(&'a self, inner_fn: impl FnOnce(PanelContentsRef)) -> &Self {
        let context = PanelContentsRef::new(self.ui, self.panel_id);
        inner_fn(context);
        self
    }

    pub fn style_cloned(&self) -> Style {
        self.ui.panel_ref(&self.panel_id).unwrap().style
    }

    pub fn style(&self, inner_fn: impl FnOnce(&Style)) -> &Self {
        if let Some(panel) = self.ui.panel_ref(&self.panel_id) {
            inner_fn(&panel.style);
        }
        self
    }
}

pub struct PanelMut<'a> {
    ui: &'a mut Ui,
    panel_id: UiId,
}

impl<'a> PanelMut<'a> {
    pub(crate) fn new(ui: &'a mut Ui, panel_id: UiId) -> Self {
        Self { ui, panel_id }
    }

    pub fn contents(&'a mut self, inner_fn: impl FnOnce(PanelContentsMut)) -> &mut Self {
        let context = PanelContentsMut::new(self.ui, self.panel_id);
        inner_fn(context);
        self
    }

    pub fn set_visible(&mut self, visible: bool) -> &mut Self {
        if let Some(panel) = self.ui.panel_mut(&self.panel_id) {
            panel.visible = visible;
        }
        self
    }

    pub fn style_cloned(&self) -> Style {
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

fn draw_node(
    depth: f32,
    render_frame: &mut RenderFrame,
    render_layer_opt: Option<&RenderLayer>,
    mesh_handle: &Handle<CpuMesh>,
    cache: &LayoutCache,
    store: &PanelStore,
    id: &UiId,
) {
    let Some((width, height, posx, posy)) = cache.bounds(id) else {
        warn!("no bounds for id: {:?}", id);
        return;
    };

    let Some(panel_ref) = store.get(&id) else {
        warn!("no panel for id: {:?}", id);
        return;
    };
    let Some(mat_handle) = panel_ref.style.background_color_handle() else {
        warn!("no color handle for id: {:?}", id);
        return;
    };

    let mut transform = Transform::from_xyz(posx, posy, depth);
    transform.scale.x = width;
    transform.scale.y = height;
    render_frame.draw_mesh(render_layer_opt, mesh_handle, &mat_handle, &transform);

    for child in panel_ref.children.iter() {
        //info!("drawing child: {:?}", child);
        draw_node(
            depth + 0.1, // TODO: make this configurable
            render_frame,
            render_layer_opt,
            mesh_handle,
            cache,
            store,
            child,
        );
    }
}
