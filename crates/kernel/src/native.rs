
static mut EXIT_ACTION_CONTAINER: Option<String> = None;
pub struct ExitActionContainer;
impl ExitActionContainer {
    pub fn is_set() -> bool {
        unsafe { EXIT_ACTION_CONTAINER.is_some() }
    }
    pub fn set(action: String) {
        unsafe {
            if EXIT_ACTION_CONTAINER.is_some() {
                panic!("ExitActionContainer already set");
            }
            EXIT_ACTION_CONTAINER = Some(action);
        }
    }
    pub fn take() -> String {
        unsafe {
            let Some(output) = EXIT_ACTION_CONTAINER.take() else {
                panic!("ExitActionContainer not set");
            };
            output
        }
    }
}