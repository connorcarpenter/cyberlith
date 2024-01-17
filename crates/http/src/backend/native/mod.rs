
use bevy_tasks::{AsyncComputeTaskPool, Task};
use bevy_ecs::{
    component::Component,
    system::{Commands, Query},
    entity::Entity,
    query::Without,
};
use ehttp::Response;
use futures_lite::future;

use crate::{HttpRequest, HttpResponse, HttpResponseError};

#[derive(Component)]
pub struct RequestTask(pub Task<Result<Response, ehttp::Error>>);

pub fn handle_request(
    mut commands: Commands,
    requests: Query<(Entity, &HttpRequest), Without<RequestTask>>,
) {
    let thread_pool = AsyncComputeTaskPool::get();
    for (entity, request) in requests.iter() {

        let req = request.clone();

        let task = thread_pool.spawn(async { ehttp::fetch_async(req.0).await });

        commands
            .entity(entity)
            .remove::<HttpRequest>()
            .insert(RequestTask(task));
    }
}

pub fn handle_response(
    mut commands: Commands,
    mut request_tasks: Query<(Entity, &mut RequestTask)>,
) {
    for (entity, mut task) in request_tasks.iter_mut() {
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
