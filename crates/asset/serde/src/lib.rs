mod error;
pub use error::AssetIoError;

mod traits;

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(any(feature = "write_bits", feature = "read_bits"))] {
        pub mod bits;
    } else {}
}

cfg_if! {
    if #[cfg(any(feature = "write_json", feature = "read_json"))] {
        pub mod json;
    } else {}
}
