use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
        compile_error!("this should only be compiled for wasm, it is for web only");
    } else {}
}

use config::{GATEWAY_PORT, PUBLIC_IP_ADDR, PUBLIC_PROTOCOL};
use kernel::{redirect_to_url, Kernel};
use launcher_app::LauncherApp;
use logging::info;

use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub async fn main() -> Result<(), JsValue> {
    let mut kernel = Kernel::new();

    info!("Starting LauncherApp...");
    kernel.load::<LauncherApp>();

    info!("Running Kernel...");
    let app_result = kernel.run_async().await;

    info!("Kernel ran app, received AppExitAction: {}", app_result);
    match app_result.as_str() {
        "exit" => {
            println!("Exiting...");
        }
        "game" => {
            println!("Loading GameApp...");
            let domain_str = format!("{}://{}:{}", PUBLIC_PROTOCOL, PUBLIC_IP_ADDR, GATEWAY_PORT);
            let url = format!("{}/game", domain_str);
            redirect_to_url(&url);
        }
        _ => {
            panic!("Unknown AppExitAction: {}", app_result);
        }
    }

    Ok(())
}
