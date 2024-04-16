mod manager;

pub(crate) use manager::ClipboardManagerImpl;

////

use crate::error::TaskError;

pub(crate) struct TaskJob;

pub(crate) fn start_task() -> TaskJob {
    TaskJob
}

pub(crate) fn poll_task(_task: &mut TaskJob) -> Option<Result<String, TaskError>> {
    let mut clipboard = ClipboardManagerImpl::get_clipboard();
    match clipboard.get_text() {
        Ok(contents) => return Some(Ok(contents)),
        Err(err) => {
            logging::error!("Failed to get clipboard contents: {:?}", err);
            return Some(Err(TaskError::IoError(err.to_string())));
        }
    }
}
