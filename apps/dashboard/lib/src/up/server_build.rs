use log::info;
use vultr::VultrError;

use crate::utils::run_command;

pub async fn server_build_content() -> Result<(), VultrError> {
    info!("server_build_content done!");
    Ok(())
}

pub async fn server_build_orchestrator() -> Result<(), VultrError> {
    return server_build_common("orchestrator", "orchestrator").await;
}

pub async fn server_build_region() -> Result<(), VultrError> {
    return server_build_common("region", "region_server").await;
}

pub async fn server_build_session() -> Result<(), VultrError> {
    return server_build_common("session", "session_server").await;
}

pub async fn server_build_world() -> Result<(), VultrError> {
    return server_build_common("world", "world_server").await;
}

async fn server_build_common(dir_name: &str, app_name: &str) -> Result<(), VultrError> {
    run_command(app_name, format!("cargo build --release --features local --manifest-path apps/{}/Cargo.toml", dir_name).as_str()).await?;
    run_command(app_name, format!("cp target/release/{} {}", app_name, app_name).as_str()).await?;
    run_command(app_name, format!("docker build --build-arg server_name={} --progress=plain -t {}_image .", app_name, dir_name).as_str()).await?;
    run_command(app_name, format!("rm {}", app_name).as_str()).await?;

    info!("server_build_common({}) done!", dir_name);

    Ok(())
}