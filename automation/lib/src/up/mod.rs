pub mod containers_up;
mod instance_init;
mod instance_start;
mod instance_up;
mod instance_wait;
pub mod server_build;
mod ssh_init;

use std::{collections::HashSet, thread, time::Duration};

use logging::info;

use crate::{
    up::containers_up::containers_up,
    utils::{thread_init_1arg, check_channel, thread_init_compat},
    CliError,
};

pub fn up() -> Result<(), CliError> {

    let image_tag = random::generate_random_string(10);

    let config: HashSet<String> = vec![
        "instance",
        "network",
        "redirector",
        "gateway",
        "content",
        "auth",
        "region",
        "session",
        "world",
        "asset",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();

    let (mut instance_rdy, instance_rcvr) = if config.contains("instance") {
        (false, Some(thread_init_compat(instance_up::instance_up)))
    } else {
        (true, None)
    };

    let (mut redirector_rdy, redirector_rcvr) = if config.contains("redirector") {
        (
            false,
            Some(thread_init_1arg(image_tag.clone(), server_build::server_build_redirector)),
        )
    } else {
        (true, None)
    };

    let (mut gateway_rdy, gateway_rcvr) = if config.contains("gateway") {
        (false, Some(thread_init_1arg(image_tag.clone(), server_build::server_build_gateway)))
    } else {
        (true, None)
    };

    let (mut content_rdy, content_rcvr) = if config.contains("content") {
        (false, Some(thread_init_1arg(image_tag.clone(), server_build::server_build_content)))
    } else {
        (true, None)
    };

    let (mut auth_rdy, auth_rcvr) = if config.contains("auth") {
        (false, Some(thread_init_1arg(image_tag.clone(), server_build::server_build_auth)))
    } else {
        (true, None)
    };

    let (mut region_rdy, region_rcvr) = if config.contains("region") {
        (false, Some(thread_init_1arg(image_tag.clone(), server_build::server_build_region)))
    } else {
        (true, None)
    };

    let (mut session_rdy, session_rcvr) = if config.contains("session") {
        (false, Some(thread_init_1arg(image_tag.clone(), server_build::server_build_session)))
    } else {
        (true, None)
    };

    let (mut world_rdy, world_rcvr) = if config.contains("world") {
        (false, Some(thread_init_1arg(image_tag.clone(), server_build::server_build_world)))
    } else {
        (true, None)
    };

    let (mut asset_rdy, asset_rcvr) = if config.contains("asset") {
        (false, Some(thread_init_1arg(image_tag.clone(), server_build::server_build_asset)))
    } else {
        (true, None)
    };

    loop {
        thread::sleep(Duration::from_secs(5));

        if let Some(instance_rcvr) = &instance_rcvr {
            check_channel(instance_rcvr, &mut instance_rdy)?;
        }
        if let Some(redirector_rcvr) = &redirector_rcvr {
            check_channel(redirector_rcvr, &mut redirector_rdy)?;
        }
        if let Some(gateway_rcvr) = &gateway_rcvr {
            check_channel(gateway_rcvr, &mut gateway_rdy)?;
        }
        if let Some(content_rcvr) = &content_rcvr {
            check_channel(content_rcvr, &mut content_rdy)?;
        }
        if let Some(auth_rcvr) = &auth_rcvr {
            check_channel(auth_rcvr, &mut auth_rdy)?;
        }
        if let Some(region_rcvr) = &region_rcvr {
            check_channel(region_rcvr, &mut region_rdy)?;
        }
        if let Some(session_rcvr) = &session_rcvr {
            check_channel(session_rcvr, &mut session_rdy)?;
        }
        if let Some(world_rcvr) = &world_rcvr {
            check_channel(world_rcvr, &mut world_rdy)?;
        }
        if let Some(asset_rcvr) = &asset_rcvr {
            check_channel(asset_rcvr, &mut asset_rdy)?;
        }

        if instance_rdy
            && redirector_rdy
            && gateway_rdy
            && content_rdy
            && auth_rdy
            && region_rdy
            && session_rdy
            && world_rdy
            && asset_rdy
        {
            break;
        }
    }

    containers_up(config, image_tag)?;

    info!("Done!");
    Ok(())
}
