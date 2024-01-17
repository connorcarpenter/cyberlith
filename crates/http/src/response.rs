use bevy_ecs::component::Component;
use ehttp::Response;

#[derive(Component, Debug, Clone)]
pub struct HttpResponse(pub Response);

#[derive(Component, Debug, Clone)]
pub struct HttpResponseError(pub String);