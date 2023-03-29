//!
//! Graphical User Interface support.
//!

cfg_if! {
    if #[cfg(feature = "editor")] {
        mod egui_gui;
        pub use egui_gui::*;
    }
}
