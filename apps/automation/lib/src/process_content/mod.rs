mod filetypes;
pub use filetypes::ProcessedFileMeta;
mod error;
pub use error::FileIoError;

use std::{fs, path::Path, time::Duration};

use asset_id::ETag;
use git::{branch_exists, ObjectType, create_branch, git_commit, git_pull, git_push, repo_init, Tree, Repository, switch_to_branch, write_file_bytes, read_file_bytes};
use logging::info;

use crate::{utils::{run_command, run_command_blocking}, CliError};

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

struct UnprocessedFile {
    full_file_path: String,
    file_name: String,
    extension: String,
    hash: FileHash,
}

// should build our deployments, process all files, and push to the content repo
pub fn process_content(
    // should be the directory of the entire cyberlith repo
    project_path: &str,
    // should be the directory we do all the work in, added to project_path
    service_path: &str,
    // what build environment are we in
    target_env: TargetEnv
) -> Result<(), CliError> {

    let repo_name = "cyberlith_content";
    let deployments = ["launcher", "game"];

    let target_path = format!("{}/{}/target", project_path, service_path);
    let output_path = format!("{}/{}", project_path, repo_name);
    let target_env_str = target_env.to_string();

    // build web deployments
    let mut files = build_deployments(target_env, project_path, &target_path, &deployments);

    // create repo
    let repo = repo_init(repo_name, &target_path);

    // if the repo already exists, process files if they have changed
    // otherwise, process all files
    if branch_exists(&repo, &target_env_str) {
        info!("branch {:?} exists, processing only modified files..", target_env_str);
        git_pull(&repo, &target_env_str);
        switch_to_branch(&repo, &target_env_str);

        // get files from previously processed environment
        let old_meta_files = load_all_processed_meta_files(&target_path, &repo);

        // prune out unprocessed files that have not changed since last being processed
        files = prune_unchanged_files(&old_meta_files, files);
        if files.is_empty() {
            info!("no files to process, exiting..");
            return Ok(());
        }
    } else {
        info!(
            "branch {:?} doesn't exist, processing all files for the first time",
            target_env_str
        );

        // create and switch to new branch
        create_branch(&repo, &target_env_str);
        switch_to_branch(&repo, &target_env_str);
    }

    // if release mode, wasm-opt on Wasm
    if target_env == TargetEnv::Prod {
        wasm_opt_deployments(project_path, &target_path, &files);
    }

    // if release mode, minify/uglify JS
    if target_env == TargetEnv::Prod {
        js_uglify(project_path, &target_path, &files);
    }

    // brotlify all files
    brotlify_deployments(project_path, &target_path, &files);

    // process each file
    write_processed_files_to_repo(&repo, &files);
    git_commit(&repo, &target_env_str, "connorcarpenter", "connorcarpenter@gmail.com", "processing all modified content files");
    git_push(&repo, &target_env_str);

    Ok(())
}

fn build_deployments(
    target_env: TargetEnv,
    // this is the working directory of the 'cyberlith' repo
    source_path: &str,
    // this is the directory the files should go into
    target_path: &str,
    deployments: &[&str]
) -> Vec<UnprocessedFile> {
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

        // rename 'deployment_bg.wasm' to 'deployment.wasm'
        let result = run_command_blocking(
            deployment,
            format!(
                "mv {}/{}_bg.wasm {}/{}.wasm",
                target_path,
                deployment,
                target_path,
                deployment,
            )
                .as_str(),
        );
        if let Err(e) = result {
            panic!("failed to rename wasm file: {}", e);
        }

        output.push(UnprocessedFile {
            full_file_path: format!("{}/{}.wasm", target_path, deployment),
            extension: "wasm".to_string(),
            hash: wasm_hash,
        });

        // get hash of js file
        let js_hash = {
            let js_bytes = read_file_bytes(
                target_path,
                "",
                format!("{}.js", deployment).as_str(),
            );
            get_file_hash(&js_bytes)
        };

        output.push(UnprocessedFile {
            full_file_path: format!("{}/{}.js", target_path, deployment),
            extension: "js".to_string(),
            hash: js_hash,
        });

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
        if let Err(e) = result {
            panic!("failed to copy html file: {}", e);
        }

        // get hash of html file
        let html_hash = {
            let html_bytes = read_file_bytes(
                target_path,
                "",
                format!("{}.html", deployment).as_str(),
            );
            get_file_hash(&html_bytes)
        };

        output.push(UnprocessedFile {
            full_file_path: format!("{}/{}.html", target_path, deployment),
            extension: "html".to_string(),
            hash: html_hash,
        });
    }

    output
}

fn wasm_opt_deployments(
    // this is the working directory of the 'cyberlith' repo
    source_path: &str,
    // this is the directory the files should go into
    target_path: &str,
    deployments: &[&str],
) {
    info!("run wasm-opt on deployments..");
    info!("source_path: {}", source_path);
    info!("target_path: {}", target_path);

    for deployment in deployments {
        // run wasm-opt
        let result = run_command_blocking(
            deployment,
            format!(
                "wasm-opt -Os -o {}/{}_opt.wasm {}/{}.wasm",
                target_path,
                deployment,
                target_path,
                deployment,
            )
                .as_str(),
        );
        if let Err(e) = result {
            panic!("failed to rename wasm file: {}", e);
        }

        // delete original wasm file
        let result = run_command_blocking(
            deployment,
            format!(
                "rm {}/{}.wasm",
                target_path,
                deployment,
            )
                .as_str(),
        );
        if let Err(e) = result {
            panic!("failed to delete wasm file: {}", e);
        }

        // rename *_opt.wasm to *.wasm
        let result = run_command_blocking(
            deployment,
            format!(
                "mv {}/{}_opt.wasm {}/{}.wasm",
                target_path,
                deployment,
                target_path,
                deployment,
            )
                .as_str(),
        );
        if let Err(e) = result {
            panic!("failed to rename wasm file: {}", e);
        }
    }
}

fn js_uglify(
    // this is the working directory of the 'cyberlith' repo
    source_path: &str,
    // this is the directory the files should go into
    target_path: &str,
    deployments: &[&str],
) {
    info!("run UglifyJS on deployments..");
    info!("source_path: {}", source_path);
    info!("target_path: {}", target_path);

    for deployment in deployments {
        // uglify
        let result = run_command_blocking(
            deployment,
            format!(
                "/home/connor/.nvm/versions/node/v18.6.0/bin/node /home/connor/.nvm/versions/node/v18.6.0/bin/uglifyjs {}/{}.js -o {}/{}_min.js --mangle --compress --no-annotations",
                target_path,
                deployment,
                target_path,
                deployment,
            )
                .as_str(),
        );
        if let Err(e) = result {
            panic!("failed to uglify js file: {}", e);
        }

        // delete non-minified js file
        let result = run_command_blocking(
            deployment,
            format!(
                "rm {}/{}.js",
                target_path,
                deployment,
            )
                .as_str(),
        );
        if let Err(e) = result {
            panic!("failed to delete js file: {}", e);
        }

        // rename *_min.js to *.js
        let result = run_command_blocking(
            deployment,
            format!(
                "mv {}/{}_min.js {}/{}.js",
                target_path,
                deployment,
                target_path,
                deployment,
            )
                .as_str(),
        );
    }
}

fn brotlify_deployments(
    // this is the working directory of the 'cyberlith' repo
    source_path: &str,
    // this is the directory the files should go into
    target_path: &str,
    deployments: &[&str],
) {
    info!("run Brotli on deployments..");
    info!("source_path: {}", source_path);
    info!("target_path: {}", target_path);

    for deployment in deployments {
        for ext in ["js", "wasm", "html"] {
            // brotlify
            let result = run_command_blocking(
                deployment,
                format!(
                    "brotli -9 {}/{}.{} -o {}/{}.{}.br",
                    target_path,
                    deployment,
                    ext,
                    target_path,
                    deployment,
                    ext,
                )
                    .as_str(),
            );
            if let Err(e) = result {
                panic!("failed to brotlify file: {}", e);
            }

            // delete original file
            let result = run_command_blocking(
                deployment,
                format!(
                    "rm {}/{}.{}",
                    target_path,
                    deployment,
                    ext,
                )
                    .as_str(),
            );
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

fn write_processed_files_to_repo(repo: &Repository, unprocessed_files: &Vec<UnprocessedFile>) {
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

        // copy file over

        // process Asset Meta
        let meta_file_path = format!("{}.meta", file_path);
        let meta_full_path = format!("{}.meta", full_path);
        let processed_meta = process_new_meta_file(&file_name, unprocessed_file.hash);
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

        let prev_meta = old_meta_files
            .iter()
            .find(|meta| meta.name().eq(&unprocessed_file.file_name));

        if let Some(meta) = prev_meta {
            if unprocessed_file.hash == meta.hash() {
                info!("file unchanged: {}", unprocessed_file.full_file_path);
                continue;
            } else {
                info!("file changed: {}", unprocessed_file.full_file_path);
            }
        } else {
            info!("file new: {}", unprocessed_file.full_file_path);
        }

        output.push(unprocessed_file);
    }

    output
}