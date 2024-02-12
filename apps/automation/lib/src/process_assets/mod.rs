mod convert_to_bits;

use std::{fs, fs::File, io::{Read, Write}, path::Path};

use git2::{Cred, FetchOptions, Index, Oid, PushOptions, Repository, Signature, Tree};
use log::info;

use asset_io::{AssetId, ETag, json::{Asset, AssetData, AssetMeta, ProcessedAssetMeta}};

use crate::CliError;

pub fn process_assets(env: &str) -> Result<(), CliError> {
    // pull all assets into memory, from "main" branch
    let root = "target/repo";
    let repo = repo_init(root);
    let files = load_all_unprocessed_files(root, &repo);

    if branch_exists(&repo, env) {
        update_processed_assets(env, root, repo, files);
    } else {
        create_processed_assets(env, repo, files);
    }

    Ok(())
}

fn create_processed_assets(env: &str, repo: Repository, all_new_unprocessed_files: Vec<UnprocessedFile>) {
    info!("branch {:?} doesn't exist, processing all files for the first time", env);
    // create new branch
    create_branch(&repo, env);

    // delete all files
    delete_all_files(&repo, &all_new_unprocessed_files);
    git_commit(&repo, env, "deleting all unprocessed files");
    git_push(&repo, env);

    // process each file
    write_all_new_files(&repo, &all_new_unprocessed_files);
    git_commit(&repo, env, "processing all files");
    git_push(&repo, env);
}

fn update_processed_assets(env: &str, root: &str, repo: Repository, all_unprocessed_files: Vec<UnprocessedFile>) {
    info!("branch {:?} exists, processing only modified files..", env);
    // switch to "env" branch
    git_pull(&repo, env);
    switch_to_branch(&repo, env);

    // get files from previously processed environment
    let old_meta_files = load_all_processed_meta_files(root, &repo);

    // prune out unprocessed files that have not changed since last being processed
    let new_modified_unprocessed_files = prune_unchanged_files(&old_meta_files, all_unprocessed_files);
    if new_modified_unprocessed_files.is_empty() {
        info!("no files to process, exiting..");
        return;
    }

    // process each file
    write_all_new_files(&repo, &new_modified_unprocessed_files);
    git_commit(&repo, env, "processing all modified files");
    git_push(&repo, env);
}

fn get_remote_callbacks(access_token: &str) -> git2::RemoteCallbacks {
    let mut remote_callbacks = git2::RemoteCallbacks::new();
    remote_callbacks.credentials(move |_url, _username_from_url, _allowed_types| {
        Cred::userpass_plaintext("token", access_token)
    });

    remote_callbacks
}

fn get_fetch_options() -> FetchOptions<'static> {
    let access_token = include_str!("../../../../../.secrets/github_token");
    let mut fetch_options = FetchOptions::new();
    fetch_options.remote_callbacks(get_remote_callbacks(access_token));
    fetch_options
}

fn get_push_options() -> PushOptions<'static> {
    let access_token = include_str!("../../../../../.secrets/github_token");
    let mut push_options = PushOptions::new();
    push_options.remote_callbacks(get_remote_callbacks(access_token));
    push_options
}

fn repo_init(root_dir: &str) -> Repository {
    // Create Working directory if it doesn't already exist
    let path = Path::new(&root_dir);
    let repo_url = include_str!("../../../../../.secrets/assets_repo_url");
    let fetch_options = get_fetch_options();

    if path.exists() {
        info!("repo exists, removing..");
        fs::remove_dir_all(path).unwrap();
    }

    if path.exists() {
        panic!("should have removed directory: {:?}", root_dir);
    }

    // Create new directory
    fs::create_dir_all(path).unwrap();

    // Put fetch options into builder
    let mut builder = git2::build::RepoBuilder::new();
    builder.fetch_options(fetch_options);

    // Clone repo
    let repo = builder.clone(repo_url, path).unwrap();

    info!("initialized repo at: `{}`", root_dir);

    repo
}

struct UnprocessedFile {
    path: String,
    name: String,
    bytes: Vec<u8>,
}

impl UnprocessedFile {
    pub fn new(path: &str, name: &str, bytes: Vec<u8>) -> Self {
        Self { path: path.to_string(), name: name.to_string(), bytes }
    }
}

fn load_all_unprocessed_files(root: &str, repo: &Repository) -> Vec<UnprocessedFile> {

    let mut output = Vec::new();
    let head = repo.head().unwrap();
    let tree = head.peel_to_tree().unwrap();

    collect_unprocessed_files(&mut output, root, &repo, &tree, "");

    output
}

fn load_all_processed_meta_files(root: &str, repo: &Repository) -> Vec<ProcessedAssetMeta> {

    let mut output = Vec::new();
    let head = repo.head().unwrap();
    let tree = head.peel_to_tree().unwrap();

    collect_processed_meta_files(&mut output, root, &repo, &tree, "");

    output
}

fn collect_unprocessed_files(
    output: &mut Vec<UnprocessedFile>,
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

                collect_unprocessed_files(
                    output,
                    root,
                    repo,
                    &git_children,
                    &new_path,
                );
            }
            Some(git2::ObjectType::Blob) => {

                let bytes = get_file_contents(root, path, &name);
                // let bytes_len = bytes.len();

                let file_entry = UnprocessedFile::new(
                    path,
                    &name,
                    bytes,
                );

                info!("read file: {}", file_entry.name);

                output.push(file_entry);
            }
            _ => {
                info!("Unknown file type: {:?}", git_entry.kind());
            }
        }
    }
}

fn collect_processed_meta_files(
    output: &mut Vec<ProcessedAssetMeta>,
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

                collect_processed_meta_files(
                    output,
                    root,
                    repo,
                    &git_children,
                    &new_path,
                );
            }
            Some(git2::ObjectType::Blob) => {

                let name_split = name.split(".");
                let extension = name_split.last().unwrap();
                if extension != "meta" {
                    continue;
                }

                let bytes = get_file_contents(root, path, &name);

                let processed_meta = ProcessedAssetMeta::read(
                    &bytes,
                ).unwrap();

                info!("read meta file: {}", name);

                output.push(processed_meta);
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

fn branch_exists(repo: &Repository, branch_name: &str) -> bool {
    let remote_branch = format!("refs/remotes/origin/{}", branch_name);
    repo.find_reference(&remote_branch).is_ok()
}

fn create_branch(repo: &Repository, branch_name: &str) {

    // finding current commit, then creating a new local branch there
    let commit = repo.head().unwrap().peel_to_commit().unwrap();
    let _branch = repo.branch(branch_name, &commit, true).unwrap();
    let branch_ref = format!("refs/heads/{}", branch_name);

    // Push the new branch to the remote, (linking it to the remote branch)
    let mut remote = repo.find_remote("origin").unwrap();
    let mut push_options = get_push_options();
    remote.push(&[&format!("{}:{}", &branch_ref, &branch_ref)], Some(&mut push_options)).unwrap();

    info!("Created remote branch: {:?}", branch_name);

    // switch to branch
    switch_to_branch(repo, branch_name);
}

fn switch_to_branch(repo: &Repository, branch_name: &str) {
    let branch_ref = format!("refs/heads/{}", branch_name);
    repo.set_head(&branch_ref).unwrap();

    let mut checkout_builder = git2::build::CheckoutBuilder::new();
    checkout_builder.force();
    repo.checkout_head(Some(&mut checkout_builder)).unwrap();
    info!("switched to {:?} branch!", branch_name);
}

fn delete_all_files(repo: &Repository, file_entries: &Vec<UnprocessedFile>) {

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
        // info!("removed file from index: {}", file_path);
    }
}

fn git_commit(repo: &Repository, branch_name: &str, commit_message: &str) -> Oid {

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
    let author = Signature::now("connorcarpenter", "connorcarpenter@gmail.com").expect("Failed to create author signature");
    let committer =
        Signature::now("connorcarpenter", "connorcarpenter@gmail.com").expect("Failed to create committer signature");

    // Create the commit
    let commit_id = repo.commit(
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

fn git_push(repo: &Repository, branch_name: &str) {

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

fn git_pull(repo: &Repository, branch_name: &str) {

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
    let branch_ref_target_id = repo.refname_to_id(&branch_ref).unwrap_or_else(|_| branch_ref_target);
    repo.reference(&branch_ref, branch_ref_target_id, true, "Updating local reference").unwrap();

    // Push the new branch to the remote, (linking it to the remote branch)
    let mut push_options = get_push_options();
    remote.push(&[&format!("{}:{}", &branch_ref, &branch_ref)], Some(&mut push_options)).unwrap();

    info!("pulled from {:?} branch!", branch_name);
}

fn write_all_new_files(repo: &Repository, unprocessed_files: &Vec<UnprocessedFile>) {
    // let ref_name = format!("refs/heads/{}", branch_name);
    let mut index = repo.index().expect("Failed to open index");

    for unprocessed_file in unprocessed_files {

        // let prev_path = format!("{}/{}", unprocessed_file.path, unprocessed_file.name);
        // info!("processing file at path: {}", prev_path);

        let mut file_name_split = unprocessed_file.name.split(".");
        let file_name = file_name_split.next().unwrap();
        let true_file_ext = file_name_split.next().unwrap();
        let json_file_ext = file_name_split.next().unwrap();
        if json_file_ext != "json" {
            panic!("Expected file to be json, got: {}", json_file_ext);
        }

        let file_path = format!("{}{}.{}", unprocessed_file.path, file_name, true_file_ext);
        let full_path = format!("{}{}", repo.workdir().unwrap().to_str().unwrap(), file_path);

        let hash = get_asset_hash(&unprocessed_file.bytes);

        let unprocessed_asset = Asset::read(&unprocessed_file.bytes).expect("Failed to read asset");

        let asset_data_type_name = unprocessed_asset.data().type_name();
        if asset_data_type_name.as_str() != true_file_ext {
            panic!("Expected file type to be: {}, got: {}", true_file_ext, asset_data_type_name);
        }
        let dependencies = get_dependencies(&unprocessed_asset.data());

        // convert asset data to bits
        let processed_asset_bytes = match unprocessed_asset.data() {
            AssetData::Palette(data) => {
                convert_to_bits::palette(data)
            }
            AssetData::Scene(data) => {
                convert_to_bits::scene(data)
            }
            AssetData::Mesh(data) => {
                convert_to_bits::mesh(data)
            }
            AssetData::Skin(data) => {
                convert_to_bits::skin(data)
            }
            AssetData::Model(data) => {
                convert_to_bits::model(data)
            }
            AssetData::Skeleton(data) => {
                convert_to_bits::skeleton(data)
            }
            AssetData::Animation(data) => {
                convert_to_bits::animation(data)
            }
            AssetData::Icon(data) => {
                convert_to_bits::icon(data)
            }
        };

        // write new data file
        write_new_file(&mut index, &file_path, &full_path, processed_asset_bytes);

        // process Asset Meta
        let meta_file_path = format!("{}.meta", file_path);
        let meta_full_path = format!("{}.meta", full_path);
        let processed_meta = process_new_meta_file(&unprocessed_asset.meta(), dependencies, hash);
        let meta_bytes = processed_meta.write();

        // write new meta file
        write_new_file(&mut index, &meta_file_path, &meta_full_path, meta_bytes);
    }
}

pub type AssetHash = [u8; 32];

pub(crate) fn get_asset_hash(bytes: &[u8]) -> AssetHash {
    let mut hasher = blake3::Hasher::new();
    hasher.update(bytes);
    *hasher.finalize().as_bytes()
}

fn get_dependencies(data: &AssetData) -> Vec<AssetId> {
    match data {
        AssetData::Palette(data) => data.dependencies(),
        AssetData::Scene(data) => data.dependencies(),
        AssetData::Mesh(data) => data.dependencies(),
        AssetData::Skin(data) => data.dependencies(),
        AssetData::Model(data) => data.dependencies(),
        AssetData::Skeleton(data) => data.dependencies(),
        AssetData::Animation(data) => data.dependencies(),
        AssetData::Icon(data) => data.dependencies(),
    }
}

fn process_new_meta_file(unprocessed_meta: &AssetMeta, dependencies: Vec<AssetId>, hash: AssetHash) -> ProcessedAssetMeta {
    ProcessedAssetMeta::new(
        unprocessed_meta.asset_id(),
        ETag::get_random(),
        unprocessed_meta.schema_version(),
        dependencies,
        hash.to_vec(),
    )
}

fn write_new_file(index: &mut Index, file_path: &str, full_path: &str, bytes: Vec<u8>) {

    // if file exists, delete it
    let path = Path::new(full_path);
    let file_exists = path.exists();
    if file_exists {
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
    let mut file = match File::create(full_path) {
        Ok(file) => file,
        Err(err) => panic!("Failed to create file: {}", err),
    };
    match file.write_all(&bytes) {
        Ok(_) => {
            info!("wrote file: {}", file_path);
        }
        Err(err) => {
            info!("failed to write file: {}", err);
        }
    }

    // add_path will also update the index
    index
        .add_path(Path::new(&file_path))
        .expect("Failed to add file to index");

    if !file_exists {
        // info!("added file to index: {}", file_path);
    } else {
        // info!("updated file index: {}", file_path);
    }
}

fn prune_unchanged_files(
    old_meta_files: &Vec<ProcessedAssetMeta>,
    all_unprocessed_files: Vec<UnprocessedFile>
) -> Vec<UnprocessedFile> {
    let mut output = Vec::new();

    for unprocessed_file in all_unprocessed_files {
        let prev_path = format!("{}/{}", unprocessed_file.path, unprocessed_file.name);

        let mut file_name_split = unprocessed_file.name.split(".");
        let _file_name = file_name_split.next().unwrap();
        let true_file_ext = file_name_split.next().unwrap();
        let json_file_ext = file_name_split.next().unwrap();
        if json_file_ext != "json" {
            panic!("Expected file to be json, got: {}", json_file_ext);
        }

        let unprocessed_hash = get_asset_hash(&unprocessed_file.bytes);
        let unprocessed_asset = Asset::read(&unprocessed_file.bytes).expect("Failed to read asset");

        let asset_data_type_name = unprocessed_asset.data().type_name();
        if asset_data_type_name.as_str() != true_file_ext {
            panic!("Expected file type to be: {}, got: {}", true_file_ext, asset_data_type_name);
        }

        let prev_meta = old_meta_files.iter().find(|meta| meta.asset_id() == unprocessed_asset.meta().asset_id());

        if let Some(meta) = prev_meta {
            if unprocessed_hash == meta.hash() {
                info!("file unchanged: {}", prev_path);
                continue;
            }
            else {
                info!("file changed: {}", prev_path);
            }
        } else {
            info!("file new: {}", prev_path);
        }

        output.push(unprocessed_file);
    }

    output
}

// use std::{fs, fs::File, io::{Read, Write}, path::Path, collections::{HashSet, HashMap}};
//
// use git2::{Cred, PushOptions, Repository, Tree};
// use log::info;
// use crypto::U32Token;
//
// use asset_io::json::{Asset, AssetData, AssetMeta};
//
// use crate::{process_assets::convert, CliError};
//
// pub(crate) fn process_assets() -> Result<(), CliError> {
//     info!("Processing assets: 'json'");
//
//
//
//     Ok(())
// }
//
//
//
// fn push_to_branch(repo: &Repository, branch_name: &str) {
//     let access_token = include_str!("../../../../../.secrets/github_token");
//
//     let mut push_options = PushOptions::new();
//     push_options.remote_callbacks(get_remote_callbacks(access_token));
//
//     let mut remote = repo.find_remote("origin").unwrap();
//     remote.push(&[&format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name)], Some(&mut push_options)).unwrap();
//
//     info!("pushed to {:?} branch!", branch_name);
// }
//
//

//
// // will create branch if necessary
// fn switch_branches(repo: &Repository, branch_name: &str) {
//
//     let access_token = include_str!("../../../../../.secrets/github_token");
//
//     let mut fetch_options = git2::FetchOptions::new();
//     fetch_options.remote_callbacks(get_remote_callbacks(access_token));
//
//     let mut remote = repo.find_remote("origin").unwrap();
//     remote
//         .fetch(&[branch_name, "main"], Some(&mut fetch_options), None)
//         .unwrap();
//
//     let tracking_branch = format!("refs/remotes/origin/{}", branch_name);
//     let branch_exists = repo.find_reference(&tracking_branch).is_ok();
//     let local_branch_name = format!("refs/heads/{}", branch_name);
//
//     if branch_exists {
//         // Remote branch exists, switch to it and pull
//         info!("Remote branch: {:?} exists!", branch_name);
//         let branch_reference = repo.find_reference(&tracking_branch).unwrap();
//         let branch_commit = repo.find_commit(branch_reference.target().unwrap()).unwrap();
//
//         // Checkout the local branch
//         let mut checkout_builder = git2::build::CheckoutBuilder::new();
//         repo.checkout_tree(branch_commit.as_object(), Some(&mut checkout_builder)).unwrap();
//         repo.set_head(&local_branch_name).unwrap();
//
//         // Pull changes from the remote branch
//         remote.fetch(&[branch_name], Some(&mut fetch_options), None).unwrap();
//     } else {
//         // Remote branch doesn't exist, create and push it
//         info!("remote branch {:?} doesn't exist, creating..", branch_name);
//
//         let branch_reference = repo.head().unwrap();
//         let branch_commit = branch_reference.peel_to_commit().unwrap();
//
//         // Set up tracking to the remote branch
//         repo.reference(
//             &tracking_branch,
//             branch_commit.id(),
//             true,
//             "Setting up tracking branch",
//         ).unwrap();
//
//         // Push the new branch to the remote
//         let mut push_options = PushOptions::new();
//         push_options.remote_callbacks(get_remote_callbacks(access_token));
//
//         remote.push(&[&format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name)], Some(&mut push_options)).unwrap();
//
//         info!("Created remote branch: {:?}", branch_name);
//     }
//
//     info!("switched to {:?} branch!", branch_name);
// }
//
// // commits

//
//
