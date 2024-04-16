use logging::info;

use crate::{utils::run_command, CliError};

pub async fn server_build_content() -> Result<(), CliError> {
    build_web_deploy_files("launcher").await?;
    build_web_deploy_files("game").await?;

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

    // clean up files
    clean_web_deploy_files("launcher").await?;
    clean_web_deploy_files("game").await?;
    run_command("content_server", "rm content_server").await?;

    Ok(())
}

async fn clean_web_deploy_files(name: &str) -> Result<(), CliError> {
    run_command(name, format!("rm {}.html", name).as_str()).await?;

    run_command(name, format!("rm {}.js", name).as_str()).await?;

    run_command(name, format!("rm {}_bg.wasm", name).as_str()).await?;

    Ok(())
}

async fn build_web_deploy_files(name: &str) -> Result<(), CliError> {
    // build game
    run_command(
        name,
        format!(
            "cargo build \
            --release \
            --features gl_renderer,prod \
            --manifest-path deployments/web/{}/Cargo.toml \
            --target wasm32-unknown-unknown \
            --lib",
            name
        )
        .as_str(),
    )
    .await?;

    run_command(
        name,
        format!(
            "wasm-bindgen \
            --out-dir target \
            --out-name {} \
            --target web \
            --no-typescript target/wasm32-unknown-unknown/release/{}.wasm",
            name, name
        )
        .as_str(),
    )
    .await?;

    // move files to main dir
    run_command(name, format!("cp target/{}.js {}.js", name, name).as_str()).await?;

    run_command(
        name,
        format!("cp target/{}_bg.wasm {}_bg.wasm", name, name).as_str(),
    )
    .await?;

    run_command(
        name,
        format!("cp deployments/web/{}/{}.html {}.html", name, name, name).as_str(),
    )
    .await?;
    Ok(())
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

pub async fn server_build_asset() -> Result<(), CliError> {
    return server_build_common("asset", "asset_server").await;
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
