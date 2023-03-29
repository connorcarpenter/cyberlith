cfg_if! {
    if #[cfg(feature = "editor")] {
        mod editor;
        pub use editor::{setup, ContextPlugin};
    } else {
        mod editorless;
        pub use editorless::{setup, ContextPlugin};
    }
}
