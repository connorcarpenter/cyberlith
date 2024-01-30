mod instance_start;
mod instance_wait;
mod instance_init;
mod ssh_init;
mod instance_up;

use std::{future::Future, time::Duration};

use async_compat::Compat;
use crossbeam_channel::{bounded, Receiver};
use log::info;
use vultr::VultrError;

pub fn up() {

    let instance_rcvr = thread_init(instance_up::instance_up);

    let mut instance_rdy = false;

    loop {
        std::thread::sleep(Duration::from_secs(5));

        if !instance_rdy {
            match instance_rcvr.try_recv() {
                Ok(_) => instance_rdy = true,
                Err(e) => {
                    match e {
                        crossbeam_channel::TryRecvError::Empty => {},
                        crossbeam_channel::TryRecvError::Disconnected => info!("receiver error: {:?}", e),
                    }
                }
            }
        }

        if instance_rdy {
            break;
        }
    }

    info!("Done!");
}

fn thread_init<F: Future<Output=Result<(), VultrError>> + Sized + Send + 'static>(
    x: fn() -> F
) -> Receiver<Result<(), VultrError>> {
    let (sender, receiver) = bounded(1);

    executor::spawn(Compat::new(async move {
        let result = x().await;
        sender.send(result).expect("failed to send result");
    }))
        .detach();

    receiver
}


