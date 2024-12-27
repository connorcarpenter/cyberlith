use crate::types::Direction;

#[derive(Clone)]
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

    pub fn buffered_move(&self) -> Option<Direction> {
        self.buffered_move_dir
    }

    pub fn pop_buffered_move(&mut self) -> Option<Direction> {
        self.buffered_move_dir.take()
    }

    pub fn mirror(&mut self, other: &Self) {
        self.buffered_move_dir = other.buffered_move_dir;
    }

    pub fn clear(&mut self) {
        self.buffered_move_dir = None;
    }
}
