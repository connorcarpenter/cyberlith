use std::{fs, fs::File, io::Read, path::Path};

use git2::{Cred, FetchOptions, Index, Oid, PushOptions, Repository, Signature, Tree};
use log::info;

use crate::{DbRowValue, DbTableKey};

pub(crate) struct GitFile {
    pub(crate) name: String,
    pub(crate) bytes: Vec<u8>,
}

impl GitFile {
    pub(crate) fn new(name: &str, bytes: Vec<u8>) -> Self {
        Self {
            name: name.to_string(),
            bytes,
        }
    }
}

pub fn repo_init(repo_name: &str) -> (String, Repository) {
    // Create Working directory if it doesn't already exist
    let dir_name = format!("{}{}", "target/", repo_name);
    let dir_path = Path::new(&dir_name);
    let repo_url_root = get_repo_url_root();
    let repo_url = format!("{}{}.git", repo_url_root, repo_name);
    let fetch_options = get_fetch_options();

    if dir_path.exists() {
        info!("repo: `{}` exists, removing..", dir_name);
        fs::remove_dir_all(dir_path).unwrap();
    }

    if dir_path.exists() {
        panic!("should have removed directory but didn't!: {:?}", dir_path);
    }

    // Create new directory
    fs::create_dir_all(dir_path).unwrap();

    // Put fetch options into builder
    let mut builder = git2::build::RepoBuilder::new();
    builder.fetch_options(fetch_options);

    // Clone repo
    let repo = builder.clone(&repo_url, dir_path).unwrap();

    info!("initialized repo at: `{}`", dir_path.to_str().unwrap());

    (dir_name, repo)
}

pub fn pull_repo_get_all_files(dir_path: &str, repo: &Repository) -> Vec<GitFile> {
    // pull all assets into memory, from "main" branch
    let mut output = Vec::new();
    let head = repo.head().unwrap();
    let tree = head.peel_to_tree().unwrap();

    collect_files(&mut output, dir_path, repo, &tree, "");

    output
}

pub fn create_new_file<K: DbTableKey>(dir_path: &str, repo: &Repository, file: K::Value) {
    // get creds
    let file_name = file.get_file_name();
    let commit_message = file.get_insert_commit_message();
    let file_contents = file.to_bytes();

    let mut index = repo.index().expect("Failed to open index");

    info!("dir_path: {}", dir_path);
    let file_name = format!("{}.json", file_name);
    info!("file_name: {}", file_name);
    let file_path = format!("{}/{}", dir_path, file_name);
    info!("file_path: {}", file_path);
    let full_path = format!("{}{}", repo.workdir().unwrap().to_str().unwrap(), file_name);
    info!("full_path: {}", full_path);

    // write new file, add to index
    write_new_file(&mut index, &full_path, &file_name, file_contents);

    // commit, push, pull
    git_commit(repo, &commit_message);
    git_push(repo);
    git_pull(repo);
}

pub fn update_file<K: DbTableKey>(dir_path: &str, repo: &Repository, file: &K::Value) {
    // get creds
    let file_name = file.get_file_name();
    let commit_message = file.get_update_commit_message();
    let file_contents = file.to_bytes();

    let mut index = repo.index().expect("Failed to open index");

    info!("dir_path: {}", dir_path);
    let file_name = format!("{}.json", file_name);
    info!("file_name: {}", file_name);
    let file_path = format!("{}/{}", dir_path, file_name);
    info!("file_path: {}", file_path);
    let full_path = format!("{}{}", repo.workdir().unwrap().to_str().unwrap(), file_name);
    info!("full_path: {}", full_path);

    // update & write file
    update_file_impl(&mut index, &full_path, &file_name, file_contents);

    // commit, push, pull
    git_commit(repo, &commit_message);
    git_push(repo);
    git_pull(repo);
}

pub fn update_nextid(dir_path: &str, repo: &Repository, next_id: u64) {
    // get creds
    let file_name = ".nextid";
    let commit_message = format!("update .nextid to {}", next_id);
    let file_contents = serde_json::to_vec_pretty(&next_id).unwrap().to_vec();

    let mut index = repo.index().expect("Failed to open index");
    info!("dir_path: {}", dir_path);
    info!("file_name: {}", file_name);
    let file_path = format!("{}/{}", dir_path, file_name);
    info!("file_path: {}", file_path);
    let full_path = format!("{}{}", repo.workdir().unwrap().to_str().unwrap(), file_name);
    info!("full_path: {}", full_path);

    // update & write file
    update_file_impl(&mut index, &full_path, &file_name, file_contents);

    // commit, push, pull
    git_commit(repo, &commit_message);
    git_push(repo);
    git_pull(repo);
}

fn write_new_file(index: &mut Index, full_path: &str, file_path: &str, bytes: Vec<u8>) {
    let path = Path::new(full_path);
    let file_exists = path.exists();
    if file_exists {
        panic!("file already exists: {}", full_path);
    }

    // write data file
    match fs::write(full_path, &bytes) {
        Ok(()) => {}
        Err(err) => panic!("failed to write (file: `{}`) err: {}", full_path, err),
    };

    // add_path will also update the index
    if let Err(e) = index.add_path(Path::new(&file_path)) {
        panic!("Failed to add file `{}` to index: {}", file_path, e);
    }
}

fn update_file_impl(index: &mut Index, full_path: &str, file_path: &str, bytes: Vec<u8>) {
    let path = Path::new(full_path);
    let file_exists = path.exists();
    if !file_exists {
        panic!("file does not exist: {}", full_path);
    }

    // write data file
    match fs::write(full_path, &bytes) {
        Ok(()) => {}
        Err(err) => panic!("failed to write (file: `{}`) err: {}", full_path, err),
    };

    // add_path will also update the index
    if let Err(e) = index.add_path(Path::new(&file_path)) {
        panic!("Failed to add file `{}` to index: {}", file_path, e);
    }
}

fn get_remote_callbacks(access_token: &str) -> git2::RemoteCallbacks {
    let mut remote_callbacks = git2::RemoteCallbacks::new();
    remote_callbacks.credentials(move |_url, _username_from_url, _allowed_types| {
        Cred::userpass_plaintext("token", access_token)
    });

    remote_callbacks
}

fn get_access_token() -> &'static str {
    include_str!("../../../.secrets/github_token")
}

fn get_repo_url_root() -> &'static str {
    include_str!("../../../.secrets/db_repo_url_root")
}

fn get_fetch_options() -> FetchOptions<'static> {
    let access_token = get_access_token();
    let mut fetch_options = FetchOptions::new();
    fetch_options.remote_callbacks(get_remote_callbacks(access_token));
    fetch_options
}

fn get_push_options() -> PushOptions<'static> {
    let access_token = get_access_token();
    let mut push_options = PushOptions::new();
    push_options.remote_callbacks(get_remote_callbacks(access_token));
    push_options
}

fn collect_files(
    output: &mut Vec<GitFile>,
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

                collect_files(output, root, repo, &git_children, &new_path);
            }
            Some(git2::ObjectType::Blob) => {
                let bytes = get_file_contents(root, path, &name);
                // let bytes_len = bytes.len();

                let file_entry = GitFile::new(&name, bytes);

                info!("read file: {}", file_entry.name);

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

fn git_commit(repo: &Repository, commit_message: &str) -> Oid {
    let tree_id = repo
        .index()
        .expect("Failed to open index")
        .write_tree()
        .expect("Failed to write tree");

    let parent_commit = repo
        .head()
        .expect("Failed to get HEAD reference")
        .peel_to_commit()
        .expect("Failed to peel HEAD to commit");

    // Prepare the commit details
    let author = Signature::now("connorcarpenter", "connorcarpenter@gmail.com")
        .expect("Failed to create author signature");
    let committer = Signature::now("connorcarpenter", "connorcarpenter@gmail.com")
        .expect("Failed to create committer signature");

    // Create the commit
    let commit_id = repo
        .commit(
            Some("HEAD"),
            &author,
            &committer,
            commit_message,
            &repo.find_tree(tree_id).expect("Failed to find tree"),
            &[&parent_commit],
        )
        .expect("Failed to create commit");

    info!("committed to local `main` branch!");

    commit_id
}

fn git_push(repo: &Repository) {
    let mut remote = repo
        .find_remote("origin")
        .expect("Failed to find remote 'origin'");
    let mut push_options = get_push_options();
    let branch_ref = "refs/heads/main";
    remote
        .push(&[branch_ref], Some(&mut push_options))
        .expect("Failed to push commit");

    info!("pushed to remote `origin/main` branch!");
}

fn git_pull(repo: &Repository) {
    let branch_name = "main";
    // Fetch changes from the remote
    let mut remote = repo.find_remote("origin").unwrap();
    let mut fetch_options = get_fetch_options();
    remote
        .fetch(&[branch_name], Some(&mut fetch_options), None)
        .unwrap();

    // Get the updated reference after fetch
    let fetch_head = repo.find_reference("FETCH_HEAD").unwrap();
    let fetch_commit = repo.reference_to_annotated_commit(&fetch_head).unwrap();
    let fetch_commit_oid = fetch_commit.id();
    let fetch_commit_object = repo.find_object(fetch_commit_oid, None).unwrap();

    // Reset the local branch to the head of the remote branch
    let mut checkout_builder = git2::build::CheckoutBuilder::new();
    checkout_builder.force();

    // Reset local changes
    repo.reset(
        &fetch_commit_object,
        git2::ResetType::Hard,
        Some(&mut checkout_builder),
    )
    .unwrap();

    // Create a local reference pointing to the head of the local branch
    let branch_ref = format!("refs/heads/{}", branch_name);
    let branch_ref_target = fetch_commit_oid;
    let branch_ref_target_id = repo
        .refname_to_id(&branch_ref)
        .unwrap_or_else(|_| branch_ref_target);
    repo.reference(
        &branch_ref,
        branch_ref_target_id,
        true,
        "Updating local reference",
    )
    .unwrap();

    // Push the new branch to the remote, (linking it to the remote branch)
    let mut push_options = get_push_options();
    remote
        .push(
            &[&format!("{}:{}", &branch_ref, &branch_ref)],
            Some(&mut push_options),
        )
        .unwrap();

    info!("pulled from {:?} branch!", branch_name);
}
