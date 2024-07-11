mod manager;

pub(crate) use manager::ClipboardManagerImpl;

////

use crossbeam_channel::{bounded, Receiver};
use logging::info;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::window;

use crate::error::TaskError;

pub(crate) struct TaskJob(pub Receiver<Result<String, TaskError>>);

pub(crate) fn start_task() -> TaskJob {

    let (tx, task) = bounded(1);
    executor::spawn(async move {
            let result = task_process_async().await;
            tx.send(result).ok();
        })
        .detach();

    TaskJob(task)
}

pub(crate) fn poll_task(task: &mut TaskJob) -> Option<Result<String, TaskError>> {
    match task.0.try_recv() {
        Ok(Ok(result_enum)) => Some(Ok(result_enum)),
        Ok(Err(error)) => Some(Err(error)),
        Err(_) => None,
    }
}

pub(crate) async fn task_process_async() -> Result<String, TaskError> {
    let Some(window) = window() else {
        return Err(TaskError::IoError(
            "Failed to access the window object".to_owned(),
        ));
    };

    let nav = window.navigator();
    let Some(clipboard) = nav.clipboard() else {
        return Err(TaskError::IoError("Failed to access clipboard".to_owned()));
    };

    let promise = clipboard.read_text();
    match JsFuture::from(promise).await {
        Ok(value) => match value.as_string() {
            Some(contents) => {
                info!("read from clipboard: {}", &contents);
                return Ok(contents);
            }
            None => {
                return Err(TaskError::IoError(
                    "Failed to read from clipboard: empty value".to_owned(),
                ));
            }
        },
        Err(err) => {
            return Err(TaskError::IoError(format!(
                "Failed to read from clipboard: {}",
                string_from_js_value(&err)
            )));
        }
    }
}

pub(crate) fn string_from_js_value(value: &JsValue) -> String {
    value.as_string().unwrap_or_else(|| format!("{value:#?}"))
}
