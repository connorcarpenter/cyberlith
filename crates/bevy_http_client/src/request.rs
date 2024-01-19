use bevy_ecs::component::Component;
use ehttp::Request;

#[derive(Component, Debug, Clone)]
pub struct HttpRequest(pub(crate) Request);

impl HttpRequest {
    pub fn new(request: Request) -> Self {
        Self(request)
    }

    pub fn get(url: &str) -> Self {
        Self(Request::get(url))
    }

    pub fn post(url: &str, body: Box<[u8]>) -> Self {
        Self(Request::post(url, body.to_vec()))
    }
}