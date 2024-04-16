use futures::channel::oneshot::{Receiver, Sender};
use logging::info;

pub fn redirect_to_url(url: &str) {
    web_sys::window().unwrap().location().set_href(url).unwrap();
}

static mut EXIT_ACTION_CONTAINER: Option<Sender<String>> = None;
pub struct ExitActionContainer;
impl ExitActionContainer {
    pub(crate) fn init() -> Receiver<String> {
        info!("initializing wasm exit action container");

        let (sender, receiver) = futures::channel::oneshot::channel::<String>();

        unsafe {
            EXIT_ACTION_CONTAINER = Some(sender);
        }

        receiver
    }

    pub fn is_set() -> bool {
        unsafe {
            // None means that the message has been sent over the channel
            EXIT_ACTION_CONTAINER.is_none()
        }
    }

    pub fn set(action: String) {
        info!("setting exit action: {}", action);
        unsafe {
            if EXIT_ACTION_CONTAINER.is_none() {
                panic!("ExitActionContainer not initialized!");
            }
            let sender = EXIT_ACTION_CONTAINER.take().unwrap();
            sender.send(action).unwrap()
        }
    }
}
