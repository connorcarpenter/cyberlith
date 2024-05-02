use std::sync::{Arc, RwLock};

use bevy_app::{App as BevyApp, Plugin};

use crate::{ExitActionContainer, http::CookieStore};

pub struct Kernel {
    current_app: Option<Box<dyn KernelAppInner>>,
    cookie_store_opt: Option<Arc<RwLock<CookieStore>>>,
}

impl Kernel {

    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {

            pub fn new() -> Self {
                logging::initialize();

                Self {
                    current_app: None,
                    cookie_store_opt: None,
                }
            }

            pub async fn run_async(&self) -> String {

                let recvr = ExitActionContainer::init();

                let Some(current_app) = &self.current_app else {
                    panic!("Kernel has no app loaded. Call kernel.load::<App>() first.");
                };
                if self.cookie_store_opt.is_some() {
                    panic!("Kernel has cookie store set. This is not supported in wasm.");
                }
                current_app.run_until_quit(None);

                recvr.await.unwrap()
            }
        } else {
            pub fn new() -> Self {
                logging::initialize();

                Self {
                    current_app: None,
                    cookie_store_opt: Some(Arc::new(RwLock::new(CookieStore::new()))),
                }
            }

            pub fn run(&self) -> String {
                let Some(current_app) = &self.current_app else {
                    panic!("Kernel has no app loaded. Call kernel.load::<App>() first.");
                };
                let cookie_store_opt = self.cookie_store_opt.as_ref().unwrap().clone();
                current_app.run_until_quit(Some(cookie_store_opt));

                ExitActionContainer::take()
            }

            pub fn head_request(&self, url: &str) -> http_common::Response {

                let cookie_store = self.cookie_store_opt.as_ref().unwrap().clone();
                crate::http::head_request(cookie_store, url)
            }
        }
    }

    pub fn load<A: KernelApp>(&mut self) {
        self.current_app = Some(A::get_boxed());
    }
}

pub trait KernelApp: Plugin {
    fn init(cookie_store_opt: Option<Arc<RwLock<CookieStore>>>) -> Self
    where
        Self: Sized;
}

trait KernelAppInner: KernelApp {
    fn get_boxed() -> Box<dyn KernelAppInner>
    where
        Self: Sized;
    fn run_until_quit(&self, cookie_store_opt: Option<Arc<RwLock<CookieStore>>>);
}

impl<P: KernelApp> KernelAppInner for P {
    fn get_boxed() -> Box<dyn KernelAppInner>
    where
        Self: Sized,
    {
        Box::new(Self::init(None))
    }

    fn run_until_quit(&self, cookie_store_opt: Option<Arc<RwLock<CookieStore>>>) {
        let mut app = BevyApp::default();
        app.add_plugins(P::init(cookie_store_opt));
        app.run();
    }
}
