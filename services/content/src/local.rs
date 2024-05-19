use automation_lib::{copy_from_repo_to_target_dir, TargetEnv};
use logging::info;

pub(crate) fn setup() {
    info!("Setting up local environment");

    let project_path = "/home/connor/Work/cyberlith";

    // process content
    automation_lib::process_content(project_path, TargetEnv::Local).unwrap();

    // copy ./target/content_repo/* to ./content/*
    let source_dir = format!("{}/target/cyberlith_content", project_path);
    let destination_dir = format!("{}/services/content/files", project_path);
    copy_from_repo_to_target_dir(&source_dir, &destination_dir);

    // finished
    info!("Local environment setup complete!");
}
