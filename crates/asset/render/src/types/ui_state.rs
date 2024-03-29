use render_api::base::{CpuMaterial, CpuMesh};
use storage::Storage;
use ui::{Ui, UiState};

pub struct UiStateData {
    ui_state: UiState,
}

impl Default for UiStateData {
    fn default() -> Self {
        panic!("");
    }
}

impl UiStateData {
    pub fn from_ui(ui: &Ui) -> Self {
        let ui_state = UiState::new(ui);

        Self { ui_state }
    }

    pub(crate) fn get_ui_state_ref(&self) -> &UiState {
        &self.ui_state
    }

    pub(crate) fn get_ui_state_mut(&mut self) -> &mut UiState {
        &mut self.ui_state
    }

    pub(crate) fn load_cpu_data(
        &mut self,
        ui: &Ui,
        meshes: &mut Storage<CpuMesh>,
        materials: &mut Storage<CpuMaterial>,
    ) {
        self.ui_state.set_handles(ui, meshes, materials);
    }
}
