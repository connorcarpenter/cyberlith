use log::info;
use vultr::VultrError;

pub async fn server_build_content() -> Result<(), VultrError> {
    info!("server_build_content done!");
    Ok(())
}

pub async fn server_build_orchestrator() -> Result<(), VultrError> {
    return server_build_common("orch").await;
}

pub async fn server_build_region() -> Result<(), VultrError> {
    return server_build_common("region").await;
}

pub async fn server_build_session() -> Result<(), VultrError> {
    return server_build_common("session").await;
}

pub async fn server_build_world() -> Result<(), VultrError> {
    return server_build_common("world").await;
}

async fn server_build_common(name: &str) -> Result<(), VultrError> {
    info!("server_build_common({}) done!", name);
    Ok(())
}