use bevy_ecs::component::Component;
use ehttp::Request;

#[derive(Component, Debug, Clone)]
pub struct HttpRequest(pub(crate) Request);

impl HttpRequest {
    pub fn new(request: Request) -> Self {
        Self(request)
    }

    pub fn get(url: impl ToString) -> Self {
        Self(Request::get(url))
    }

    pub fn post(url: &str, body: Vec<u8>) -> Self {
        Self(Request::post(url, body))
    }
}