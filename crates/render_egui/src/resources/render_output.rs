use bevy_ecs::system::Resource;

use egui;

/// Is used for storing Egui shapes and textures delta.
#[derive(Clone, Default, Debug, Resource)]
pub struct EguiRenderOutput {
    /// Pairs of rectangles and paint commands.
    ///
    /// The field gets populated during the [`EguiSet::ProcessOutput`] system (belonging to [`CoreSet::PostUpdate`]) and reset during `EguiNode::update`.
    pub paint_jobs: Vec<egui::ClippedPrimitive>,

    /// The change in egui textures since last frame.
    pub textures_delta: egui::TexturesDelta,
}
