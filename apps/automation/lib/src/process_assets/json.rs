use std::{fs, fs::File, io::Read, path::Path};
use std::io::Write;

use git2::{Cred, PushOptions, Repository, Tree};
use log::info;

use super::convert;
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
    delete_all_files(&repo, "json", &files);
    push_to_branch(&repo, "json");

    // create json file for each previous file
    write_all_files(&repo, "json", &files);

    // push

    Ok(())
}

fn write_all_files(repo: &Repository, branch: &str, file_entries: &Vec<FileEntry>) {
    for file_entry in file_entries {
        let file_path = format!("{}{}", file_entry.path, file_entry.name);
        let full_path = format!("{}/{}", repo.workdir().unwrap().to_str().unwrap(), file_path);

        let path = Path::new(full_path.as_str());
        let mut file = match File::create(path) {
            Ok(file) => file,
            Err(err) => panic!("Failed to create file: {}", err),
        };

        let in_bytes = &file_entry.bytes;
        let out_bytes: Vec<u8> = match file_entry.file_ext.as_str() {
            "palette" => {
                convert::palette(in_bytes)
            }
            "scene" => {
                convert::scene(in_bytes)
            }
            "mesh" => {
                convert::mesh(in_bytes)
            }
            "skin" => {
                convert::skin(in_bytes)
            }
            "model" => {
                convert::model(in_bytes)
            }
            "skel" => {
                convert::skel(in_bytes)
            }
            "anim" => {
                convert::anim(in_bytes)
            }
            "icon" => {
                convert::icon(in_bytes)
            }
            _ => {
                in_bytes.to_vec()
            }
        };

        match file.write_all(&out_bytes) {
            Ok(_) => {
                info!("wrote file: {}", full_path);
            }
            Err(err) => {
                info!("failed to write file: {}", err);
            }
        }
    }
}

fn push_to_branch(repo: &Repository, branch_name: &str) {
    let access_token = include_str!("../../../../../.secrets/github_token");

    let mut push_options = PushOptions::new();
    push_options.remote_callbacks(get_remote_callbacks(access_token));

    let mut remote = repo.find_remote("origin").unwrap();
    remote.push(&[&format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name)], Some(&mut push_options)).unwrap();

    info!("pushed to {:?} branch!", branch_name);
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
    let local_branch_name = format!("refs/heads/{}", branch_name);

    if branch_exists {
        // Remote branch exists, switch to it and pull
        info!("Remote branch: {:?} exists!", branch_name);
        let branch_reference = repo.find_reference(&tracking_branch).unwrap();
        let branch_commit = repo.find_commit(branch_reference.target().unwrap()).unwrap();

        // Checkout the local branch
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

        // Set up tracking to the remote branch
        repo.reference(
            &tracking_branch,
            branch_commit.id(),
            true,
            "Setting up tracking branch",
        ).unwrap();

        // Push the new branch to the remote
        let mut push_options = PushOptions::new();
        push_options.remote_callbacks(get_remote_callbacks(access_token));

        remote.push(&[&format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name)], Some(&mut push_options)).unwrap();

        info!("Created remote branch: {:?}", branch_name);
    }

    info!("switched to {:?} branch!", branch_name);
}

// commits
fn delete_all_files(repo: &Repository, branch_name: &str, file_entries: &Vec<FileEntry>) {

    // Fetch the latest changes from the remote repository
    let access_token = include_str!("../../../../../.secrets/github_token");

    let mut fetch_options = git2::FetchOptions::new();
    fetch_options.remote_callbacks(get_remote_callbacks(access_token));

    let mut remote = repo.find_remote("origin").expect("Failed to find remote");
    remote.fetch(&["main", "json"], Some(&mut fetch_options), None)
        .expect("Failed to fetch changes from the remote");

    // get ref
    let ref_name = format!("refs/heads/{}", branch_name);
    // repo.set_head(&ref_name).expect("Failed to set head to the new branch");
    repo.checkout_head(Some(git2::build::CheckoutBuilder::new().force()))
        .expect("Failed to checkout the new branch");

    let mut index = repo.index().expect("Failed to open index");

    for file_entry in file_entries {
        let file_path = format!("{}{}", file_entry.path, file_entry.name);
        let full_path = format!("{}/{}", repo.workdir().unwrap().to_str().unwrap(), file_path);

        let path = Path::new(full_path.as_str());
        match fs::remove_file(path) {
            Ok(_) => {
                info!("deleted file: {}", full_path);
            }
            Err(err) => {
                info!("failed to delete file: {}", err);
            }
        }

        index
            .remove_path(Path::new(&file_path))
            .expect("Failed to remove file from index");

    }

    let tree_id = index.write_tree().expect("Failed to write index");
    let tree = repo.find_tree(tree_id).expect("Failed to find tree");

    let signature = repo.signature().expect("Failed to get signature");
    let parent_commit = repo.head().expect("Failed to get head").peel_to_commit().expect("Failed to peel to commit");

    let commit_id = repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        &format!("committing to {:?}", branch_name),
        &tree,
        &[&parent_commit],
    ).unwrap();

    repo.reference(&ref_name, commit_id, true, "committing to branch").unwrap();

    info!("committed to {:?} branch!", branch_name);
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