use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
        compile_error!("this should only be compiled for wasm, it is for web only");
    } else {}
}

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
            let domain_str = "http://127.0.0.1:14196"; // TODO: get this from config
            redirect_to_url(&format!("{}/", domain_str)); // root goes to launcher
        }
        _ => {
            panic!("Unknown AppExitAction: {}", app_result);
        }
    }

    Ok(())
}
