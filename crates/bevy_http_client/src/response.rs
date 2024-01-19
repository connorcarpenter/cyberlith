use bevy_ecs::component::Component;
use ehttp::Response;

#[derive(Component, Debug, Clone)]
pub struct HttpResponse(pub(crate) Response);

impl HttpResponse {

    pub fn text(&self) -> Option<&str> {
        std::str::from_utf8(&self.0.bytes).ok()
    }

    pub fn url(&self) -> &str {
        &self.0.url
    }

    pub fn ok(&self) -> bool {
        self.0.ok
    }

    pub fn status(&self) -> u16 {
        self.0.status
    }

    pub fn status_text(&self) -> &str {
        &self.0.status_text
    }

    pub fn headers(&self) -> &std::collections::BTreeMap<String, String> {
        &self.0.headers
    }

    pub fn bytes(&self) -> &Vec<u8> {
        &self.0.bytes
    }
}


#[derive(Component, Debug, Clone)]
pub struct HttpResponseError(pub(crate) String);

impl HttpResponseError {

    pub fn error_message(&self) -> &str {
        &self.0
    }
}