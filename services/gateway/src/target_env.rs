pub enum TargetEnv {
    Local,
    Prod,
}

cfg_if::cfg_if! {
    if #[cfg(feature = "local")] {
        pub(crate) fn get_env() -> TargetEnv {
            TargetEnv::Local
        }
    } else if #[cfg(feature = "prod")] {
        pub(crate) fn get_env() -> TargetEnv {
            TargetEnv::Prod
        }
    }
}