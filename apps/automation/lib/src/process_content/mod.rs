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

    pub fn feature_flag(&self) -> String {
        match self {
            TargetEnv::Local => "local".to_string(),
            TargetEnv::Prod => "prod".to_string(),
        }
    }
}

struct UnprocessedFile {
    file_path: String,
    file_name: String,
    extension: FileExtension,
    hash: FileHash,
}

impl UnprocessedFile {
    pub fn new(file_path: &str, file_name: &str, extension: FileExtension, hash: FileHash) -> Self {
        Self {
            file_path: file_path.to_string(),
            file_name: file_name.to_string(),
            extension,
            hash,
        }
    }

    pub fn target_path(&self, target_path: &str) -> String {
        format!("{}/{}", target_path, self.file_path)
    }

    pub fn file_name_w_ext(&self) -> String {
        format!("{}.{}", self.file_name, self.extension.to_string())
    }

    pub fn relative_file_path(&self) -> String {
        format!("{}{}", self.file_path, self.file_name_w_ext())
    }

    pub fn full_file_path(&self, target_path: &str) -> String {
        format!("{}{}", self.target_path(target_path), self.file_name_w_ext())
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum FileExtension {
    Wasm,
    Js,
    Html,
}

impl FileExtension {
    pub fn to_string(&self) -> String {
        match self {
            FileExtension::Wasm => "wasm".to_string(),
            FileExtension::Js => "js".to_string(),
            FileExtension::Html => "html".to_string(),
        }
    }

    pub fn from_string(ext: &str) -> Self {
        match ext {
            "wasm" => FileExtension::Wasm,
            "js" => FileExtension::Js,
            "html" => FileExtension::Html,
            _ => panic!("unknown file extension: {}", ext),
        }
    }
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
    let output_path = format!("{}/{}", target_path, repo_name);
    let target_env_str = target_env.to_string();

    // build web deployments
    let mut files = build_deployments(target_env, project_path, &target_path, &deployments);

    // create repo
    let repo = repo_init(repo_name, &output_path);

    // if the repo already exists, process files if they have changed
    // otherwise, process all files
    if branch_exists(&repo, &target_env_str) {
        info!("branch {:?} exists, processing only modified files..", target_env_str);
        git_pull(&repo, &target_env_str);

        // get files from previously processed environment
        let old_meta_files = load_all_processed_meta_files(&output_path, &repo);

        // prune out unprocessed files that have not changed since last being processed
        files = prune_unchanged_files(&old_meta_files, files);
        if files.is_empty() {
            info!("no files to process, skipping processing");
            return Ok(());
        }
    } else {
        info!(
            "branch {:?} doesn't exist, processing all files for the first time",
            target_env_str
        );

        // create new branch since it doesn't exist
        create_branch(&repo, &target_env_str);
    }

    let files = files;
    switch_to_branch(&repo, &target_env_str);

    // if release mode, wasm-opt on Wasm
    if target_env == TargetEnv::Prod {
        wasm_opt_deployments(&target_path, &files);
    }

    // if release mode, minify/uglify JS
    if target_env == TargetEnv::Prod {
        js_uglify(&target_path, &files);
    }

    // brotlify all files
    brotlify_deployments(&target_path, &files);

    // write each file and meta file
    write_files_to_repo(&target_path, &repo, &files);

    // commit and push
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
        let feature_flag = target_env.feature_flag();
        let result = run_command_blocking(
            deployment,
            format!(
                "cargo build {}\
                    --features gl_renderer,{} \
                    --manifest-path {}/deployments/web/{}/Cargo.toml \
                    --target wasm32-unknown-unknown \
                    --target-dir {} \
                    --lib",
                release_arg,
                feature_flag,
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

        // add wasm file to output
        output.push(UnprocessedFile::new(
            "",
            format!("{}_bg", deployment).as_str(),
            FileExtension::Wasm,
            wasm_hash,
        ));

        // get hash of js file
        let js_hash = {
            let js_bytes = read_file_bytes(
                target_path,
                "",
                format!("{}.js", deployment).as_str(),
            );
            get_file_hash(&js_bytes)
        };

        // add js file to output
        output.push(UnprocessedFile::new(
            "",
            deployment,
            FileExtension::Js,
            wasm_hash,
        ));

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

        output.push(UnprocessedFile::new("", deployment, FileExtension::Html, html_hash));
    }

    output
}

fn wasm_opt_deployments(
    target_path: &str,
    files: &Vec<UnprocessedFile>,
) {
    info!("run wasm-opt on deployments..");

    for file in files {
        if file.extension != FileExtension::Wasm {
            continue;
        }
        // run wasm-opt
        let result = run_command_blocking(
            &file.file_name,
            format!(
                "wasm-opt -Os -o {}{}_opt.wasm {}{}.wasm",
                file.target_path(target_path),
                file.file_name,
                file.target_path(target_path),
                file.file_name,
            )
                .as_str(),
        );
        if let Err(e) = result {
            panic!("failed to rename wasm file: {}", e);
        }

        // delete original wasm file
        let result = run_command_blocking(
            &file.file_name,
            format!(
                "rm {}{}.wasm",
                file.target_path(target_path),
                file.file_name,
            )
                .as_str(),
        );
        if let Err(e) = result {
            panic!("failed to delete wasm file: {}", e);
        }

        // rename *_opt.wasm to *.wasm
        let result = run_command_blocking(
            &file.file_name,
            format!(
                "mv {}{}_opt.wasm {}/{}.wasm",
                file.target_path(target_path),
                file.file_name,
                file.target_path(target_path),
                file.file_name,
            )
                .as_str(),
        );
        if let Err(e) = result {
            panic!("failed to rename wasm file: {}", e);
        }
    }
}

fn js_uglify(
    target_path: &str,
    files: &Vec<UnprocessedFile>,
) {
    info!("run UglifyJS on deployments..");

    for file in files {
        if file.extension != FileExtension::Js {
            continue;
        }

        // uglify
        let result = run_command_blocking(
            &file.file_name,
            format!(
                "/home/connor/.nvm/versions/node/v18.6.0/bin/node /home/connor/.nvm/versions/node/v18.6.0/bin/uglifyjs {}{}.js -o {}{}_min.js --mangle --compress --no-annotations",
                file.target_path(target_path),
                file.file_name,
                file.target_path(target_path),
                file.file_name,
            )
                .as_str(),
        );
        if let Err(e) = result {
            panic!("failed to uglify js file: {}", e);
        }

        // delete non-minified js file
        let result = run_command_blocking(
            &file.file_name,
            format!(
                "rm {}{}.js",
                file.target_path(target_path),
                file.file_name,
            )
                .as_str(),
        );
        if let Err(e) = result {
            panic!("failed to delete js file: {}", e);
        }

        // rename *_min.js to *.js
        let result = run_command_blocking(
            &file.file_name,
            format!(
                "mv {}{}_min.js {}{}.js",
                file.target_path(target_path),
                file.file_name,
                file.target_path(target_path),
                file.file_name,
            )
                .as_str(),
        );
    }
}

fn brotlify_deployments(
    target_path: &str,
    files: &Vec<UnprocessedFile>,
) {
    info!("run Brotli on all files..");

    for file in files {

        // brotlify
        let result = run_command_blocking(
            &file.file_name,
            format!(
                "brotli -9 {}{}.{} -o {}{}_br.{}",
                file.target_path(target_path),
                file.file_name,
                file.extension.to_string(),
                file.target_path(target_path),
                file.file_name,
                file.extension.to_string(),
            )
                .as_str(),
        );
        if let Err(e) = result {
            panic!("failed to brotlify file: {}", e);
        }

        // delete original file
        let result = run_command_blocking(
            &file.file_name,
            format!(
                "rm {}{}.{}",
                file.target_path(target_path),
                file.file_name,
                file.extension.to_string(),
            )
                .as_str(),
        );

        // rename *_br.* to *.*
        let result = run_command_blocking(
            &file.file_name,
            format!(
                "mv {}{}_br.{} {}{}.{}",
                file.target_path(target_path),
                file.file_name,
                file.extension.to_string(),
                file.target_path(target_path),
                file.file_name,
                file.extension.to_string(),
            )
                .as_str(),
        );
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

fn write_files_to_repo(
    target_path: &str,
    repo: &Repository,
    files: &Vec<UnprocessedFile>
) {
    let repo_path = repo.workdir().unwrap().to_str().unwrap();
    // let ref_name = format!("refs/heads/{}", branch_name);
    let mut index = repo.index().expect("Failed to open index");

    for file in files {

        // copy file over into repo
        // if file exists, delete it
        {
            let repo_full_file_path_str = format!("{}{}", repo_path, file.relative_file_path());
            let repo_full_file_path = Path::new(&repo_full_file_path_str);
            let repo_file_exists = repo_full_file_path.exists();
            let result = run_command_blocking(
                &file.file_name,
                format!(
                    "mv {} {}",
                    file.full_file_path(target_path),
                    repo_full_file_path_str
                )
                    .as_str(),
            );
            if let Err(e) = result {
                panic!("failed to copy file over: {}", e);
            }

            // add_path will also update the index
            let index_path = file.relative_file_path();
            index
                .add_path(Path::new(&index_path))
                .expect("Failed to add file to index");

            if !repo_file_exists {
                info!("added file to index: {}", index_path);
            } else {
                info!("updated file index: {}", index_path);
            }
        }

        // process Asset Meta
        {
            let meta_file_path = format!("{}.meta", file.relative_file_path());
            let meta_full_target_path = format!("{}.meta", file.full_file_path(repo_path));
            let processed_meta = process_new_meta_file(&file.file_name_w_ext(), file.hash);
            let meta_bytes = processed_meta.write();

            // write new meta file
            write_file_bytes(&mut index, &meta_file_path, &meta_full_target_path, meta_bytes, false, true);
        }
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
            .find(|meta| meta.name().eq(&unprocessed_file.file_name_w_ext()));

        let Some(meta) = prev_meta else {
            info!("file new: {}", unprocessed_file.relative_file_path());
            output.push(unprocessed_file);
            continue;
        };
        if unprocessed_file.hash != meta.hash() {
            info!("file changed: {}", unprocessed_file.relative_file_path());
            output.push(unprocessed_file);
            continue;
        }

        info!("file unchanged: {}", unprocessed_file.relative_file_path());
        // do not add to output
    }

    output
}