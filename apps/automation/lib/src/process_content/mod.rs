mod filetypes;
pub use filetypes::ProcessedFileMeta;
mod error;
pub use error::FileIoError;

use std::{fs, path::Path};

use asset_id::ETag;
use git::{branch_exists, ObjectType, create_branch, git_commit, git_pull, git_push, repo_init, Tree, Repository, switch_to_branch, write_file_bytes, read_file_bytes};
use logging::info;

use crate::CliError;
use crate::utils::{run_command, run_command_blocking};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum TargetEnv {
    Local,
    Prod,
}

impl TargetEnv {
    pub fn to_string(&self) -> String {
        match self {
            TargetEnv::Local => "local".to_string(),
            TargetEnv::Prod => "prod".to_string(),
        }
    }

    pub fn cargo_env(&self) -> String {
        match self {
            TargetEnv::Local => "debug".to_string(),
            TargetEnv::Prod => "release".to_string(),
        }
    }
}

// should build our deployments, process all files, and push to the content repo
pub fn process_content(
    // should be the directory of the entire cyberlith repo
    source_path: &str,
    // should be the directory we do all the work in, added to source_path
    target_path: &str,
    // what environment are we in
    target_env: TargetEnv
) -> Result<(), CliError> {

    let target_path = format!("{}/{}", source_path, target_path);

    let deployments = ["launcher", "game"];

    // build web deployments
    build_deployments(target_env, source_path, &target_path, &deployments);

    todo!();

    let repo_name = "cyberlith_content";

    // wasm
    // let wasm_paths = [
    //     ("launcher", "launcher_bg.wasm"),
    //     ("game", "game_bg.wasm")
    // ].map(
    //     |(deployment, file_name)| format!(
    //         "{}/deployments/web/{}/target/wasm32-unknown-unknown/{}/{}",
    //         source_path,
    //         deployment,
    //         target_env.cargo_env(),
    //         file_name,
    //     ));
    //
    // // js
    // let js_paths = [
    //     "launcher",
    //     "game"
    // ].map(
    //     |deployment| format!(
    //         "{}/deployments/web/{}/target/wasm32-unknown-unknown/{}/{}.js",
    //         source_path,
    //         deployment,
    //         target_env.cargo_env(),
    //         deployment
    //     ));
    //
    // // html
    // let html_paths = [
    //     "launcher",
    //     "game"
    // ].map(
    //     |deployment| format!(
    //         "{}/deployments/web/{}/{}.html",
    //         source_path,
    //         deployment,
    //         deployment
    //     ));

    // // if release mode, wasm-opt
    // if target_env == TargetEnv::Prod {
    //     wasm_opt_deployments(&wasm_paths)
    // }
    //
    // // if release mode, minify/uglify JS
    // if target_env == TargetEnv::Prod {
    //     js_uglify(&js_paths);
    // }
    //
    // let mut file_paths = wasm_paths.to_vec();
    // file_paths.extend(js_paths);
    // file_paths.extend(html_paths);
    //
    // // load all files into memory
    // let files = load_all_unprocessed_files(&file_paths);
    //
    // // create repo
    // let repo = repo_init(repo_name, target_path);
    //
    // // if the repo already exists, process files if they have changed
    // // otherwise, process all files
    // let target_env_str = target_env.to_string();
    // if branch_exists(&repo, &target_env_str) {
    //     update_processed_content(target_env, &target_path, &repo, files);
    // } else {
    //     create_processed_content(target_env, &repo, files);
    // }

    Ok(())
}

fn build_deployments(
    target_env: TargetEnv,
    // this is the working directory of the 'cyberlith' repo
    source_path: &str,
    // this is the directory the files should go into
    target_path: &str,
    deployments: &[&str]
) -> Vec<(String, FileHash, FileHash, FileHash)> {
    info!("building deployments..");
    info!("source_path: {}", source_path);
    info!("target_path: {}", target_path);
    info!("target env: {}", target_env.to_string());

    let mut output = Vec::new();

    for deployment in deployments {
        // cargo build
        let release_arg = if target_env == TargetEnv::Prod { "--release " } else { "" };
        let result = run_command_blocking(
            deployment,
            format!(
                "cargo build {}\
                    --features gl_renderer,prod \
                    --manifest-path {}/deployments/web/{}/Cargo.toml \
                    --target wasm32-unknown-unknown \
                    --target-dir {} \
                    --lib",
                release_arg,
                source_path,
                deployment,
                target_path,
            )
                .as_str(),
        );
        if let Err(e) = result {
            panic!("failed to build deployment: {}", e);
        }

        // get hash of wasm file
        let wasm_hash = {
            let wasm_bytes = read_file_bytes(
                target_path,
                format!("wasm32-unknown-unknown/{}/", target_env.cargo_env()).as_str(),
                format!("{}.wasm", deployment).as_str(),
            );
            get_file_hash(&wasm_bytes)
        };

        // then wasm-bindgen the wasm file
        let wasm_file_path = format!(
            "{}/wasm32-unknown-unknown/{}/{}.wasm",
            target_path,
            target_env.cargo_env(),
            deployment
        );
        let result = run_command_blocking(
            deployment,
            format!(
                "wasm-bindgen \
                    --out-dir {} \
                    --out-name {} \
                    --target web \
                    --no-typescript {}",
                target_path, deployment, wasm_file_path
            )
                .as_str(),
        );
        if let Err(e) = result {
            panic!("failed to wasm-bindgen deployment: {}", e);
        }

        // get hash of js file
        let js_hash = {
            let js_bytes = read_file_bytes(
                target_path,
                "",
                format!("{}_bg.wasm", deployment).as_str(),
            );
            get_file_hash(&js_bytes)
        };

        // copy html file over
        let result = run_command_blocking(
            deployment,
            format!(
                "cp {}/deployments/web/{}/{}.html {}/{}.html",
                source_path,
                deployment,
                deployment,
                target_path,
                deployment,
            )
                .as_str(),
        );

        // get hash of html file
        let html_hash = {
            let html_bytes = read_file_bytes(
                target_path,
                "",
                format!("{}.html", deployment).as_str(),
            );
            get_file_hash(&html_bytes)
        };

        output.push((deployment.to_string(), wasm_hash, js_hash, html_hash));
    }

    output
}

fn wasm_opt_deployments(wasm_files: &[String; 2]) {
    for wasm_file in wasm_files {
        todo!()
    }
}

fn js_uglify(js_files: &[String; 2]) {
    for js_file in js_files {
        todo!()
    }
}

fn create_processed_content(
    env: TargetEnv,
    repo: &Repository,
    all_new_unprocessed_files: Vec<UnprocessedFile>,
) {
    let env_str = env.to_string();
    info!(
        "branch {:?} doesn't exist, processing all files for the first time",
        env
    );
    // create new branch
    create_branch(repo, &env_str);
    switch_to_branch(repo, &env_str);

    // delete all files
    delete_all_files(&repo, &all_new_unprocessed_files);
    git_commit(repo, &env_str, "connorcarpenter", "connorcarpenter@gmail.com", "deleting all unprocessed files");
    git_push(repo, &env_str);

    // process each file
    process_and_write_all_files(repo, &all_new_unprocessed_files);
    git_commit(repo, &env_str, "connorcarpenter", "connorcarpenter@gmail.com", "processing all files");
    git_push(repo, &env_str);
}

fn update_processed_content(
    env: TargetEnv,
    root: &str,
    repo: &Repository,
    all_unprocessed_files: Vec<UnprocessedFile>,
) {
    info!("branch {:?} exists, processing only modified files..", env);

    let env_str = env.to_string();

    // switch to "env" branch
    git_pull(repo, &env_str);
    switch_to_branch(repo, &env_str);

    // get files from previously processed environment
    let old_meta_files = load_all_processed_meta_files(root, repo);

    // prune out unprocessed files that have not changed since last being processed
    let new_modified_unprocessed_files =
        prune_unchanged_files(&old_meta_files, all_unprocessed_files);
    if new_modified_unprocessed_files.is_empty() {
        info!("no files to process, exiting..");
        return;
    }

    // process each file
    process_and_write_all_files(repo, &new_modified_unprocessed_files);
    git_commit(repo, &env_str, "connorcarpenter", "connorcarpenter@gmail.com", "processing all modified files");
    git_push(repo, &env_str);
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

fn load_all_unprocessed_files(file_paths: &Vec<String>) -> Vec<UnprocessedFile> {
    let mut output = Vec::new();

    for file_path in file_paths {
        todo!()
    }

    output
}

fn load_all_processed_meta_files(root_path: &str, repo: &Repository) -> Vec<ProcessedFileMeta> {
    let mut output = Vec::new();
    let head = repo.head().unwrap();
    let tree = head.peel_to_tree().unwrap();

    collect_processed_meta_files(&mut output, root_path, &repo, &tree, "");

    output
}

fn collect_processed_meta_files(
    output: &mut Vec<ProcessedFileMeta>,
    root_path: &str,
    repo: &Repository,
    git_tree: &Tree,
    file_path: &str,
) {
    for git_entry in git_tree.iter() {
        let name = git_entry.name().unwrap().to_string();

        match git_entry.kind() {
            Some(ObjectType::Tree) => {
                let new_path = format!("{}{}", file_path, name);

                let git_children = git_entry.to_object(repo).unwrap().peel_to_tree().unwrap();

                collect_processed_meta_files(output, root_path, repo, &git_children, &new_path);
            }
            Some(ObjectType::Blob) => {
                let name_split = name.split(".");
                let extension = name_split.last().unwrap();
                if extension != "meta" {
                    continue;
                }

                let bytes = read_file_bytes(root_path, file_path, &name);

                let processed_meta = ProcessedFileMeta::read(&bytes).unwrap();

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

fn process_and_write_all_files(repo: &Repository, unprocessed_files: &Vec<UnprocessedFile>) {
    // let ref_name = format!("refs/heads/{}", branch_name);
    let mut index = repo.index().expect("Failed to open index");

    for unprocessed_file in unprocessed_files {
        // let prev_path = format!("{}/{}", unprocessed_file.path, unprocessed_file.name);
        // info!("processing file at path: {}", prev_path);

        let mut file_name_split = unprocessed_file.name.split(".");
        let file_name = file_name_split.next().unwrap();
        let true_file_ext = file_name_split.next().unwrap();

        let file_path = format!("{}{}.{}", unprocessed_file.path, file_name, true_file_ext);
        let full_path = format!("{}{}", repo.workdir().unwrap().to_str().unwrap(), file_path);

        let hash = get_file_hash(&unprocessed_file.bytes);

        // TODO!!!: do things with bytes of content here!
        // wasm-opt, and brotli-fy
        let processed_file_bytes = unprocessed_file.bytes.clone();

        // write new data file
        write_file_bytes(&mut index, &file_path, &full_path, processed_file_bytes, false, true);

        // process Asset Meta
        let meta_file_path = format!("{}.meta", file_path);
        let meta_full_path = format!("{}.meta", full_path);
        let processed_meta = process_new_meta_file(&file_name, hash);
        let meta_bytes = processed_meta.write();

        // write new meta file
        write_file_bytes(&mut index, &meta_file_path, &meta_full_path, meta_bytes, false, true);
    }
}

pub type FileHash = [u8; 32];

pub(crate) fn get_file_hash(bytes: &[u8]) -> FileHash {
    let mut hasher = blake3::Hasher::new();
    hasher.update(bytes);
    *hasher.finalize().as_bytes()
}

fn process_new_meta_file(
    file_name: &str,
    hash: FileHash,
) -> ProcessedFileMeta {
    ProcessedFileMeta::new(
        file_name,
        ETag::gen_random(),
        hash.to_vec(),
    )
}

fn prune_unchanged_files(
    old_meta_files: &Vec<ProcessedFileMeta>,
    all_unprocessed_files: Vec<UnprocessedFile>,
) -> Vec<UnprocessedFile> {
    let mut output = Vec::new();

    for unprocessed_file in all_unprocessed_files {
        let prev_path = format!("{}/{}", unprocessed_file.path, unprocessed_file.name);

        let mut file_name_split = unprocessed_file.name.split(".");
        let file_name = file_name_split.next().unwrap();
        let true_file_ext = file_name_split.next().unwrap();

        let unprocessed_hash = get_file_hash(&unprocessed_file.bytes);

        let prev_meta = old_meta_files
            .iter()
            .find(|meta| &meta.name() == file_name);

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