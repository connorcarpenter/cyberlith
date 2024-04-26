use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
        compile_error!("this should only be compiled for wasm, it is for web only");
    } else {}
}

use config::{GATEWAY_PORT, PUBLIC_IP_ADDR, PUBLIC_PROTOCOL, SUBDOMAIN_WWW};
use game_app::GameApp;
use kernel::{redirect_to_url, Kernel};
use logging::info;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub async fn main() -> Result<(), JsValue> {
    let mut kernel = Kernel::new();

    info!("Starting GameApp...");
    kernel.load::<GameApp>();

    info!("Running Kernel...");
    let app_result = kernel.run_async().await;

    info!("Kernel ran app, received AppExitAction: {}", app_result);
    match app_result.as_str() {
        "exit" => {
            println!("Exiting...");
        }
        "launcher" => {
            println!("Loading LauncherApp...");
            let url = if SUBDOMAIN_WWW.is_empty() {
                format!("{}://{}:{}/", PUBLIC_PROTOCOL, PUBLIC_IP_ADDR, GATEWAY_PORT)
            } else {
                format!("{}://{}.{}:{}/", PUBLIC_PROTOCOL, SUBDOMAIN_WWW, PUBLIC_IP_ADDR, GATEWAY_PORT)
            };
            redirect_to_url(&url); // root goes to launcher
        }
        _ => {
            panic!("Unknown AppExitAction: {}", app_result);
        }
    }

    Ok(())
}
