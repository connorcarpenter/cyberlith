use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        compile_error!("this should not be compiled for web, it is native only");
    } else {}
}

use game_app::GameApp;
use kernel::Kernel;
use logging::info;

fn main() {
    let mut kernel = Kernel::new();

    kernel.load::<GameApp>();

    let app_result = kernel.run();
    match app_result.as_str() {
        "exit" => {
            info!("Exiting...");
        }
        _ => {
            panic!("Unknown app result: {}", app_result);
        }
    }
    info!("Done.");
}
