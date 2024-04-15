mod manager;
pub(crate) use manager::ClipboardManagerImpl;

////

use crossbeam_channel::{Receiver};

use crate::error::TaskError;

pub(crate) struct TaskJob(pub Receiver<Result<Option<String>, TaskError>>);