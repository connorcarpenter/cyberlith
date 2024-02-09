use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "bits")] {
        pub mod bits;
    } else {}
}

cfg_if! {
    if #[cfg(feature = "json")] {
        pub mod json;
    } else {}
}