
use bevy_tasks::AsyncComputeTaskPool;
use bevy_ecs::{
    component::Component,
    system::{Commands, Query},
    entity::Entity,
    query::Without,
};
use ehttp::Response;
use crossbeam_channel::{bounded, Receiver};

use crate::{HttpRequest, HttpResponse};

#[derive(Component)]
pub struct RequestTask(pub Receiver<Result<Response, ehttp::Error>>);

pub fn handle_request(
    mut commands: Commands,
    requests: Query<(Entity, &HttpRequest), Without<RequestTask>>,
) {
    let thread_pool = AsyncComputeTaskPool::get();
    for (entity, request) in requests.iter() {

        let req = request.clone();

        let (tx, task) = bounded(1);
        thread_pool
            .spawn(async move {
                let response = ehttp::fetch_async(req.0).await;
                tx.send(response).ok();
            })
            .detach();

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
    for (entity, task) in request_tasks.iter_mut() {

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
    }
}
