use std::process::Command;
use log::info;

pub(crate) fn setup() {
    info!("Setting up local environment");

    // process assets
    automation_lib::process_assets("local").unwrap();

    // copy ./target/repo/* to ./assets/*
    let source_dir = "./target/repo";
    let destination_dir = "./assets";

    // Create and replace the destination directory
    Command::new("rm").arg("-r").arg(destination_dir).status().expect("Failed to delete destination directory");
    Command::new("mkdir").arg("-p").arg(destination_dir).status().expect("Failed to create destination directory");

    // Execute shell command to copy files from source to destination
    Command::new("cp")
        .arg("-r")
        .arg(format!("{}/.", source_dir))
        .arg(destination_dir)
        .status()
        .expect("Failed to execute copy command");

    // Execute shell command to delete files from source
    Command::new("rm")
        .arg("-r")
        .arg("./target")
        .status()
        .expect("Failed to execute delete command");

    // finished
    info!("Local environment setup complete!");
}