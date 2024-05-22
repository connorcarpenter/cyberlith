use bevy_ecs::{event::EventReader, system::{Res, ResMut}, schedule::{NextState, State}};

use game_engine::{
    asset::{
        AssetLoadedEvent, AssetType
    },
    logging::info,
    ui::UiManager,
};

use crate::{ui::{on_ui_load, UiCatalog}, states::AppState};

pub fn session_load_asset_events(
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut ui_manager: ResMut<UiManager>,
    mut ui_catalog: ResMut<UiCatalog>,
    mut event_reader: EventReader<AssetLoadedEvent>,
) {
    for event in event_reader.read() {
        let asset_id = event.asset_id;
        let asset_type = event.asset_type;
        info!("received Asset Loaded Event! (asset_id: {:?})", asset_id);
        if asset_type == AssetType::Ui {
            let state = *state.get();
            on_ui_load(state, &mut next_state, &mut ui_manager, &mut ui_catalog, asset_id);
        }
    }
}
