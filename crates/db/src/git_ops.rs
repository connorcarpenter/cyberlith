use git::{
    git_commit, git_pull, git_push, read_file_bytes, write_file_bytes, ObjectType, Repository, Tree,
};
use logging::info;

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

pub fn pull_repo_get_all_files(root_path: &str, repo: &Repository) -> Vec<GitFile> {
    // pull all assets into memory, from "main" branch
    let mut output = Vec::new();
    let head = repo.head().unwrap();
    let tree = head.peel_to_tree().unwrap();

    collect_files(&mut output, root_path, repo, &tree, "");

    output
}

pub fn create_new_file<K: DbTableKey>(dir_path: &str, repo: &Repository, file: K::Value) {
    let branch_name = "main"; // TODO: parameterize?

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
    write_file_bytes(
        &mut index,
        &full_path,
        &file_name,
        file_contents,
        false,
        false,
    );

    // commit, push, pull
    git_commit(
        repo,
        branch_name,
        "connorcarpenter",
        "connorcarpenter@gmail.com",
        &commit_message,
    );
    git_push(repo, branch_name);
    git_pull(repo, branch_name);
}

pub fn update_file<K: DbTableKey>(dir_path: &str, repo: &Repository, file: &K::Value) {
    let branch_name = "main"; // TODO: parameterize?

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
    write_file_bytes(
        &mut index,
        &full_path,
        &file_name,
        file_contents,
        true,
        true,
    );

    // commit, push, pull
    git_commit(
        repo,
        branch_name,
        "connorcarpenter",
        "connorcarpenter@gmail.com",
        &commit_message,
    );
    git_push(repo, branch_name);
    git_pull(repo, branch_name);
}

pub fn update_nextid(dir_path: &str, repo: &Repository, next_id: u64) {
    let branch_name = "main"; // TODO: parameterize?

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
    write_file_bytes(
        &mut index,
        &full_path,
        &file_name,
        file_contents,
        true,
        true,
    );

    // commit, push, pull
    git_commit(
        repo,
        branch_name,
        "connorcarpenter",
        "connorcarpenter@gmail.com",
        &commit_message,
    );
    git_push(repo, branch_name);
    git_pull(repo, branch_name);
}

fn collect_files(
    output: &mut Vec<GitFile>,
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

                collect_files(output, root_path, repo, &git_children, &new_path);
            }
            Some(ObjectType::Blob) => {
                let bytes = read_file_bytes(root_path, file_path, &name);
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
