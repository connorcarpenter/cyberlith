use bevy_ecs::prelude::SystemSet;

#[derive(SystemSet, Clone, Hash, Debug, Eq, PartialEq)]
pub enum EguiStartupSet {
    /// Initializes Egui contexts for available windows.
    InitContexts,
}

#[derive(SystemSet, Clone, Hash, Debug, Eq, PartialEq)]
pub enum EguiSet {
    /// Initializes Egui contexts for newly created windows.
    InitContexts,
    /// Reads Egui inputs (keyboard, mouse, etc) and writes them into the [`EguiInput`] resource.
    ///
    /// To modify the input, you can hook your system like this:
    ///
    /// `system.after(EguiSet::ProcessInput).before(EguiSet::BeginFrame)`.
    ProcessInput,
    /// Begins the `egui` frame.
    BeginFrame,
    /// Processes the [`EguiOutput`] resource.
    ProcessOutput,
}
