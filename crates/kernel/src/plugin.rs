use std::sync::{Arc, RwLock};

use bevy_app::{App, Plugin};

use crate::{http::CookieStore, AppExitAction, http::HttpClientPlugin};

pub struct KernelPlugin {
    cookie_store_opt: Option<Arc<RwLock<CookieStore>>>,
}

impl KernelPlugin {
    pub fn new(cookie_store_opt: Option<Arc<RwLock<CookieStore>>>) -> Self {
        Self { cookie_store_opt }
    }
}

impl Plugin for KernelPlugin {
    fn build(&self, app: &mut App) {
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                if self.cookie_store_opt.is_some() {
                    panic!("Kernel has cookie store set. This is not supported in wasm.");
                }
                let http_client_plugin = HttpClientPlugin::default();
                app.add_plugins(http_client_plugin);
            } else {
                let http_client_plugin = HttpClientPlugin::new(self.cookie_store_opt.as_ref().unwrap().clone());
                app.add_plugins(http_client_plugin);
            }
        }

        app.add_event::<AppExitAction>();
    }
}
