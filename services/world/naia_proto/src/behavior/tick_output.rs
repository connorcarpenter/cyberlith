use crate::types::Direction;

pub struct TickOutput {
    net_tile_target: Option<(i16, i16)>,
    net_move_buffer: Option<Option<Direction>>,
}

impl TickOutput {
    pub fn new() -> Self {
        Self {
            net_tile_target: None,
            net_move_buffer: None,
        }
    }

    pub fn set_net_tile_target(&mut self, x: i16, y: i16) {
        self.net_tile_target = Some((x, y));
    }

    pub fn set_net_move_buffer(&mut self, direction: Option<Direction>) {
        self.net_move_buffer = Some(direction);
    }

    pub fn take_outbound_net_tile_target(&mut self) -> Option<(i16, i16)> {
        self.net_tile_target.take()
    }

    pub fn take_outbound_net_move_buffer(&mut self) -> Option<Option<Direction>> {
        self.net_move_buffer.take()
    }
}
