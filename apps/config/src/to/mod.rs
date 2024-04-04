cfg_if! {
    if #[cfg(feature = "client")] {
        mod client;
        pub use self::client::*;
    }
    else if #[cfg(feature = "gateway")] {
        mod gateway;
        pub use self::gateway::*;
    }
    else if #[cfg(feature = "region")] {
        mod region;
        pub use self::region::*;
    }
    else if #[cfg(feature = "session")] {
        mod session;
        pub use self::session::*;
    }
    else if #[cfg(feature = "world")] {
        mod world;
        pub use self::world::*;
    }
    else if #[cfg(feature = "content")] {
        mod content;
        pub use self::content::*;
    }
    else if #[cfg(feature = "asset")] {
        mod asset;
        pub use self::asset::*;
    }
    else if #[cfg(feature = "auth")] {
        mod auth;
        pub use self::auth::*;
    }
    else {
        compile_error!("Required to specify a feature flag for the target environment, either 'local' or 'prod'");
    }
}
