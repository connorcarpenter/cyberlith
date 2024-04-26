use serde::Serialize;

#[derive(Serialize, Debug)]
pub enum VultrInstanceType {
    OS(u32),
    ISO(String),
    Snapshot(String),
    App(String),
    Image(String),
}
