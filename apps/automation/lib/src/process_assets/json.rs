use std::{fs, fs::File, io::Read, path::Path};

use git2::{BranchType, Cred, PushOptions, Repository, Tree};
use log::info;

use crate::CliError;

pub(crate) fn process_assets() -> Result<(), CliError> {
    info!("Processing assets: 'json'");

    // pull all assets into memory, from "main" branch
    let root = "target/repo";
    let repo = repo_init(root);
    let files = load_all_files(root, &repo);

    // switch to "json" branch
    switch_branches(&repo, "json");

    // delete all files, push

    // create json file for each previous file

    // push

    Ok(())
}

fn repo_init(root_dir: &str) -> Repository {
    // Create Working directory if it doesn't already exist
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
        }

        info!("pulled repo with new changes");

        repo
    };

    repo
}

#[derive(Debug)]
struct FileEntry {
    path: String,
    name: String,
    file_ext: String,
    bytes: Vec<u8>,
}

fn load_all_files(root: &str, repo: &Repository) -> Vec<FileEntry> {

    let mut output = Vec::new();

    let head = repo.head().unwrap();
    let tree = head.peel_to_tree().unwrap();

    fill_file_entries_from_git(&mut output, root, &repo, &tree, "");

    output
}

// will create branch if necessary
fn switch_branches(repo: &Repository, branch_name: &str) {

    let access_token = include_str!("../../../../../.secrets/github_token");

    let mut fetch_options = git2::FetchOptions::new();
    fetch_options.remote_callbacks(get_remote_callbacks(access_token));

    let mut remote = repo.find_remote("origin").unwrap();
    remote
        .fetch(&[branch_name, "main"], Some(&mut fetch_options), None)
        .unwrap();

    let tracking_branch = format!("refs/remotes/origin/{}", branch_name);
    let branch_exists = repo.find_reference(&tracking_branch).is_ok();

    if branch_exists {
        // Remote branch exists, switch to it and pull
        info!("Remote branch: {:?} exists!", branch_name);
        let branch_reference = repo.find_reference(&tracking_branch).unwrap();
        let branch_commit = repo.find_commit(branch_reference.target().unwrap()).unwrap();
        let local_branch_name = format!("refs/heads/{}", branch_name);

        // Checkout the local branch
        let local_branch = repo.find_branch(&local_branch_name, BranchType::Local);
        let mut checkout_builder = git2::build::CheckoutBuilder::new();
        repo.checkout_tree(branch_commit.as_object(), Some(&mut checkout_builder)).unwrap();
        repo.set_head(&local_branch_name).unwrap();

        // Pull changes from the remote branch
        remote.fetch(&[branch_name], Some(&mut fetch_options), None).unwrap();
    } else {
        // Remote branch doesn't exist, create and push it
        info!("remote branch {:?} doesn't exist, creating..", branch_name);

        let branch_reference = repo.head().unwrap();
        let branch_commit = branch_reference.peel_to_commit().unwrap();

        // Create the local branch
        let local_branch_name = format!("refs/heads/{}", branch_name);
        let branch_reference = repo.branch(&local_branch_name, &branch_commit, true).unwrap();

        // Set up tracking to the remote branch
        repo.reference(
            &tracking_branch,
            branch_commit.id(),
            true,
            "Setting up tracking branch",
        ).unwrap();

        // Checkout the new branch
        let mut checkout_builder = git2::build::CheckoutBuilder::new();
        repo.checkout_tree(branch_commit.as_object(), Some(&mut checkout_builder)).unwrap();
        repo.set_head(&local_branch_name).unwrap();

        // Push the new branch to the remote
        let mut push_options = PushOptions::new();
        push_options.remote_callbacks(get_remote_callbacks(access_token));

        remote.push(&[&format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name)], Some(&mut push_options)).unwrap();

        info!("Created remote branch: {:?}", branch_name);
    }

    info!("switched to {:?} branch!", branch_name);
}

fn get_remote_callbacks(access_token: &str) -> git2::RemoteCallbacks {
    let mut remote_callbacks = git2::RemoteCallbacks::new();
    remote_callbacks.credentials(move |_url, _username_from_url, _allowed_types| {
        Cred::userpass_plaintext("token", access_token)
    });

    remote_callbacks
}

fn fill_file_entries_from_git(
    output: &mut Vec<FileEntry>,
    root: &str,
    repo: &Repository,
    git_tree: &Tree,
    path: &str,
) {
    for git_entry in git_tree.iter() {
        let name = git_entry.name().unwrap().to_string();

        match git_entry.kind() {
            Some(git2::ObjectType::Tree) => {

                let new_path = format!("{}{}", path, name);

                let git_children = git_entry.to_object(repo).unwrap().peel_to_tree().unwrap();

                fill_file_entries_from_git(
                    output,
                    root,
                    repo,
                    &git_children,
                    &new_path,
                );
            }
            Some(git2::ObjectType::Blob) => {

                let file_ext = name.split(".").last().unwrap();
                let bytes = get_file_contents(root, path, &name);

                let file_entry = FileEntry {
                    path: path.to_string(),
                    name: name.to_string(),
                    file_ext: file_ext.to_string(),
                    bytes,
                };

                info!("read file: {}, byte_len: {}", file_entry.name, file_entry.bytes.len());

                output.push(file_entry);
            }
            _ => {
                info!("Unknown file type: {:?}", git_entry.kind());
            }
        }
    }
}

fn get_file_contents(root: &str, path: &str, file: &str) -> Vec<u8> {
    let file_path = format!("{}{}", path, file);
    let full_path = format!("{}/{}", root, file_path);

    // info!("Getting blob for file: {}", full_path);

    let path = Path::new(full_path.as_str());
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(err) => panic!("Failed to open file: {}", err),
    };

    let mut contents = Vec::new();
    match file.read_to_end(&mut contents) {
        Ok(_) => contents,
        Err(err) => panic!("Failed to read file: {}", err),
    }
}