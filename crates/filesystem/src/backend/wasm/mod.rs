use bevy_tasks::AsyncComputeTaskPool;

use crossbeam_channel::{bounded, Receiver};
use js_sys::{Array, AsyncIterator, Function, Promise};
use log::info;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::{FileSystemDirectoryHandle, FileSystemGetDirectoryOptions, FileSystemGetFileOptions, FileSystemFileHandle, FileSystemWritableFileStream, WritableStream};

use crate::{tasks::{write::WriteTask, read_dir::{ReadDirTask, ReadDirEntry}, read::ReadTask, create_dir::CreateDirTask, task_enum::{FsTaskEnum, FsTaskResultEnum}}, error::TaskError, ReadDirResult, CreateDirResult, WriteResult};

pub(crate) struct FsTaskJob(pub Receiver<Result<FsTaskResultEnum, TaskError>>);

pub(crate) fn start_task(
    task_enum: FsTaskEnum,
) -> FsTaskJob {
    let thread_pool = AsyncComputeTaskPool::get();

    let (tx, task) = bounded(1);
    thread_pool
        .spawn(async move {
            let result = crate::backend::task_process_async(task_enum).await;
            tx.send(result).ok();
        })
        .detach();

    FsTaskJob(task)
}

pub(crate) fn poll_task(task: &mut FsTaskJob) -> Option<Result<FsTaskResultEnum, TaskError>> {
    match task.0.try_recv() {
        Ok(Ok(result_enum)) => Some(Ok(result_enum)),
        Ok(Err(error)) => Some(Err(error)),
        Err(_) => None,
    }
}

pub async fn task_process_async(
    task_enum: &FsTaskEnum,
) -> Result<FsTaskResultEnum, TaskError> {
    match task_enum {
        FsTaskEnum::Read(task) => handle_read(task).await,
        FsTaskEnum::Write(task) => handle_write(task).await,
        FsTaskEnum::ReadDir(task) => handle_read_dir(task).await,
        FsTaskEnum::CreateDir(task) => handle_create_dir(task).await,
    }
}

async fn handle_read(task: &ReadTask) -> Result<FsTaskResultEnum, TaskError> {
    todo!()
}

async fn handle_write(task: &WriteTask) -> Result<FsTaskResultEnum, TaskError> {
    let output = WriteResult::new();

    let root = get_root().await;

    let folder_name = task.path.parent().unwrap().to_str().unwrap();

    let dir_handle_promise = root.get_directory_handle(&folder_name);
    let dir_handle_js = JsFuture::from(dir_handle_promise).await.expect("Error getting directory handle");
    let dir_handle: FileSystemDirectoryHandle = dir_handle_js.try_into().expect("Failed to cast JsValue to FileSystemDirectoryHandle");

    let file_name = task.path.file_name().unwrap().to_str().unwrap();

    // create file
    let mut options = FileSystemGetFileOptions::new();
    options.create(true);
    let file_handle_promise = dir_handle.get_file_handle_with_options(file_name, &options);

    info!("attempting to create file handle with name: {:?}", file_name);
    let file_handle_js = JsFuture::from(file_handle_promise).await.expect("Error creating file handle");
    info!("file handle created");

    let file_handle: FileSystemFileHandle = file_handle_js.try_into().expect("Failed to cast JsValue to FileSystemFileHandle");

    // create file write stream
    let file_stream_promise = file_handle.create_writable();
    let file_stream_js = JsFuture::from(file_stream_promise).await.expect("Error creating file stream");
    let file_stream: FileSystemWritableFileStream = file_stream_js.clone().try_into().expect("Failed to cast JsValue to FileSystemWritableFileStream");

    // write to file
    let write_promise = file_stream.write_with_u8_array(task.bytes.as_ref()).expect("Error writing to file");
    // from documentation, this promise should return 'undefined'
    let _ = JsFuture::from(write_promise).await.expect("Error resolving file writing promise");

    // close file write stream
    let writeable_stream: WritableStream = file_stream_js.try_into().expect("Failed to cast JsValue to WritableStream");
    let close_promise = writeable_stream.close();
    let _ = JsFuture::from(close_promise).await.expect("Error closing file stream");

    Ok(FsTaskResultEnum::Write(output))
}

async fn handle_read_dir(task: &ReadDirTask) -> Result<FsTaskResultEnum, TaskError> {

    let mut output = ReadDirResult::new();

    let root = get_root().await;

    let folder_name = task.path.clone().into_os_string().into_string().unwrap();
    let dir_handle_promise = root.get_directory_handle(&folder_name);
    let dir_handle_js = match JsFuture::from(dir_handle_promise).await {
        Ok(dir_handle_js) => dir_handle_js,
        Err(e) => {
            let error_string = format!("Error getting directory handle: {:?}", e);
            // NOTE: need this error, in order to create new dir for later!
            return Err(TaskError::IoError(error_string));
        }
    };
    let dir_handle: FileSystemDirectoryHandle = dir_handle_js.try_into().expect("Failed to cast JsValue to FileSystemDirectoryHandle");

    let dir_handle_js = JsValue::from(dir_handle);

    // Get the JavaScript function for the 'entries' method
    let entries_function_js = js_sys::Reflect::get(
        &dir_handle_js,
        &JsValue::from("entries"),
    )
        .unwrap();
    let entries_function: Function = entries_function_js.try_into().expect("Failed to cast JsValue to Function");

    // Call 'entries' method using Reflect::apply()
    let arguments_list = Array::new();
    let async_iterator_js = js_sys::Reflect::apply(&entries_function, &dir_handle_js, &arguments_list)
        .expect("Failed to call entries method");
    let async_iterator: AsyncIterator = async_iterator_js.try_into().expect("Failed to cast JsValue to AsyncIterator");

    loop {
        let next_promise = async_iterator.next().expect("Error getting next entry (before promise)");

        let next_entry = JsFuture::from(next_promise).await.expect("Error getting next entry (after promise)");
        let done = js_sys::Reflect::get(&next_entry, &JsValue::from_str("done"))
            .unwrap()
            .as_bool()
            .unwrap();
        if done {
            info!("Done with directory iterator!");
            break;
        }
        let value = js_sys::Reflect::get(&next_entry, &JsValue::from_str("value"))
            .unwrap();

        let next_entry: Array = value.try_into().expect("Failed to cast iterator's value to Array");
        let name_js = next_entry.get(0);
        let handle_js = next_entry.get(1);

        let name = name_js.as_string().expect("Failed to cast JsValue to String");

        info!("Found entry: {:?}", name);
        output.add_entry(ReadDirEntry::new("".into(), name));

        // TODO: get path, add to entry
        // TODO: handle subdirectories
    }

    Ok(FsTaskResultEnum::ReadDir(output))
}

async fn handle_create_dir(task: &CreateDirTask) -> Result<FsTaskResultEnum, TaskError> {
    let output = CreateDirResult::new();

    let root = get_root().await;

    let folder_name = task.path.clone().into_os_string().into_string().unwrap();
    let mut options = FileSystemGetDirectoryOptions::new();
    options.create(true);
    let dir_handle_promise = root.get_directory_handle_with_options(&folder_name, &options);
    let dir_handle_js = JsFuture::from(dir_handle_promise).await.expect("Error creating directory handle");
    let _dir_handle: FileSystemDirectoryHandle = dir_handle_js.try_into().expect("Failed to cast JsValue to FileSystemDirectoryHandle");

    Ok(FsTaskResultEnum::CreateDir(output))
}

// utils for tasks
async fn get_root() -> FileSystemDirectoryHandle {
    let window = web_sys::window().expect("no global `window` exists");
    let navigator = window.navigator();
    let storage_manager = navigator.storage();
    let root_promise: Promise = storage_manager.get_directory();
    let root_result = JsFuture::from(root_promise).await;
    let root_handle: FileSystemDirectoryHandle = match root_result {
        Ok(root_handle_js) => {
            // Attempt to cast JsValue to FileSystemDirectoryHandle
            let root_handle: FileSystemDirectoryHandle = root_handle_js.try_into().expect("Failed to cast JsValue to FileSystemDirectoryHandle");
            root_handle
        },
        Err(e) => {
            panic!("Error getting root directory: {:?}", e);
        },
    };
    root_handle
}