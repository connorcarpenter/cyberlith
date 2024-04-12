use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        compile_error!("this should not be compiled for web, it is native only");
    } else {}
}