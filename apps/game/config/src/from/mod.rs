cfg_if! {
    if #[cfg(feature = "local")] {
        mod local;
        pub use self::local::*;
    }
    else if #[cfg(feature = "prod")] {
        mod prod;
        pub use self::prod::*;
    } else {
        compile_error!("Required to specify a feature flag for the target environment, either 'local' or 'prod'");
    }
}
