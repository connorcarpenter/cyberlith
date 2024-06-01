use asset_loader::{TypedAssetId, UiDependencies, UiTextMeasurer};
use logging::warn;
use math::Vec3;
use render_api::{
    base::CpuMaterial,
    components::{CameraBundle, ClearOperation, Projection, Transform, Viewport},
};
use storage::Storage;
use ui_runner_config::{NodeId, SerdeErr, UiRuntimeConfig, BaseNodeStyle, StyleId};
use ui_state::UiState;

use crate::{config::{ValidationType, UiNode}, handle::UiHandle};

pub struct UiRuntime {
    state: UiState,
    config: UiRuntimeConfig,
    dependencies: UiDependencies,
    camera: CameraBundle,
}

impl UiRuntime {
    pub(crate) fn load_from_bytes(bytes: &[u8]) -> Result<Self, SerdeErr> {
        let config = UiRuntimeConfig::load_from_bytes(bytes)?;
        Ok(Self::load_from_config(config))
    }

    pub(crate) fn load_from_config(config: UiRuntimeConfig) -> Self {
        let dependencies = UiDependencies::new();
        let state = UiState::from_ui_config(&config);

        Self {
            state,
            config,
            dependencies,
            camera: Self::default_camera_bundle(),
        }
    }

    fn default_camera_bundle() -> CameraBundle {
        let mut default_bundle =
            CameraBundle::default_3d_perspective(&Viewport::new_at_origin(0, 0));

        default_bundle.camera.clear_operation = ClearOperation {
            red: None,
            green: None,
            blue: None,
            alpha: None,
            depth: Some(1.0),
        };
        default_bundle.transform =
            Transform::from_xyz(0.0, 0.0, 1000.0).looking_at(Vec3::ZERO, Vec3::NEG_Y);

        default_bundle
    }

    pub(crate) fn update_state(&mut self, delta_ms: f32) {
        self.state.update(delta_ms);
    }

    // returns true if viewport was updated
    pub(crate) fn update_viewport(&mut self, viewport: &Viewport) -> bool {
        // update ui camera
        if viewport != self.camera.camera.viewport.as_ref().unwrap() {
            // info!("ui viewport updated: {:?}", viewport);

            self.camera.camera.viewport = Some(*viewport);

            let Projection::Perspective(perspective) = &self.camera.projection else {
                panic!("expected perspective projection");
            };
            let distance = ((viewport.height as f32) / 2.0) / f32::tan(perspective.fov / 2.0);
            //let distance = 1000.0;
            let x = viewport.width as f32 * 0.5;
            let y = viewport.height as f32 * 0.5;
            self.camera.transform.translation.x = x;
            self.camera.transform.translation.y = y;
            self.camera.transform.translation.z = distance;
            self.camera
                .transform
                .look_at(Vec3::new(x, y, 0.0), Vec3::NEG_Y);

            return true;
        }

        return false;
    }

    pub fn inner_refs(&self) -> (&UiState, &UiRuntimeConfig, &UiDependencies, &CameraBundle) {
        (&self.state, &self.config, &self.dependencies, &self.camera)
    }

    // dependencies

    pub(crate) fn load_dependencies(
        &self,
        ui_handle: UiHandle,
        dependencies: &mut Vec<(TypedAssetId, TypedAssetId)>,
    ) {
        let typed_asset_id = TypedAssetId::Ui(ui_handle.asset_id());
        self.dependencies
            .load_dependencies(typed_asset_id, dependencies);
    }

    pub(crate) fn finish_dependency(&mut self, dependency_typed_id: TypedAssetId) {
        self.dependencies.finish_dependency(dependency_typed_id);
    }

    // config

    pub(crate) fn get_node_id_by_id_str(&self, id_str: &str) -> Option<NodeId> {
        self.config.get_node_id_by_id_str(id_str)
    }

    // state

    pub(crate) fn load_cpu_data(&mut self, materials: &mut Storage<CpuMaterial>) {
        self.state.load_cpu_data(&self.config, materials);
    }

    pub(crate) fn recalculate_layout(
        &mut self,
        text_measurer: &UiTextMeasurer,
        z: f32,
    ) -> Vec<(UiHandle, Viewport, f32)> {
        self.state
            .recalculate_layout(
                &self.config,
                text_measurer,
                self.camera.camera.viewport.as_ref().unwrap(),
                z,
            )
            .iter()
            .map(|(asset_id, viewport, z)| (UiHandle::new(*asset_id), *viewport, *z))
            .collect()
    }

    pub fn add_style(&mut self, base_node_style: BaseNodeStyle) -> StyleId {
        self.state.style_state_init(&base_node_style.widget_style.kind());
        self.config.add_style(base_node_style)
    }

    pub fn add_node(&mut self, node: UiNode) -> NodeId {
        self.state.node_state_init(&node);
        self.config.add_node(node)
    }

    pub fn remove_nodes_after(&mut self, node_id: &NodeId) {
        self.state.remove_nodes_after(node_id);
        self.config.remove_nodes_after(node_id);
    }

    pub(crate) fn get_textbox_validator(&self, id_str: &str) -> Option<ValidationType> {
        // get node_id from id_str
        let node_id = self.get_node_id_by_id_str(id_str)?;

        // get result from config
        let textbox = self.config.textbox_ref(&node_id)?;
        textbox.validation.clone()
    }

    pub(crate) fn get_textbox_text(&self, id_str: &str) -> Option<String> {
        // get node_id from id_str
        let node_id = self.get_node_id_by_id_str(id_str)?;

        // get result from state
        self.state.get_textbox_text(&node_id)
    }

    pub(crate) fn set_textbox_text(&mut self, id_str: &str, val: &str) {
        // get node_id from id_str
        let Some(node_id) = self.get_node_id_by_id_str(id_str) else {
            warn!("set_textbox_text: node_id not found for id_str: {}", id_str);
            return;
        };

        // set text
        self.state.set_textbox_text(&node_id, val);
    }

    pub(crate) fn get_text_by_id_str(&self, id_str: &str) -> Option<String> {
        // get node_id from id_str
        let node_id = self.get_node_id_by_id_str(id_str)?;

        // get result from state
        self.state.get_text(&node_id)
    }

    pub fn set_text(&mut self, node_id: &NodeId, val: &str) {
        self.state.set_text(node_id, val);
    }

    pub(crate) fn set_text_from_id_str(&mut self, id_str: &str, val: &str) {
        // get node_id from id_str
        let Some(node_id) = self.get_node_id_by_id_str(id_str) else {
            warn!("set_text: node_id not found for id_str: {}", id_str);
            return;
        };

        // set text
        self.state.set_text(&node_id, val);
    }

    pub(crate) fn textbox_receive_hover(
        &mut self,
        node_id: &NodeId,
        bounds: (f32, f32, f32, f32),
        mouse_x: f32,
        mouse_y: f32,
    ) -> bool {
        let textbox_config = self.config.textbox_ref(node_id).unwrap();
        self.state
            .store
            .textbox_mut(node_id)
            .unwrap()
            .receive_hover(textbox_config, bounds, mouse_x, mouse_y)
    }

    pub(crate) fn set_textbox_password_eye_visible(&mut self, id_str: &str, val: bool) {
        // get node_id from id_str
        let Some(node_id) = self.get_node_id_by_id_str(id_str) else {
            warn!(
                "set_textbox_password_eye_visible: node_id not found for id_str: {}",
                id_str
            );
            return;
        };

        // set text
        self.state.set_textbox_password_eye_visible(&node_id, val);
    }

    pub fn set_node_visible(&mut self, id_str: &str, val: bool) {
        // get node_id from id_str
        let Some(node_id) = self.get_node_id_by_id_str(id_str) else {
            warn!("set_node_visible: node_id not found for id_str: {}", id_str);
            return;
        };

        // set text
        self.state.set_node_visible(&node_id, val);
    }

    pub fn get_ui_container_contents(&self, id: &NodeId) -> Option<UiHandle> {
        // get ui handle
        self.state.get_ui_container_asset_id_opt(id).map(UiHandle::new)
    }

    pub fn get_ui_container_contents_by_id_str(&self, id_str: &str) -> Option<UiHandle> {
        // get node_id from id_str
        let Some(node_id) = self.get_node_id_by_id_str(id_str) else {
            warn!(
                "get_ui_container_contents: node_id not found for id_str: {}",
                id_str
            );
            return None;
        };

        // get ui handle
        self.get_ui_container_contents(&node_id)
    }

    pub fn set_ui_container_contents(&mut self, id_str: &str, child_handle: &UiHandle) {
        // get node_id from id_str
        let Some(node_id) = self.get_node_id_by_id_str(id_str) else {
            warn!(
                "set_ui_container_contents: node_id not found for id_str: {}",
                id_str
            );
            return;
        };

        // set ui handle
        self.state
            .set_ui_container_asset_id(&node_id, &child_handle.asset_id());
    }

    pub fn clear_ui_container_contents(&mut self, id_str: &str) {
        // get node_id from id_str
        let Some(node_id) = self.get_node_id_by_id_str(id_str) else {
            warn!(
                "clear_ui_container_contents: node_id not found for id_str: {}",
                id_str
            );
            return;
        };

        // set ui handle
        self.state.clear_ui_container(&node_id);
    }

    pub fn ui_state_ref(&self) -> &UiState {
        &self.state
    }

    pub fn ui_state_mut(&mut self) -> &mut UiState {
        &mut self.state
    }

    pub fn ui_config_ref(&self) -> &UiRuntimeConfig {
        &self.config
    }

    pub fn ui_config_mut(&mut self) -> &mut UiRuntimeConfig {
        &mut self.config
    }
}
