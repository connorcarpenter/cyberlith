
cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        mod wasm;
        pub use self::wasm::*;
    }
    else {
        mod native;
        pub use self::native::*;
    }
}

use crate::GamepadId;

pub fn convert_gamepad_id(gamepad_id: gilrs::GamepadId) -> GamepadId {
    GamepadId::new(gamepad_id.into())
}