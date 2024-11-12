use game_engine::world::components::{ProcessTickResult, TileMovement};

pub(crate) trait ClientTileMovement {
    fn inner_mut(&mut self) -> &mut TileMovement;
    fn process_result(&mut self, result: ProcessTickResult);
}