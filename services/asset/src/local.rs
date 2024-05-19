use automation_lib::{copy_from_repo_to_target_dir, OutputType, TargetEnv};

use logging::info;

pub(crate) fn setup() {
    info!("Setting up local environment");

    let project_path = "/home/connor/Work/cyberlith";

    // process assets
    automation_lib::process_assets(project_path, TargetEnv::Local, OutputType::Json).unwrap();

    // copy ./target/assets_repo/* to ./assets/*
    let source_dir = format!("{}/target/cyberlith_assets", project_path);
    let destination_dir = format!("{}/services/asset/assets", project_path);
    copy_from_repo_to_target_dir(&source_dir, &destination_dir);

    // finished
    info!("Local environment setup complete!");
}
