use crate::types::Direction;

pub struct TickOutput {
    next_tile_position: Option<(i16, i16)>,
    next_move_buffer: Option<Option<Direction>>,
}

impl TickOutput {
    pub fn new() -> Self {
        Self {
            next_tile_position: None,
            next_move_buffer: None,
        }
    }

    pub fn set_next_tile_position(&mut self, x: i16, y: i16) {
        self.next_tile_position = Some((x, y));
    }

    pub fn set_next_move_buffer(&mut self, direction: Option<Direction>) {
        self.next_move_buffer = Some(direction);
    }

    pub fn take_outbound_next_tile_position(&mut self) -> Option<(i16, i16)> {
        self.next_tile_position.take()
    }

    pub fn take_outbound_next_move_buffer(&mut self) -> Option<Option<Direction>> {
        self.next_move_buffer.take()
    }
}