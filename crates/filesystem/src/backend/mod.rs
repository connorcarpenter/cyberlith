use crate::error::TaskError;
use crate::tasks::task_enum::{FsTaskEnum, FsTaskResultEnum};
cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        mod wasm;
        pub(crate) use self::wasm::*;
    }
    else {
        mod native;
        pub(crate) use self::native::*;
    }
}

pub(crate) async fn task_process_async(task_enum: FsTaskEnum) -> Result<FsTaskResultEnum, TaskError> {
    #[cfg(not(target_arch = "wasm32"))]
    return native::task_process_async(task_enum).await;

    #[cfg(target_arch = "wasm32")]
    return wasm::task_process_async(&task_enum).await;
}