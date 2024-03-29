use render_api::base::{CpuMaterial, CpuMesh};
use storage::Storage;
use ui_types::UiConfig;
use ui_state::UiState;

pub struct UiStateData {
    ui_state: UiState,
}

impl Default for UiStateData {
    fn default() -> Self {
        panic!("");
    }
}

impl UiStateData {
    pub fn from_ui_config(ui_config: &UiConfig) -> Self {
        let ui_state = UiState::new(ui_config);

        Self { ui_state }
    }

    pub fn get_ui_state_ref(&self) -> &UiState {
        &self.ui_state
    }

    pub fn get_ui_state_mut(&mut self) -> &mut UiState {
        &mut self.ui_state
    }

    pub fn load_cpu_data(
        &mut self,
        ui_config: &UiConfig,
        meshes: &mut Storage<CpuMesh>,
        materials: &mut Storage<CpuMaterial>,
    ) {
        self.ui_state.set_handles(ui_config, meshes, materials);
    }
}
