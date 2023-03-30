// Editor
cfg_if! {
    if #[cfg(feature = "editor")] {
        mod editor;
        pub use editor::{setup, ContextPlugin};
    } else {
        mod editorless;
        pub use editorless::{setup, ContextPlugin};
    }
}

// Renderer
cfg_if! {
    if #[cfg(all(feature = "glow_renderer", feature = "wgpu_renderer"))]
    {
        // Use both renderer...
        compile_error!("Requires either 'glow_renderer' or 'wgpu_renderer' feature, you must pick one.");
    }
    else if #[cfg(all(not(feature = "glow_renderer"), not(feature = "wgpu_renderer")))]
    {
        // Use no protocols...
        compile_error!("Requires either 'glow_renderer' or 'wgpu_renderer' feature, you must pick one.");
    }
}

cfg_if! {
    if #[cfg(feature = "glow_renderer")] {
        mod glow_renderer;
        pub use glow_renderer::RendererPlugin;
    }
}

cfg_if! {
    if #[cfg(feature = "wgpu_renderer")] {
        mod wgpu_renderer;
        pub use wgpu_renderer::RendererPlugin;
    }
}
