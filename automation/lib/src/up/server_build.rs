use logging::info;

use crate::{utils::run_command, CliError, TargetEnv};

pub async fn server_build_content() -> Result<(), CliError> {
    let _ = crate::process_content("/home/connor/Work/cyberlith", "", TargetEnv::Prod)?;

    // build content_server
    run_command(
        "content_server",
        "cargo build --release --features prod --manifest-path services/content/Cargo.toml",
    )
    .await?;

    // move content_server executable to main dir
    run_command(
        "content_server",
        "cp target/release/content_server content_server",
    )
    .await?;

    // docker build
    run_command(
        "content_server",
        "docker build --file content.dockerfile --progress=plain -t content_image .",
    )
    .await?;

    // clean up server file
    run_command("content_server", "rm content_server").await?;

    // clean up content repo
    run_command("content_server", "rm -rf target/cyberlith_content").await?;

    Ok(())
}

pub async fn server_build_asset() -> Result<(), CliError> {
    let _ = crate::process_assets("/home/connor/Work/cyberlith", "", TargetEnv::Prod)?;

    // build asset_server
    run_command(
        "asset_server",
        "cargo build --release --features prod --manifest-path services/asset/Cargo.toml",
    )
    .await?;

    // move asset_server executable to main dir
    run_command(
        "asset_server",
        "cp target/release/asset_server asset_server",
    )
    .await?;

    // docker build
    run_command(
        "asset_server",
        "docker build --file asset.dockerfile --progress=plain -t asset_image .",
    )
    .await?;

    // clean up server file
    run_command("asset_server", "rm asset_server").await?;

    // clean up assets repo
    run_command("asset_server", "rm -rf target/cyberlith_assets").await?;

    Ok(())
}

pub async fn server_build_redirector() -> Result<(), CliError> {
    return server_build_common("redirector", "redirector").await;
}

pub async fn server_build_gateway() -> Result<(), CliError> {
    return server_build_common("gateway", "gateway").await;
}

pub async fn server_build_region() -> Result<(), CliError> {
    return server_build_common("region", "region_server").await;
}

pub async fn server_build_session() -> Result<(), CliError> {
    return server_build_common("session", "session_server").await;
}

pub async fn server_build_world() -> Result<(), CliError> {
    return server_build_common("world", "world_server").await;
}

pub async fn server_build_auth() -> Result<(), CliError> {
    return server_build_common("auth", "auth_server").await;
}

async fn server_build_common(dir_name: &str, app_name: &str) -> Result<(), CliError> {
    run_command(
        app_name,
        format!(
            "cargo build --release --features prod --manifest-path services/{}/Cargo.toml",
            dir_name
        )
        .as_str(),
    )
    .await?;
    run_command(
        app_name,
        format!("cp target/release/{} {}", app_name, app_name).as_str(),
    )
    .await?;
    run_command(
        app_name,
        format!(
            "docker build --build-arg server_name={} --progress=plain -t {}_image .",
            app_name, dir_name
        )
        .as_str(),
    )
    .await?;
    run_command(app_name, format!("rm {}", app_name).as_str()).await?;

    info!("server_build_common({}) done!", dir_name);

    Ok(())
}