use std::{fs, fs::File, io::Read, path::Path};

use git2::{Cred, FetchOptions, Index, Oid, PushOptions, Repository, Signature};

use logging::info;

// returns root path and repo
pub fn repo_init(repo_name: &str, target_path: &str) -> Repository {
    // Create Working directory if it doesn't already exist
    let target_dir_path = Path::new(&target_path);
    let repo_url_root = get_repo_url_root();
    let repo_url = format!("{}{}.git", repo_url_root, repo_name);
    let fetch_options = get_fetch_options();

    if target_dir_path.exists() {
        info!("repo: `{}` exists, removing..", target_path);
        fs::remove_dir_all(target_dir_path).unwrap();
    }

    if target_dir_path.exists() {
        panic!("should have removed directory but didn't!: {:?}", target_dir_path);
    }

    // Create new directory
    fs::create_dir_all(target_dir_path).unwrap();

    // Put fetch options into builder
    let mut builder = git2::build::RepoBuilder::new();
    builder.fetch_options(fetch_options);

    // Clone repo
    let repo = builder.clone(&repo_url, target_dir_path).unwrap();

    info!("initialized repo at: `{}`", target_dir_path.to_str().unwrap());

    repo
}

pub fn create_branch(repo: &Repository, branch_name: &str) {
    // finding current commit, then creating a new local branch there
    let commit = repo.head().unwrap().peel_to_commit().unwrap();
    let _branch = repo.branch(branch_name, &commit, true).unwrap();
    let branch_ref = format!("refs/heads/{}", branch_name);

    // Push the new branch to the remote, (linking it to the remote branch)
    let mut remote = repo.find_remote("origin").unwrap();
    let mut push_options = get_push_options();
    remote
        .push(
            &[&format!("{}:{}", &branch_ref, &branch_ref)],
            Some(&mut push_options),
        )
        .unwrap();

    info!("Created remote branch: {:?}", branch_name);
}

pub fn switch_to_branch(repo: &Repository, branch_name: &str) {
    let branch_ref = format!("refs/heads/{}", branch_name);
    repo.set_head(&branch_ref).unwrap();

    let mut checkout_builder = git2::build::CheckoutBuilder::new();
    checkout_builder.force();
    repo.checkout_head(Some(&mut checkout_builder)).unwrap();
    info!("switched to {:?} branch!", branch_name);
}

pub fn get_current_branch_name(repo: &Repository) -> String {
    let head = repo.head().unwrap();
    if !head.is_branch() {
        panic!("HEAD is detached");
    }
    let branch_name = head
        .shorthand()
        .expect("Failed to get current branch name")
        .to_string();
    branch_name
}

pub fn branch_exists(repo: &Repository, branch_name: &str) -> bool {
    let remote_branch = format!("refs/remotes/origin/{}", branch_name);
    repo.find_reference(&remote_branch).is_ok()
}

pub fn write_file_bytes(
    index: &mut Index,
    file_path: &str,
    full_path: &str,
    bytes: Vec<u8>,
    require_file_exist: bool,
    overwrite_existing_file: bool,
) {
    // if file exists, delete it
    let path = Path::new(full_path);
    let file_exists = path.exists();

    if require_file_exist {
        if !file_exists {
            panic!("file does not exist: {}", full_path);
        }
    }

    if file_exists {

        if !overwrite_existing_file {
            panic!("file already exists: {}", full_path);
        }

        match fs::remove_file(path) {
            Ok(_) => {
                // info!("deleted file: {}", file_path);
            }
            Err(err) => {
                info!("failed to delete file: {}", err);
            }
        }
    }

    // write data file
    match fs::write(full_path, &bytes) {
        Ok(()) => {}
        Err(err) => panic!("failed to write file: {}", err),
    };

    // add_path will also update the index
    index
        .add_path(Path::new(&file_path))
        .expect("Failed to add file to index");

    if !file_exists {
        info!("added file to index: {}", file_path);
    } else {
        info!("updated file index: {}", file_path);
    }
}

pub fn read_file_bytes(root_path: &str, path: &str, file: &str) -> Vec<u8> {
    let file_path = format!("{}{}", path, file);
    let full_path = format!("{}/{}", root_path, file_path);

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

pub fn git_update_index(repo: &Repository) {
    // get index
    let mut index = repo.index().expect("Failed to open index");

    // Get the updated tree
    index.write_tree().expect("Failed to write tree");
}

pub fn git_commit(
    repo: &Repository,
    branch_name: &str,
    username: &str,
    email: &str,
    commit_message: &str
) -> Oid {

    if get_current_branch_name(repo) != branch_name {
        panic!("current branch is not the same as the branch you are trying to commit to");
    }

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
    // TODO: parameterize my name and email?
    let author = Signature::now(username, email)
        .expect("Failed to create author signature");
    let committer = Signature::now(username, email)
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

    info!("committed to local {:?} branch!", branch_name);

    commit_id
}

pub fn git_pull(repo: &Repository, branch_name: &str) {
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

pub fn git_push(repo: &Repository, branch_name: &str) {
    let mut remote = repo
        .find_remote("origin")
        .expect("Failed to find remote 'origin'");
    let mut push_options = get_push_options();
    let branch_ref = format!("refs/heads/{}", branch_name);
    remote
        .push(&[branch_ref], Some(&mut push_options))
        .expect("Failed to push commit");

    info!("pushed to remote {:?} branch!", branch_name);
}

fn get_repo_url_root() -> &'static str {
    include_str!("../../../.secrets/db_repo_url_root")
}

fn get_remote_callbacks(access_token: &str) -> git2::RemoteCallbacks {
    let mut remote_callbacks = git2::RemoteCallbacks::new();
    remote_callbacks.credentials(move |_url, _username_from_url, _allowed_types| {
        Cred::userpass_plaintext("token", access_token)
    });

    remote_callbacks
}

fn get_fetch_options() -> FetchOptions<'static> {
    let access_token = include_str!("../../../.secrets/github_token");
    let mut fetch_options = FetchOptions::new();
    fetch_options.remote_callbacks(get_remote_callbacks(access_token));
    fetch_options
}

fn get_push_options() -> PushOptions<'static> {
    let access_token = include_str!("../../../.secrets/github_token");
    let mut push_options = PushOptions::new();
    push_options.remote_callbacks(get_remote_callbacks(access_token));
    push_options
}