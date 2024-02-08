use std::{fs, path::Path};

use git2::{Cred, Repository, Tree};
use log::info;

use crate::CliError;

pub(crate) fn process_assets() -> Result<(), CliError> {
    info!("Processing assets: 'json'");

    // pull all assets into memory, from "main" branch
    create_project();

    // switch to "json" branch

    // delete all files, push

    // create json file for each previous file

    // push

    Ok(())
}

fn create_project() {
    // Create Working directory if it doesn't already exist
    let root_dir = "target/repo";
    let path = Path::new(&root_dir);
    let repo_url = include_str!("../../../../../.secrets/assets_repo_url");
    let access_token = include_str!("../../../../../.secrets/github_token");

    let mut fetch_options = git2::FetchOptions::new();
    fetch_options.remote_callbacks(get_remote_callbacks(access_token));

    let repo = if !path.exists() {
        // Create new directory
        fs::create_dir_all(path).unwrap();

        // Put fetch options into builder
        let mut builder = git2::build::RepoBuilder::new();
        builder.fetch_options(fetch_options);

        // Clone repo
        let repo = builder.clone(repo_url, path).unwrap();

        info!("initialized repo at: `{}`", root_dir);

        repo
    } else {
        info!("repo exists at: `{}`", root_dir);

        // Open repo
        let repo = Repository::open(path).unwrap();

        {
            let mut remote = repo.find_remote("origin").unwrap();
            remote
                .fetch(&["main"], Some(&mut fetch_options), None)
                .unwrap();

            let reference = repo.find_reference("FETCH_HEAD").unwrap();
            let target = reference.peel_to_commit().unwrap();

            // Set up a CheckoutBuilder to force the working directory to match the target
            let mut checkout_builder = git2::build::CheckoutBuilder::new();
            checkout_builder.force();

            // Reset local changes
            repo.reset(
                target.as_object(),
                git2::ResetType::Hard,
                Some(&mut checkout_builder),
            )
                .unwrap();

            info!("pulled repo with new changes");
        }

        repo
    };

    {
        let head = repo.head().unwrap();
        let tree = head.peel_to_tree().unwrap();

        fill_file_entries_from_git(&repo, &tree, "");
    }
}

fn get_remote_callbacks(access_token: &str) -> git2::RemoteCallbacks {
    let mut remote_callbacks = git2::RemoteCallbacks::new();
    remote_callbacks.credentials(move |_url, _username_from_url, _allowed_types| {
        Cred::userpass_plaintext("token", access_token)
    });

    remote_callbacks
}

fn fill_file_entries_from_git(
    repo: &Repository,
    git_tree: &Tree,
    path: &str,
) {
    for git_entry in git_tree.iter() {
        let name = git_entry.name().unwrap().to_string();

        info!("Git -> Tree: processing Entry `{:?}`", name);

        match git_entry.kind() {
            Some(git2::ObjectType::Tree) => {

                let new_path = format!("{}{}", path, name);

                let git_children = git_entry.to_object(repo).unwrap().peel_to_tree().unwrap();

                fill_file_entries_from_git(
                    repo,
                    &git_children,
                    &new_path,
                );
            }
            Some(git2::ObjectType::Blob) => {

            }
            _ => {
                info!("Unknown file type: {:?}", git_entry.kind());
            }
        }
    }
}