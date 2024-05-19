#[macro_use]
extern crate cfg_if;

mod from;

mod to;
pub use to::*;

cfg_if! {
    if #[cfg(any(feature = "client", feature = "gateway", feature = "auth", feature = "world"))] {
        mod target_env;
        pub use target_env::TargetEnv;
    }
}
