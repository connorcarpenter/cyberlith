use bevy_ecs::system::Resource;

use egui;

/// Is used for storing Egui output.
#[derive(Resource, Clone, Default)]
pub struct EguiOutput {
    /// The field gets updated during the [`EguiSet::ProcessOutput`] system (belonging to [`CoreSet::PostUpdate`]).
    pub platform_output: egui::PlatformOutput,
}
