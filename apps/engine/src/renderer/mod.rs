// Renderer
cfg_if! {
    if #[cfg(all(feature = "gl_renderer", feature = "wgpu_renderer"))]
    {
        // Use both renderer...
        compile_error!("Requires either 'gl_renderer' or 'wgpu_renderer' feature, you must pick one.");
    }
    else if #[cfg(all(not(feature = "gl_renderer"), not(feature = "wgpu_renderer")))]
    {
        // Use no protocols...
        compile_error!("Requires either 'gl_renderer' or 'wgpu_renderer' feature, you must pick one.");
    }
}

cfg_if! {
    if #[cfg(feature = "gl_renderer")] {
        mod gl_renderer;
        pub use gl_renderer::RendererPlugin;
    }
}

cfg_if! {
    if #[cfg(feature = "wgpu_renderer")] {
        mod wgpu_renderer;
        pub use wgpu_renderer::RendererPlugin;
    }
}
