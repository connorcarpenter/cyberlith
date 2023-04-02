//!
//! Functionality for rendering to the screen or into textures.
//!

mod clear_state;
mod color_target;
mod color_target_multisample;
mod depth_target;
mod depth_target_multisample;
mod multisample;
mod render_target;
mod render_target_ext;

pub use clear_state::*;
pub use color_target::*;
pub use color_target_multisample::*;
pub use depth_target::*;
pub use depth_target_multisample::*;
pub use multisample::*;
pub use render_target::*;
pub use render_target_ext::*;
