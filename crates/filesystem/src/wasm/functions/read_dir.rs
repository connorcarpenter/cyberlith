use std::path::Path;

use js_sys::Promise;

use crate::ReadDir;

pub fn read_dir<P: AsRef<Path>>(path: P) -> std::io::Result<ReadDir> {
    let window = web_sys::window().expect("no global `window` exists");
    let navigator = window.navigator();
    let storage_manager = navigator.storage();
    let opfs_root_promise: Promise = storage_manager.get_directory();

    todo!()
}
