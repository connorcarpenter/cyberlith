use asset_loader::{AssetHandle, IconData, TypedAssetId, UiDependencies, UiTextMeasurer};
use input::CursorIcon;
use math::Vec2;
use render_api::{base::{CpuMaterial, CpuMesh}, components::Viewport};
use storage::Storage;
use ui_input::{UiGlobalEvent, UiInputEvent, UiInputState, UiNodeEvent};
use ui_runtime_config::{UiId, UiRuntimeConfig};
use ui_state::UiState;
use ui_types::UiConfig;

pub struct UiRuntime {
    state: UiState,
    input_state: UiInputState,
    config: UiRuntimeConfig,
    dependencies: UiDependencies,
}

impl UiRuntime {

    pub(crate) fn load_from_bytes(bytes: &[u8]) -> Self {
        let config = ui_serde::bits::read_ui_bits(bytes);
        Self::load_from_config(config)
    }

    pub(crate) fn load_from_config(config: UiConfig) -> Self {

        let icon_asset_id = config.get_text_icon_asset_id();
        let dependencies = UiDependencies::new(icon_asset_id);
        let input_state = UiInputState::new();
        let runtime_config = UiRuntimeConfig::new(config);
        let state = UiState::from_ui_config(&runtime_config);


        Self {
            state,
            input_state,
            config: runtime_config,
            dependencies,
        }
    }

    pub fn decompose_to_refs(&self) -> (&UiState, &UiInputState, &UiRuntimeConfig, &UiDependencies) {
        (&self.state, &self.input_state, &self.config, &self.dependencies)
    }

    // dependencies

    pub(crate) fn load_dependencies(
        &self,
        asset_handle: AssetHandle<Self>,
        dependencies: &mut Vec<(TypedAssetId, TypedAssetId)>
    ) {
        let typed_asset_id = TypedAssetId::Ui(asset_handle.asset_id());
        self.dependencies.load_dependencies(typed_asset_id, dependencies);
    }

    pub(crate) fn finish_dependency(&mut self, dependency_typed_id: TypedAssetId) {
        self.dependencies.finish_dependency(dependency_typed_id);
    }

    pub fn get_icon_handle(&self) -> AssetHandle<IconData> {
        self.dependencies.get_icon_handle()
    }

    // config

    pub(crate) fn get_node_id_by_id_str(&self, id_str: &str) -> Option<UiId> {
        self.config.get_node_id_by_id_str(id_str)
    }

    // state

    pub(crate) fn load_cpu_data(&mut self, meshes: &mut Storage<CpuMesh>, materials: &mut Storage<CpuMaterial>) {
        self.state.load_cpu_data(&self.config, meshes, materials);
    }

    pub(crate) fn update_viewport(&mut self, viewport: &Viewport) {
        self.state.update_viewport(viewport);
    }

    pub(crate) fn needs_to_recalculate_layout(&self) -> bool {
        self.state.needs_to_recalculate_layout()
    }

    pub(crate) fn recalculate_layout(&mut self, text_measurer: &UiTextMeasurer) {
        self.state.recalculate_layout(&self.config, text_measurer);
    }

    // input

    pub(crate) fn receive_input(&mut self, text_measurer: &UiTextMeasurer, mouse_position: Option<Vec2>, input_events: Vec<UiInputEvent>) {
        self.input_state.receive_input(&self.config, &mut self.state, text_measurer, mouse_position, input_events);
    }

    pub(crate) fn take_global_events(&mut self) -> Vec<UiGlobalEvent> {
        self.input_state.take_global_events()
    }

    pub(crate) fn take_node_events(&mut self) -> Vec<(UiId, UiNodeEvent)> {
        self.input_state.take_node_events()
    }

    pub(crate) fn get_cursor_icon(&self) -> CursorIcon {
        self.input_state.get_cursor_icon()
    }
}