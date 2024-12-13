static mut EXIT_ACTION_CONTAINER: Option<String> = None;
pub struct ExitActionContainer;
impl ExitActionContainer {
    pub fn is_set() -> bool {
        unsafe {
            #[allow(static_mut_refs)]
            EXIT_ACTION_CONTAINER.is_some()
        }
    }
    pub fn set(action: String) {
        unsafe {
            #[allow(static_mut_refs)]
            if EXIT_ACTION_CONTAINER.is_some() {
                panic!("ExitActionContainer already set");
            }
            EXIT_ACTION_CONTAINER = Some(action);
        }
    }
    pub fn take() -> String {
        unsafe {
            #[allow(static_mut_refs)]
            let Some(output) = EXIT_ACTION_CONTAINER.take() else {
                panic!("ExitActionContainer not set");
            };
            output
        }
    }
}

pub fn get_querystring_param(_name: &str) -> Option<String> {
    return None;
}
