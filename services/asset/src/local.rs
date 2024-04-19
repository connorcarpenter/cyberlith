use std::process::Command;

use logging::info;

pub(crate) fn setup() {
    info!("Setting up local environment");

    // process assets
    automation_lib::process_assets("target/assets_repo", "local").unwrap();

    // copy ./target/assets_repo/* to ./assets/*
    let source_dir = "./target/assets_repo";
    let destination_dir = "./assets";

    // Delete the destination directory (will create it again later)
    info!("Deleting destination directory: {}", destination_dir);
    let mut delete_command = Command::new("rm")
        .arg("-rf")
        .arg(destination_dir)
        .spawn()
        .unwrap();

    // Wait for the process to finish and capture the exit status
    let status = delete_command.wait().unwrap();
    if status.success() {
        info!("Destination directory deleted successfully.");
    } else {
        panic!("Error: Delete command failed with status: {:?}", status);
    }

    // Create new destination directory
    info!("Creating destination directory: {}", destination_dir);
    Command::new("mkdir")
        .arg("-p")
        .arg(destination_dir)
        .status()
        .expect("Failed to create destination directory");

    // Execute shell command to copy files from source to destination
    info!("Copying files from {:?} to {:?}", source_dir, destination_dir);
    Command::new("cp")
        .arg("-r")
        .arg(format!("{}/.", source_dir))
        .arg(destination_dir)
        .status()
        .expect("Failed to execute copy command");

    // Execute shell command to delete files from source
    info!("Deleting files from {:?}", source_dir);
    Command::new("rm")
        .arg("-rf")
        .arg("./target")
        .status()
        .expect("Failed to execute delete command");

    // finished
    info!("Local environment setup complete!");
}
