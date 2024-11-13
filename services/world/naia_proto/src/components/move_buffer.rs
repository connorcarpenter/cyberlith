use crate::types::Direction;

pub struct MoveBuffer {
    buffered_move_dir: Option<Direction>,
}

impl MoveBuffer {
    pub fn new() -> Self {
        Self {
            buffered_move_dir: None,
        }
    }

    pub fn has_buffered_move(&self) -> bool {
        self.buffered_move_dir.is_some()
    }

    pub fn buffer_move(&mut self, move_dir: Direction) {
        self.buffered_move_dir = Some(move_dir);
    }

    pub fn pop_buffered_move(&mut self) -> Option<Direction> {
        self.buffered_move_dir.take()
    }
}