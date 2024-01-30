use std::time::Duration;
use crossbeam_channel::TryRecvError;
use log::{info, warn};
use vultr::VultrError;

use crate::utils::thread_init;

pub fn containers_up() -> Result<(), VultrError> {
    let rcvr = thread_init(containers_up_impl);

    loop {
        std::thread::sleep(Duration::from_secs(5));

        match rcvr.try_recv() {
            Ok(result) => return result,
            Err(TryRecvError::Disconnected) => warn!("containers receiver disconnected!"),
            _ => {},
        }
    }
}

async fn containers_up_impl() -> Result<(), VultrError> {
    info!("for debugging, pretend that containers are already up! :)");
    return Ok(());
}