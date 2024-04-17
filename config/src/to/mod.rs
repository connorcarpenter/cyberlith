cfg_if! {
    if #[cfg(feature = "client")] {
        mod client;
        pub use self::client::*;
    }
}

cfg_if! {
    if #[cfg(feature = "gateway")] {
        mod gateway;
        pub use self::gateway::*;
    }
}

cfg_if! {
    if #[cfg(feature = "region")] {
        mod region;
        pub use self::region::*;
    }
}

cfg_if! {
    if #[cfg(feature = "session")] {
        mod session;
        pub use self::session::*;
    }
}

cfg_if! {
    if #[cfg(feature = "world")] {
        mod world;
        pub use self::world::*;
    }
}

cfg_if! {
    if #[cfg(feature = "content")] {
        mod content;
        pub use self::content::*;
    }
}

cfg_if! {
    if #[cfg(feature = "asset")] {
        mod asset;
        pub use self::asset::*;
    }
}

cfg_if! {
    if #[cfg(feature = "auth")] {
        mod auth;
        pub use self::auth::*;
    }
}
