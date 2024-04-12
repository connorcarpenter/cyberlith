use bevy_app::{Plugin};

pub struct App {
    inner: Box<dyn KernelApp>
}

impl App {
    pub fn load<A: KernelApp>() -> Self {
        Self {
            inner: A::get_boxed()
        }
    }

    pub fn run(&self) -> String {
        self.inner.run()
    }
}

pub trait KernelApp: Plugin {
    fn get_boxed() -> Box<Self> where Self: Sized;
    fn run(&self) -> String;
}
