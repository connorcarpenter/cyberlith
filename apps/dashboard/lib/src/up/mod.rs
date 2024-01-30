mod instance_start;
mod instance_wait;
mod instance_init;
mod ssh_init;
mod instance_up;
mod server_build;

use std::{future::Future, time::Duration};

use async_compat::Compat;
use crossbeam_channel::{bounded, Receiver, TryRecvError};
use log::{info, warn};
use vultr::VultrError;

pub fn up() {

    let instance_rcvr = thread_init(instance_up::instance_up);
    let mut instance_rdy = false;

    let content_rcvr = thread_init(server_build::server_build_content);
    let mut content_rdy = false;

    let orch_rcvr = thread_init(server_build::server_build_orchestrator);
    let mut orch_rdy = false;

    let region_rcvr = thread_init(server_build::server_build_region);
    let mut region_rdy = false;

    let session_rcvr = thread_init(server_build::server_build_session);
    let mut session_rdy = false;

    let world_rcvr = thread_init(server_build::server_build_world);
    let mut world_rdy = false;

    loop {
        std::thread::sleep(Duration::from_secs(5));

        if !instance_rdy {
            match instance_rcvr.try_recv() {
                Ok(_) => instance_rdy = true,
                Err(TryRecvError::Disconnected) => warn!("instance receiver disconnected!"),
                _ => {},
            }
        }

        if !content_rdy {
            match content_rcvr.try_recv() {
                Ok(_) => content_rdy = true,
                Err(TryRecvError::Disconnected) => warn!("content receiver disconnected!"),
                _ => {},
            }
        }

        if !orch_rdy {
            match orch_rcvr.try_recv() {
                Ok(_) => orch_rdy = true,
                Err(TryRecvError::Disconnected) => warn!("orch receiver disconnected!"),
                _ => {},
            }
        }

        if !region_rdy {
            match region_rcvr.try_recv() {
                Ok(_) => region_rdy = true,
                Err(TryRecvError::Disconnected) => warn!("region receiver disconnected!"),
                _ => {},
            }
        }

        if !session_rdy {
            match session_rcvr.try_recv() {
                Ok(_) => session_rdy = true,
                Err(TryRecvError::Disconnected) => warn!("session receiver disconnected!"),
                _ => {},
            }
        }

        if !world_rdy {
            match world_rcvr.try_recv() {
                Ok(_) => world_rdy = true,
                Err(TryRecvError::Disconnected) => warn!("world receiver disconnected!"),
                _ => {},
            }
        }

        if instance_rdy && content_rdy && orch_rdy && region_rdy && session_rdy && world_rdy {
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


