use cfg_if::cfg_if;
cfg_if! {
    if #[cfg(all(feature = "local", feature = "prod"))]
    {
        // Use both renderer...
        compile_error!("Requires either 'local' or 'prod' feature, you must pick one.");
    }
    else if #[cfg(all(not(feature = "local"), not(feature = "prod")))]
    {
        // Use no protocols...
        compile_error!("Requires either 'local' or 'prod' feature, you must pick one.");
    }
}

pub use naia_bevy_shared::Protocol;

pub mod behavior;
pub mod channels;
pub mod components;
pub mod constants;
pub mod messages;
pub mod resources;
pub mod types;

mod protocol;
pub use protocol::protocol;
