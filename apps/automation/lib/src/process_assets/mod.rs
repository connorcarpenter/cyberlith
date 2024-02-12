use std::{fs, fs::File, io::Read, path::Path};

use git2::{Cred, FetchOptions, PushOptions, Repository, Signature, Tree};
use log::info;

use crate::CliError;

pub fn process_assets(env: &str) -> Result<(), CliError> {
    // pull all assets into memory, from "main" branch
    let root = "target/repo";
    let repo = repo_init(root);
    let files = load_all_unprocessed_files(root, &repo);

    if branch_exists(&repo, env) {
        update_processed_assets(env, root, repo, files);
    } else {
        create_processed_assets(env, root, repo, files);
    }

    Ok(())
}

fn create_processed_assets(env: &str, root: &str, repo: Repository, all_new_unprocessed_files: Vec<UnprocessedFile>) {
    info!("branch {} doesn't exist, creating..", env);
    // create new branch
    create_branch(&repo, env);

    // delete all files
    delete_all_files(&repo, &all_new_unprocessed_files);
    git_commit(&repo, env, "deleting all unprocessed files");
    git_push(&repo, env);

    // // process each file
    // write_all_files(&repo, env, &all_new_unprocessed_files);
    //
    // // push
    // push_to_branch(&repo, env);
}

fn update_processed_assets(env: &str, root: &str, repo: Repository, all_new_unprocessed_files: Vec<UnprocessedFile>) {
    info!("branch {} exists, updating..", env);
    // // switch to "env" branch
    // switch_branches(&repo, env);
    //
    // // get files from previously processed environment
    // let old_meta_files = load_all_meta_files(root, &repo);
    //
    // // prune out unprocessed files that have not changed since last being processed
    // let new_modified_unprocessed_files = prune_unchanged_files(&old_meta_files, &all_new_unprocessed_files);
    //
    // // process each modified file
    // write_all_files(&repo, env, &new_modified_unprocessed_files);
    //
    // // push
    // push_to_branch(&repo, env);
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

    info!("initializing repo at: `{}`", root_dir);

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

#[derive(Debug)]
struct UnprocessedFile {
    path: String,
    name: String,
    file_ext: String,
    bytes: Vec<u8>,
}

fn load_all_unprocessed_files(root: &str, repo: &Repository) -> Vec<UnprocessedFile> {

    let mut output = Vec::new();
    let head = repo.head().unwrap();
    let tree = head.peel_to_tree().unwrap();

    collect_unprocessed_files(&mut output, root, &repo, &tree, "");

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

                let file_ext = name.split(".").last().unwrap();
                let bytes = get_file_contents(root, path, &name);

                let file_entry = UnprocessedFile {
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

fn branch_exists(repo: &Repository, branch_name: &str) -> bool {

    // TODO: do we need to fetch here? before checking?
    // let mut fetch_options = get_fetch_options();
    //
    // let mut remote = repo.find_remote("origin").unwrap();
    // remote
    //     .fetch(&[branch_name, "main"], Some(&mut fetch_options), None)
    //     .unwrap();

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
    repo.set_head(&branch_ref).unwrap();
    repo.checkout_head(None).unwrap();
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
        info!("removed file from index: {}", file_path);
    }
}

fn git_commit(repo: &Repository, branch_name: &str, commit_message: &str) {

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
    repo.commit(
        Some("HEAD"),
        &author,
        &committer,
        commit_message,
        &repo.find_tree(tree_id).expect("Failed to find tree"),
        &[&parent_commit],
    )
        .expect("Failed to create commit");

    info!("committed to local {:?} branch!", branch_name);
}

fn git_push(repo: &Repository, branch_name: &str) {

    let mut remote = repo
        .find_remote("origin")
        .expect("Failed to find remote 'origin'");
    let mut push_options = get_push_options();
    remote
        .push(&[format!("refs/heads/{}", branch_name)], Some(&mut push_options))
        .expect("Failed to push commit");

    info!("pushed to remote {:?} branch!", branch_name);
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
// pub struct ProcessData {
//     pub(crate) asset_id: U32Token,
//     asset_data: AssetData,
//     new_file_path: String,
//     new_full_path: String,
// }
//
// fn write_all_files(repo: &Repository, branch_name: &str, file_entries: &Vec<FileEntry>) {
//     let ref_name = format!("refs/heads/{}", branch_name);
//     let mut index = repo.index().expect("Failed to open index");
//     let mut asset_ids = HashSet::<U32Token>::new();
//     let mut asset_id_map = HashMap::<String, ProcessData>::new();
//
//     for file_entry in file_entries {
//
//         let prev_path = format!("{}/{}", file_entry.path, file_entry.name);
//
//         info!("processing file at path: {}", prev_path);
//
//         let asset_id = {
//             loop {
//                 let id = U32Token::get_random();
//                 if !asset_ids.contains(&id) {
//                     asset_ids.insert(id);
//                     break id;
//                 }
//             }
//         };
//
//         let mut file_name_split = file_entry.name.split(".");
//         let file_name = file_name_split.next().unwrap();
//         let file_ext = match file_entry.file_ext.as_str() {
//             "skel" => "skeleton",
//             "anim" => "animation",
//             _ => file_entry.file_ext.as_str(),
//         };
//         let file_path = format!("{}{}.{}.json", file_entry.path, file_name, file_ext);
//         let full_path = format!("{}{}", repo.workdir().unwrap().to_str().unwrap(), file_path);
//
//         {
//             let in_bytes = &file_entry.bytes;
//             let asset_data = match file_ext {
//                 "palette" => {
//                     convert::palette(in_bytes)
//                 }
//                 "scene" => {
//                     convert::scene(in_bytes)
//                 }
//                 "mesh" => {
//                     convert::mesh(in_bytes)
//                 }
//                 "skin" => {
//                     convert::skin(in_bytes)
//                 }
//                 "model" => {
//                     convert::model(in_bytes)
//                 }
//                 "skeleton" => {
//                     convert::skel(in_bytes)
//                 }
//                 "animation" => {
//                     convert::anim(in_bytes)
//                 }
//                 "icon" => {
//                     convert::icon(in_bytes)
//                 }
//                 _ => {
//                     panic!("Unknown file type: {}", file_ext);
//                 }
//             };
//
//             asset_id_map.insert(prev_path, ProcessData {
//                 asset_id,
//                 asset_data,
//                 new_file_path: file_path,
//                 new_full_path: full_path,
//             });
//         }
//     }
//
//     for (_, process_data) in asset_id_map.iter() {
//
//         let ProcessData {
//             asset_id,
//             asset_data,
//             new_file_path,
//             new_full_path,
//         } = process_data;
//
//         let mut asset_data = asset_data.clone();
//         // asset_data.convert_to_asset_ids(&asset_id_map);
//
//         let asset = Asset {
//             meta: AssetMeta {
//                 asset_id: asset_id.as_string(),
//                 schema_version: 0,
//             },
//             data: asset_data.clone(),
//         };
//
//         let out_bytes: Vec<u8> = asset.to_pretty_json();
//
//         let mut file = match File::create(new_full_path) {
//             Ok(file) => file,
//             Err(err) => panic!("Failed to create file: {}", err),
//         };
//         match file.write_all(&out_bytes) {
//             Ok(_) => {
//                 info!("wrote file: {}", new_file_path);
//             }
//             Err(err) => {
//                 info!("failed to write file: {}", err);
//             }
//         }
//         // add to index
//         index
//             .add_path(Path::new(new_file_path))
//             .expect("Failed to add file to index");
//     }
//
//     let tree_id = index.write_tree().expect("Failed to write index");
//     let tree = repo.find_tree(tree_id).expect("Failed to find tree");
//     let signature = repo.signature().expect("Failed to get signature");
//     let parent_commit = repo.head().expect("Failed to get head").peel_to_commit().expect("Failed to peel to commit");
//
//     let commit_id = repo.commit(
//         Some("HEAD"),
//         &signature,
//         &signature,
//         &format!("committing to {:?}", branch_name),
//         &tree,
//         &[&parent_commit],
//     ).unwrap();
//
//     repo.reference(&ref_name, commit_id, true, "committing to branch").unwrap();
// }
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
