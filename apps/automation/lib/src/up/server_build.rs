use log::info;

use crate::{CliError, utils::run_command};

pub async fn server_build_content() -> Result<(), CliError> {

    // build client
    run_command("game_client", "cargo build --release --features gl_renderer,prod --manifest-path apps/client/Cargo.toml --target wasm32-unknown-unknown --lib").await?;
    run_command("game_client", "wasm-bindgen --out-dir target --out-name game_client --target web --no-typescript target/wasm32-unknown-unknown/release/game_client.wasm").await?;

    // move files to main dir
    run_command("game_client", "cp target/game_client.js game_client.js").await?;
    run_command("game_client", "cp target/game_client_bg.wasm game_client_bg.wasm").await?;
    run_command("game_client", "cp apps/client/index.html index.html").await?;

    // build content_server
    run_command("content_server", "cargo build --release --features prod --manifest-path apps/content/Cargo.toml").await?;

    // move file to main dir
    run_command("content_server", "cp target/release/content_server content_server").await?;

    // docker build
    run_command("content_server", "docker build --file content.dockerfile --progress=plain -t content_image .").await?;

    // clean up files
    run_command("game_client", "rm index.html").await?;
    run_command("game_client", "rm game_client.js").await?;
    run_command("game_client", "rm game_client_bg.wasm").await?;
    run_command("content_server", "rm content_server").await?;

    Ok(())
}

pub async fn server_build_orchestrator() -> Result<(), CliError> {
    return server_build_common("orchestrator", "orchestrator").await;
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

async fn server_build_common(dir_name: &str, app_name: &str) -> Result<(), CliError> {
    run_command(app_name, format!("cargo build --release --features prod --manifest-path apps/{}/Cargo.toml", dir_name).as_str()).await?;
    run_command(app_name, format!("cp target/release/{} {}", app_name, app_name).as_str()).await?;
    run_command(app_name, format!("docker build --build-arg server_name={} --progress=plain -t {}_image .", app_name, dir_name).as_str()).await?;
    run_command(app_name, format!("rm {}", app_name).as_str()).await?;

    info!("server_build_common({}) done!", dir_name);

    Ok(())
}