// use bevy_ecs::prelude::Component;
//
// use game_engine::{
//     logging::{info, warn},
//     naia::{sequence_greater_than, wrapping_diff, Tick},
//     world::components::Position,
// };
//
// #[derive(Component)]
// pub struct Interp {
//     predicted: bool,
//     last_tick: Tick,
//     last_x: f32,
//     last_y: f32,
//     next_tick: Tick,
//     next_x: f32,
//     next_y: f32,
// }
//
// impl Interp {
//     pub fn new(position: &Position) -> Self {
//         let predicted = position.predicted();
//         let tick = position.tick();
//         let x = position.x();
//         let y = position.y();
//         Self {
//             predicted,
//
//             last_tick: tick,
//             last_x: x,
//             last_y: y,
//
//             next_tick: tick,
//             next_x: x,
//             next_y: y,
//         }
//     }
//
//     pub(crate) fn next_position(&mut self, position: &Position, debug_opt: Option<&str>) {
//         if position.predicted() != self.predicted {
//             panic!("Interp.predicted != Position.predicted");
//         }
//
//         // if !self.predicted {
//         //     if let Some(debug_str) = debug_opt {
//         //         info!("Interp.next_position() - {:?}, (predicted: {:?})", debug_str, self.predicted);
//         //     }
//         // }
//
//         self.last_tick = self.next_tick;
//         self.last_x = self.next_x;
//         self.last_y = self.next_y;
//
//         self.next_tick = position.tick();
//         self.next_x = position.x();
//         self.next_y = position.y();
//
//         // if !self.predicted {
//         //     if self.last_tick == self.next_tick {
//         //         warn!("Interp.next_position: last_tick == next_tick");
//         //     }
//         //
//         //     info!("interp from tick {:?} -> tick {:?}", self.last_tick, self.next_tick);
//         //
//         //     if sequence_greater_than(self.last_tick, self.next_tick) {
//         //         warn!("Interp.next_position: last_tick > next_tick");
//         //     }
//         //     let tick_diff = wrapping_diff(self.last_tick, self.next_tick);
//         //     if tick_diff != 1 {
//         //         warn!("Interp.next_position: tick_diff: {:?}", tick_diff);
//         //     }
//         // }
//     }
//
//     pub(crate) fn interpolate(&self, interpolation: f32) -> (f32, f32) {
//         let x = self.last_x + ((self.next_x - self.last_x) * interpolation);
//         let y = self.last_y + ((self.next_y - self.last_y) * interpolation);
//
//         // let x = self.next_x;
//         // let y = self.next_y;
//
//         return (x, y);
//     }
//
//     pub fn mirror(&mut self, other: &Self) {
//         if !self.predicted {
//             info!("mirroring");
//         }
//         self.last_tick = other.last_tick;
//         self.last_x = other.last_x;
//         self.last_y = other.last_y;
//
//         self.next_tick = other.next_tick;
//         self.next_x = other.next_x;
//         self.next_y = other.next_y;
//     }
// }
