#[macro_use]
extern crate cfg_if;

mod bevy;
pub use bevy::*;

pub mod core;
pub mod renderer;
pub mod window;
pub mod asset;

mod gui;
pub use gui::*;
