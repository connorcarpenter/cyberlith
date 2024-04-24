mod convert_to_bits;

use std::{fs, path::Path};

use asset_id::{AssetId, ETag};
use git::{branch_exists, ObjectType, create_branch, git_commit, git_pull, git_push, repo_init, Tree, Repository, switch_to_branch, write_file_bytes, read_file_bytes};
use logging::info;

use asset_serde::json::{Asset, AssetData, AssetMeta, ProcessedAssetMeta};

use crate::CliError;

pub fn process_assets(env: &str) -> Result<(), CliError> {
    let repo_name = "cyberlith_assets";
    let target_path = "target/asset_repo";

    // pull all assets into memory, from "env" branch
    let repo = repo_init(repo_name, target_path);
    let files = load_all_unprocessed_files(&target_path, &repo);

    if branch_exists(&repo, env) {
        update_processed_assets(env, &target_path, repo, files);
    } else {
        create_processed_assets(env, repo, files);
    }

    Ok(())
}

fn create_processed_assets(
    env: &str,
    repo: Repository,
    all_new_unprocessed_files: Vec<UnprocessedFile>,
) {
    info!(
        "branch {:?} doesn't exist, processing all files for the first time",
        env
    );
    // create new branch
    create_branch(&repo, env);
    switch_to_branch(&repo, env);

    // delete all files
    delete_all_files(&repo, &all_new_unprocessed_files);
    git_commit(&repo, env,  "connorcarpenter", "connorcarpenter@gmail.com", "deleting all unprocessed files");
    git_push(&repo, env);

    // process each file
    write_all_new_files(&repo, &all_new_unprocessed_files);
    git_commit(&repo, env, "connorcarpenter", "connorcarpenter@gmail.com", "processing all files");
    git_push(&repo, env);
}

fn update_processed_assets(
    env: &str,
    root: &str,
    repo: Repository,
    all_unprocessed_files: Vec<UnprocessedFile>,
) {
    info!("branch {:?} exists, processing only modified files..", env);
    // switch to "env" branch
    git_pull(&repo, env);
    switch_to_branch(&repo, env);

    // get files from previously processed environment
    let old_meta_files = load_all_processed_meta_files(root, &repo);

    // prune out unprocessed files that have not changed since last being processed
    let new_modified_unprocessed_files =
        prune_unchanged_files(&old_meta_files, all_unprocessed_files);
    if new_modified_unprocessed_files.is_empty() {
        info!("no files to process, exiting..");
        return;
    }

    // process each file
    write_all_new_files(&repo, &new_modified_unprocessed_files);
    git_commit(&repo, env, "connorcarpenter", "connorcarpenter@gmail.com", "processing all modified files");
    git_push(&repo, env);
}

struct UnprocessedFile {
    path: String,
    name: String,
    bytes: Vec<u8>,
}

impl UnprocessedFile {
    pub fn new(path: &str, name: &str, bytes: Vec<u8>) -> Self {
        Self {
            path: path.to_string(),
            name: name.to_string(),
            bytes,
        }
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
            Some(ObjectType::Tree) => {
                let new_path = format!("{}{}", path, name);

                let git_children = git_entry.to_object(repo).unwrap().peel_to_tree().unwrap();

                collect_unprocessed_files(output, root, repo, &git_children, &new_path);
            }
            Some(ObjectType::Blob) => {
                let bytes = read_file_bytes(root, path, &name);
                // let bytes_len = bytes.len();

                let file_entry = UnprocessedFile::new(path, &name, bytes);

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
    root_path: &str,
    repo: &Repository,
    git_tree: &Tree,
    file_path: &str,
) {
    for git_entry in git_tree.iter() {
        let name = git_entry.name().unwrap().to_string();

        match git_entry.kind() {
            Some(ObjectType::Tree) => {
                let new_file_path = format!("{}{}", file_path, name);

                let git_children = git_entry.to_object(repo).unwrap().peel_to_tree().unwrap();

                collect_processed_meta_files(output, root_path, repo, &git_children, &new_file_path);
            }
            Some(ObjectType::Blob) => {
                let name_split = name.split(".");
                let extension = name_split.last().unwrap();
                if extension != "meta" {
                    continue;
                }

                let bytes = read_file_bytes(root_path, file_path, &name);

                let processed_meta = ProcessedAssetMeta::read(&bytes).unwrap();

                info!("read meta file: {}", name);

                output.push(processed_meta);
            }
            _ => {
                info!("Unknown file type: {:?}", git_entry.kind());
            }
        }
    }
}

fn delete_all_files(repo: &Repository, file_entries: &Vec<UnprocessedFile>) {
    let mut index = repo.index().expect("Failed to open index");

    for file_entry in file_entries {
        let file_path = format!("{}{}", file_entry.path, file_entry.name);
        let full_path = format!("{}{}", repo.workdir().unwrap().to_str().unwrap(), file_path);

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
        let (unprocessed_asset_meta, unprocessed_asset_data) = unprocessed_asset.deconstruct();

        let asset_data_type_name = unprocessed_asset_data.type_name();
        if asset_data_type_name.as_str() != true_file_ext {
            panic!(
                "Expected file type to be: {}, got: {}",
                true_file_ext, asset_data_type_name
            );
        }
        let dependencies = get_dependencies(&unprocessed_asset_data);

        // convert asset data to bits
        let processed_asset_bytes = match unprocessed_asset_data {
            AssetData::Palette(data) => convert_to_bits::palette(data),
            AssetData::Scene(data) => convert_to_bits::scene(data),
            AssetData::Mesh(data) => convert_to_bits::mesh(data),
            AssetData::Skin(data) => convert_to_bits::skin(data),
            AssetData::Model(data) => convert_to_bits::model(data),
            AssetData::Skeleton(data) => convert_to_bits::skeleton(data),
            AssetData::Animation(data) => convert_to_bits::animation(data),
            AssetData::Icon(data) => convert_to_bits::icon(data),
            AssetData::Ui(data) => convert_to_bits::ui(data),
        };

        // write new data file
        write_file_bytes(&mut index, &file_path, &full_path, processed_asset_bytes, false, true);

        // process Asset Meta
        let meta_file_path = format!("{}.meta", file_path);
        let meta_full_path = format!("{}.meta", full_path);
        let processed_meta = process_new_meta_file(&unprocessed_asset_meta, dependencies, hash);
        let meta_bytes = processed_meta.write();

        // write new meta file
        write_file_bytes(&mut index, &meta_file_path, &meta_full_path, meta_bytes, false, true);
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
        AssetData::Ui(data) => data.dependencies(),
    }
}

fn process_new_meta_file(
    unprocessed_meta: &AssetMeta,
    dependencies: Vec<AssetId>,
    hash: AssetHash,
) -> ProcessedAssetMeta {
    ProcessedAssetMeta::new(
        unprocessed_meta.asset_id(),
        ETag::gen_random(),
        unprocessed_meta.schema_version(),
        dependencies,
        hash.to_vec(),
    )
}

fn prune_unchanged_files(
    old_meta_files: &Vec<ProcessedAssetMeta>,
    all_unprocessed_files: Vec<UnprocessedFile>,
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
            panic!(
                "Expected file type to be: {}, got: {}",
                true_file_ext, asset_data_type_name
            );
        }

        let prev_meta = old_meta_files
            .iter()
            .find(|meta| meta.asset_id() == unprocessed_asset.meta().asset_id());

        if let Some(meta) = prev_meta {
            if unprocessed_hash == meta.hash() {
                info!("file unchanged: {}", prev_path);
                continue;
            } else {
                info!("file changed: {}", prev_path);
            }
        } else {
            info!("file new: {}", prev_path);
        }

        output.push(unprocessed_file);
    }

    output
}