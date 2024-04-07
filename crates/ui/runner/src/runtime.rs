use asset_loader::{AssetHandle, IconData, TypedAssetId, UiDependencies, UiTextMeasurer};
use input::CursorIcon;
use math::{Vec2, Vec3};
use render_api::{
    base::{CpuMaterial, CpuMesh},
    components::Viewport,
};
use render_api::components::{CameraBundle, ClearOperation, Transform};
use storage::Storage;
use ui_input::{UiGlobalEvent, UiInputEvent, UiInputState, UiNodeEvent};
use ui_runner_config::{NodeId, UiRuntimeConfig};
use ui_state::UiState;

use crate::handle::UiHandle;

pub struct UiRuntime {
    state: UiState,
    input_state: UiInputState,
    config: UiRuntimeConfig,
    dependencies: UiDependencies,
    camera: CameraBundle
}

impl UiRuntime {

    pub(crate) fn camera_bundle(&self) -> &CameraBundle {
        &self.camera
    }

    pub(crate) fn generate_new_inputs(&mut self, next_inputs: &mut Vec<UiInputEvent>) {
        self.input_state.generate_new_inputs(&self.config, next_inputs);
    }

    pub(crate) fn load_from_bytes(bytes: &[u8]) -> Self {
        let config = UiRuntimeConfig::load_from_bytes(bytes);
        Self::load_from_config(config)
    }

    pub(crate) fn load_from_config(config: UiRuntimeConfig) -> Self {
        let icon_asset_id = config.get_text_icon_asset_id();
        let dependencies = UiDependencies::new(&icon_asset_id);
        let input_state = UiInputState::new();
        let state = UiState::from_ui_config(&config);

        Self {
            state,
            input_state,
            config,
            dependencies,
            camera: Self::default_camera_bundle()
        }
    }

    fn default_camera_bundle() -> CameraBundle {
        let mut default_bundle = CameraBundle::default_3d_perspective(&Viewport::new_at_origin(0, 0));

        default_bundle.camera.clear_operation = ClearOperation::none();
        default_bundle.transform = Transform::from_xyz(
            0.0,
            0.0,
            5.0,
        )
            .looking_at(
                Vec3::ZERO,
                Vec3::NEG_Y,
            );

        default_bundle
    }

    pub fn decompose_to_refs(
        &self,
    ) -> (&UiState, &UiInputState, &UiRuntimeConfig, &UiDependencies, &CameraBundle) {
        (
            &self.state,
            &self.input_state,
            &self.config,
            &self.dependencies,
            &self.camera,
        )
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

    pub fn get_icon_handle(&self) -> AssetHandle<IconData> {
        self.dependencies.get_icon_handle()
    }

    // config

    pub(crate) fn get_node_id_by_id_str(&self, id_str: &str) -> Option<NodeId> {
        self.config.get_node_id_by_id_str(id_str)
    }

    // state

    pub(crate) fn load_cpu_data(
        &mut self,
        meshes: &mut Storage<CpuMesh>,
        materials: &mut Storage<CpuMaterial>,
    ) {
        self.state.load_cpu_data(&self.config, meshes, materials);
    }

    pub(crate) fn update_viewport(&mut self, viewport: &Viewport) {

        // update ui camera
        if viewport != self.camera.camera.viewport.as_ref().unwrap() {
            self.camera.camera.viewport = Some(*viewport);
            self.camera.transform = Transform::from_xyz(
                viewport.width as f32 * 0.5,
                viewport.height as f32 * 0.5,
                1000.0,
            )
                .looking_at(
                    Vec3::new(
                        viewport.width as f32 * 0.5,
                        viewport.height as f32 * 0.5,
                        0.0,
                    ),
                    Vec3::NEG_Y,
                );

            self.state.queue_recalculate_layout();
        }
    }

    pub(crate) fn needs_to_recalculate_layout(&self) -> bool {
        self.state.needs_to_recalculate_layout()
    }

    pub(crate) fn recalculate_layout(&mut self, text_measurer: &UiTextMeasurer) {
        self.state.recalculate_layout(&self.config, text_measurer, self.camera.camera.viewport.as_ref().unwrap());
    }

    // input

    pub(crate) fn receive_input(
        &mut self,
        text_measurer: &UiTextMeasurer,
        mouse_position: Option<Vec2>,
        input_events: Vec<UiInputEvent>,
    ) {
        self.input_state.receive_input(
            &self.config,
            &mut self.state,
            text_measurer,
            mouse_position,
            input_events,
        );
    }

    pub(crate) fn take_global_events(&mut self) -> Vec<UiGlobalEvent> {
        self.input_state.take_global_events()
    }

    pub(crate) fn take_node_events(&mut self) -> Vec<(NodeId, UiNodeEvent)> {
        self.input_state.take_node_events()
    }

    pub(crate) fn get_cursor_icon(&self) -> CursorIcon {
        self.input_state.get_cursor_icon()
    }
}
