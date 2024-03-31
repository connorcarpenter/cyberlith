use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(any(feature = "read_json", feature = "read_bits"))] {
        mod read;
        pub use read::*;
    } else {}
}

cfg_if! {
    if #[cfg(any(feature = "write_json", feature = "write_bits"))] {
        mod write;
        //pub use write::*;
    } else {}
}