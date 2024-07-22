
mod systems;

mod app;
pub use app::GameApp;

cfg_if::cfg_if! {
    if #[cfg(all(feature = "odst", feature = "no_odst"))] {
        compile_error!("Requires either 'odst' or 'no_odst' feature, you must pick one.");
    } else if #[cfg(all(not(feature = "odst"), not(feature = "no_odst")))] {
        compile_error!("Requires either 'odst' or 'no_odst' feature, you must pick one.");
    }
}