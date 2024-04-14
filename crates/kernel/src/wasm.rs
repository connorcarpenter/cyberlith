//use std::sync::{Arc, RwLock};

use futures::channel::oneshot::{Receiver, Sender};
use logging::info;

pub fn redirect_to_url(url: &str) {
    todo!("redirecting to url: {}", url);
}

static mut EXIT_ACTION_CONTAINER: Option<Sender<String>> = None;
pub(crate) struct ExitActionContainer;
impl ExitActionContainer {

    pub(crate) fn init() -> Receiver<String> {
        info!("initializing wasm exit action container");

        let (sender, receiver) = futures::channel::oneshot::channel::<String>();

        unsafe {
            EXIT_ACTION_CONTAINER = Some(sender);
        }

        receiver
    }

    pub(crate) fn is_set() -> bool {
        unsafe {
            // None means that the message has been sent over the channel
            EXIT_ACTION_CONTAINER.is_none()
        }
    }

    pub(crate) fn set(action: String) {
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