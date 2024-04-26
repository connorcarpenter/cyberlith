use cfg_if::cfg_if;
cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        compile_error!("this should not be compiled for web, it is native only");
    } else {}
}

use logging::info;

use game_app::GameApp;
use kernel::Kernel;
use launcher_app::LauncherApp;

fn main() {
    let mut kernel = Kernel::new();
    kernel.load::<LauncherApp>();
    loop {
        let app_result = kernel.run();
        match app_result.as_str() {
            "exit" => {
                info!("Exiting...");
                break;
            }
            "launcher" => {
                info!("Loading LauncherApp...");
                kernel.load::<LauncherApp>();
            }
            "game" => {
                info!("Loading GameApp...");
                kernel.load::<GameApp>();
            }
            _ => {
                panic!("Unknown app: {}", app_result);
            }
        }
    }
    info!("Done.");
}
