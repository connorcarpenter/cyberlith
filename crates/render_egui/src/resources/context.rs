use bevy_ecs::system::Resource;

#[derive(Clone, Resource, Default)]
pub struct EguiContext(egui::Context);

impl EguiContext {
    #[must_use]
    pub fn get_mut(&mut self) -> &mut egui::Context {
        &mut self.0
    }
}
