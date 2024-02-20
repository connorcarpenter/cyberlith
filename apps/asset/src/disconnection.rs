use log::info;

use http_server::{async_dup::Arc, smol::lock::RwLock};

use crate::state::State;

pub async fn handle(state: Arc<RwLock<State>>) {
    let mut state = state.write().await;

    if state.connected() {
        if state.time_to_disconnect() {
            info!("disconnecting from region server");
            state.set_disconnected();
        }
    }
}
