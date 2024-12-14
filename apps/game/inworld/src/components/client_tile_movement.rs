use game_engine::world::components::{MoveBuffer, TileMovement};

pub(crate) trait ClientTileMovement {
    fn decompose(&mut self) -> (&mut TileMovement, &mut MoveBuffer);
}
