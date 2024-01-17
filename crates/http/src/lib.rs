
pub use ehttp;

use bevy_tasks::{AsyncComputeTaskPool, Task};
use bevy_ecs::{
    component::Component,
    system::{Commands, Query},
    entity::Entity,
    query::Without,
};
use bevy_app::{Plugin, App, Update};
use ehttp::{Request, Response};
use futures_lite::future;

#[cfg(target_family = "wasm")]
use crossbeam_channel::{bounded, Receiver};

#[derive(Default)]
pub struct HttpClientPlugin;

impl Plugin for HttpClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (handle_request, handle_response));
        app.add_plugins(bevy_core::TaskPoolPlugin::default());
    }
}

/// wrap for ehttp request
#[derive(Component, Debug, Clone)]
pub struct HttpRequest(pub Request);

impl HttpRequest {
    /// create a new http request
    pub fn new(request: Request) -> Self {
        Self(request)
    }

    /// create a new http get request
    pub fn get(url: impl ToString) -> Self {
        Self(Request::get(url))
    }

    /// create a new http post request
    pub fn post(url: &str, body: Vec<u8>) -> Self {
        Self(Request::post(url, body))
    }
}

/// wrap for ehttp response
#[derive(Component, Debug, Clone)]
pub struct HttpResponse(pub Response);

/// wrap for ehttp error
#[derive(Component, Debug, Clone)]
pub struct HttpResponseError(pub String);

/// task for ehttp response result
#[cfg(target_family = "wasm")]
#[derive(Component)]
pub struct RequestTask(pub Receiver<Result<Response, ehttp::Error>>);

#[cfg(not(target_family = "wasm"))]
#[derive(Component)]
pub struct RequestTask(pub Task<Result<Response, ehttp::Error>>);

fn handle_request(
    mut commands: Commands,
    requests: Query<(Entity, &HttpRequest), Without<RequestTask>>,
) {
    let thread_pool = AsyncComputeTaskPool::get();
    for (entity, request) in requests.iter() {

        let req = request.clone();

        // wasm
        #[cfg(target_family = "wasm")]
        let (tx, task) = bounded(1);
        #[cfg(target_family = "wasm")]
        thread_pool
            .spawn(async move {
                let response = ehttp::fetch_async(req.0).await;
                tx.send(response).ok();
            })
            .detach();

        // native
        #[cfg(not(target_family = "wasm"))]
        let task = thread_pool.spawn(async { ehttp::fetch_async(req.0).await });

        commands
            .entity(entity)
            .remove::<HttpRequest>()
            .insert(RequestTask(task));
    }
}

fn handle_response(
    mut commands: Commands,
    mut request_tasks: Query<(Entity, &mut RequestTask)>,
) {
    for (entity, mut task) in request_tasks.iter_mut() {

        #[cfg(target_family = "wasm")]
        if let Ok(result) = task.0.try_recv() {
            match result {
                Ok(res) => {
                    commands
                        .entity(entity)
                        .insert(HttpResponse(res))
                        .remove::<RequestTask>();
                }
                Err(e) => {
                    commands
                        .entity(entity)
                        .insert(crate::HttpResponseError(e))
                        .remove::<RequestTask>();
                }
            }
        }

        #[cfg(not(target_family = "wasm"))]
        if let Some(result) = future::block_on(future::poll_once(&mut task.0)) {
            match result {
                Ok(res) => {
                    commands
                        .entity(entity)
                        .insert(HttpResponse(res))
                        .remove::<RequestTask>();
                }
                Err(e) => {
                    commands
                        .entity(entity)
                        .insert(HttpResponseError(e))
                        .remove::<RequestTask>();
                }
            }
        }
    }
}
