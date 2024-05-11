use logging::info;

use crate::{utils::run_command, CliError, types::{TargetEnv, OutputType}};

pub async fn server_build_content(image_tag: String) -> Result<(), CliError> {
    let _ = crate::process_content("/home/connor/Work/cyberlith", TargetEnv::Prod)?;

    // build content_server
    run_command(
        "content_server",
        "cargo build --release --target x86_64-unknown-linux-gnu --features prod --manifest-path services/content/Cargo.toml",
    )
    .await?;

    // move content_server executable to main dir
    run_command(
        "content_server",
        "cp target/x86_64-unknown-linux-gnu/release/content_server content_server",
    )
    .await?;

    // docker build
    run_command(
        "content_server",
        format!("docker build --file content.dockerfile --progress=plain -t content_image:{} .", image_tag).as_str(),
    )
    .await?;

    // clean up server file
    run_command("content_server", "rm content_server").await?;

    // clean up content repo
    run_command("content_server", "rm -rf target/cyberlith_content").await?;

    Ok(())
}

pub async fn server_build_asset(image_tag: String) -> Result<(), CliError> {
    let _ = crate::process_assets("/home/connor/Work/cyberlith", TargetEnv::Prod, OutputType::Json)?;

    // build asset_server
    run_command(
        "asset_server",
        "cargo build --release --target x86_64-unknown-linux-gnu --features prod --manifest-path services/asset/Cargo.toml",
    )
    .await?;

    // move asset_server executable to main dir
    run_command(
        "asset_server",
        "cp target/x86_64-unknown-linux-gnu/release/asset_server asset_server",
    )
    .await?;

    // docker build
    run_command(
        "asset_server",
        format!("docker build --file asset.dockerfile --progress=plain -t asset_image:{} .", image_tag).as_str(),
    )
    .await?;

    // clean up server file
    run_command("asset_server", "rm asset_server").await?;

    // clean up assets repo
    run_command("asset_server", "rm -rf target/cyberlith_assets").await?;

    Ok(())
}

pub async fn server_build_redirector(image_tag: String) -> Result<(), CliError> {
    return server_build_common("redirector", "redirector", &image_tag).await;
}

pub async fn server_build_gateway(image_tag: String) -> Result<(), CliError> {
    return server_build_common("gateway", "gateway", &image_tag).await;
}

pub async fn server_build_region(image_tag: String) -> Result<(), CliError> {
    return server_build_common("region", "region_server", &image_tag).await;
}

pub async fn server_build_session(image_tag: String) -> Result<(), CliError> {
    return server_build_common("session", "session_server", &image_tag).await;
}

pub async fn server_build_world(image_tag: String) -> Result<(), CliError> {
    return server_build_common("world", "world_server", &image_tag).await;
}

pub async fn server_build_auth(image_tag: String) -> Result<(), CliError> {
    return server_build_common("auth", "auth_server", &image_tag).await;
}

async fn server_build_common(dir_name: &str, app_name: &str, image_tag: &str) -> Result<(), CliError> {
    run_command(
        app_name,
        format!(
            "cargo build --release --target x86_64-unknown-linux-gnu --features prod --manifest-path services/{}/Cargo.toml",
            dir_name
        )
        .as_str(),
    )
    .await?;
    run_command(
        app_name,
        format!("cp target/x86_64-unknown-linux-gnu/release/{} {}", app_name, app_name).as_str(),
    )
    .await?;
    run_command(
        app_name,
        format!(
            "docker build --build-arg server_name={} --progress=plain -t {}_image:{} .",
            app_name, dir_name, image_tag,
        )
        .as_str(),
    )
    .await?;
    run_command(app_name, format!("rm {}", app_name).as_str()).await?;

    info!("server_build_common({}) done!", dir_name);

    Ok(())
}
