pub enum TargetEnv {
    #[allow(dead_code)]
    Local,
    #[allow(dead_code)]
    Prod,
}

cfg_if::cfg_if! {
    if #[cfg(feature = "local")] {
        impl TargetEnv {
            pub fn get() -> Self {
                Self::Local
            }
        }
    } else if #[cfg(feature = "prod")] {
        impl TargetEnv {
            pub fn get() -> Self {
                Self::Prod
            }
        }
    }
}