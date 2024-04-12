use cfg_if::cfg_if;
cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        compile_error!("this should not be compiled for web, it is native only");
    } else {}
}

use log::info;

use game_app::GameApp;
use launcher_app::LauncherApp;
use kernel::{App};

fn main() {
    let mut loaded_app = Some(App::load::<GameApp>());
    loop {
        if let Some(app) = loaded_app {
            let result = app.run();
            match result.as_str() {
                "exit" => loaded_app = None,
                "launcher" => loaded_app = Some(App::load::<LauncherApp>()),
                "game" => loaded_app = Some(App::load::<GameApp>()),
                _ => panic!("Unknown app: {}", result),
            }
        } else {
            info!("Exiting...");
            break;
        }
    }
    info!("Done.")
}