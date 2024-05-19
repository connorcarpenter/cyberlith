use std::{thread, time::Duration};

use cfg_if::cfg_if;
use config::TargetEnv;
cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        compile_error!("this should not be compiled for web, it is native only");
    } else {}
}

use game_app::GameApp;
use kernel::Kernel;
use launcher_app::LauncherApp;
use logging::{info, warn};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum UrlAlias {
    Launcher,
    Game,
}

impl UrlAlias {
    fn to_url(&self) -> String {
        match self {
            UrlAlias::Launcher => format!("{}/", TargetEnv::gateway_url()),
            UrlAlias::Game => format!("{}/game", TargetEnv::gateway_url()),
        }
    }
}

fn main() {
    let mut kernel = Kernel::new();
    let mut next_url = UrlAlias::Launcher;

    loop {
        next_url = handle_http(&kernel, next_url);
        match next_url {
            UrlAlias::Launcher => {
                info!("Loading LauncherApp...");
                kernel.load::<LauncherApp>();
            }
            UrlAlias::Game => {
                info!("Loading GameApp...");
                kernel.load::<GameApp>();
            }
        }

        let app_result = kernel.run();
        match app_result.as_str() {
            "exit" => {
                info!("Exiting...");
                break;
            }
            "launcher" => {
                next_url = UrlAlias::Launcher;
            }
            "game" => {
                next_url = UrlAlias::Game;
            }
            _ => {
                panic!("Unknown app: {}", app_result);
            }
        }
    }
    info!("Done.");
}

fn handle_http(kernel: &Kernel, next_url_alias: UrlAlias) -> UrlAlias {
    let next_url = next_url_alias.to_url();
    loop {
        let response = kernel.head_request(&next_url);
        match response.status {
            200 => {
                info!("Head request to {} succeeded.", next_url);
                return next_url_alias;
            }
            302 => {
                let location = response.get_header_first("location").unwrap();
                info!("Head request to {} redirected to: {}", next_url, location);
                return match location.as_str() {
                    "/game" => UrlAlias::Game,
                    _ => UrlAlias::Launcher,
                };
            }
            error => {
                warn!(
                    "Head request to {} failed with status: {} .. retrying",
                    next_url, error
                );
                thread::sleep(Duration::from_secs(2));
            }
        }
    }
}
