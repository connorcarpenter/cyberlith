use bevy_ecs::system::Resource;

/// A resource for storing global UI settings.
#[derive(Clone, Debug, PartialEq, Resource)]
pub struct EguiSettings {
    /// Global scale factor for Egui widgets (`1.0` by default).
    ///
    /// This setting can be used to force the UI to render in physical pixels regardless of DPI as follows:
    /// ```rust
    /// use bevy::{prelude::*, window::PrimaryWindow};
    /// use bevy_egui::EguiSettings;
    ///
    /// fn update_ui_scale_factor(mut egui_settings: ResMut<EguiSettings>, windows: Query<&Window, With<PrimaryWindow>>) {
    ///     if let Ok(window) = windows.get_single() {
    ///         egui_settings.scale_factor = 1.0 / window.scale_factor();
    ///     }
    /// }
    /// ```
    pub scale_factor: f64,
}

impl Default for EguiSettings {
    fn default() -> Self {
        Self { scale_factor: 1.0 }
    }
}
