use bevy_app::{Plugin, App as BevyApp};

use crate::app_exit::ExitActionContainer;

pub struct Kernel {
    current_app: Option<Box<dyn KernelAppInner>>
}

impl Kernel {
    pub fn new() -> Self {
        Self {
            current_app: None
        }
    }

    pub fn load<A: KernelApp>(&mut self) {
        self.current_app = Some(A::get_boxed());
    }

    pub fn run(&self) -> String {
        let Some(current_app) = &self.current_app else {
            panic!("Kernel has no app loaded. Call kernel.load::<App>() first.");
        };
        current_app.run_until_quit();

        ExitActionContainer::take()
    }
}

pub trait KernelApp: Plugin {
    fn init() -> Self where Self: Sized;
}

trait KernelAppInner: KernelApp {
    fn get_boxed() -> Box<dyn KernelAppInner> where Self: Sized;
    fn run_until_quit(&self);
}

impl<P: KernelApp> KernelAppInner for P {
    fn get_boxed() -> Box<dyn KernelAppInner> where Self: Sized {
        Box::new(Self::init())
    }

    fn run_until_quit(&self) {
        let mut app = BevyApp::default();
        app.add_plugins(P::init());
        app.run();
    }
}
