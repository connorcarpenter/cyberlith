use bevy_ecs::system::Resource;

use egui;

/// Is used for storing Egui context input..
///
/// It gets reset during the [`EguiSet::ProcessInput`] system.
#[derive(Resource, Clone, Debug, Default)]
pub struct EguiInput(pub egui::RawInput);

impl EguiInput {
    pub fn inner(&self) -> &egui::RawInput {
        &self.0
    }

    pub fn inner_mut(&mut self) -> &mut egui::RawInput {
        &mut self.0
    }
}
