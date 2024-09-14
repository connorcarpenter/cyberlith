pub(crate) mod core;
pub(crate) mod renderer;
pub(crate) mod window;

mod exit_system;
mod gpu_material_manager;
mod gpu_mesh_manager;
mod gpu_skin_manager;
mod input;
mod plugin;
mod render;
mod runner;
mod sync;

pub(crate) use gpu_material_manager::*;
pub(crate) use gpu_mesh_manager::*;
pub(crate) use gpu_skin_manager::*;

pub use plugin::RenderGlPlugin;
pub use core::{apply_effect, GpuTexture2D, Context};
pub use renderer::effect::FxaaEffect;
pub use window::{FrameInput, OutgoingEvent};
