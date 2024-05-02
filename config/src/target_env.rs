use crate::{GATEWAY_PORT, PUBLIC_IP_ADDR, PUBLIC_PROTOCOL};

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

impl TargetEnv {
    pub fn is_local() -> bool {
        matches!(Self::get(), Self::Local)
    }

    pub fn is_prod() -> bool {
        matches!(Self::get(), Self::Prod)
    }

    pub fn gateway_url() -> String {
        format!("{}://{}:{}", PUBLIC_PROTOCOL, PUBLIC_IP_ADDR, GATEWAY_PORT)
    }
}