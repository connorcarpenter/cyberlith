use log::info;

pub(crate) fn setup() {
    info!("Setting up local environment");
    automation_lib::process_assets("local").unwrap();
}