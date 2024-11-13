use game_engine::world::components::{MoveBuffer, ProcessTickResult, TileMovement};

pub(crate) trait ClientTileMovement {
    fn decompose(&mut self) -> (&mut TileMovement, Option<&mut MoveBuffer>);
    fn process_result(&mut self, result: ProcessTickResult);
}
