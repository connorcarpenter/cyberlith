mod instance_start;
mod instance_wait;
mod instance_init;
mod ssh_init;
mod instance_up;
pub mod server_build;
pub mod containers_up;

use std::time::Duration;

use log::info;

use crate::{utils::{check_channel, thread_init, thread_init_compat}, up::containers_up::containers_up, CliError};

pub fn up() -> Result<(), CliError> {

    let instance_rcvr = thread_init_compat(instance_up::instance_up);
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

        check_channel(&instance_rcvr, &mut instance_rdy)?;
        check_channel(&content_rcvr, &mut content_rdy)?;
        check_channel(&orch_rcvr, &mut orch_rdy)?;
        check_channel(&region_rcvr, &mut region_rdy)?;
        check_channel(&session_rcvr, &mut session_rdy)?;
        check_channel(&world_rcvr, &mut world_rdy)?;

        if instance_rdy && content_rdy && orch_rdy && region_rdy && session_rdy && world_rdy {
            break;
        }
    }

    containers_up()?;

    info!("Done!");
    Ok(())
}



