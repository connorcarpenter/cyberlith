use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct VultrApiError {
    pub error: String,
}
