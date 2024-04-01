use bevy_ecs::system::{NonSendMut, ResMut};
use winit::window::CursorIcon as WinitCursorIcon;

use clipboard::ClipboardManager;
use input::{CursorIcon, Input};

use crate::window::{FrameInput, OutgoingEvent};

pub fn run(
    frame_input: NonSendMut<FrameInput>,
    mut input: ResMut<Input>,
    mut clipboard: ResMut<ClipboardManager>,
) {
    input.recv_events(&mut clipboard, &frame_input.incoming_events);
}

pub fn update_cursor(mut frame_input: NonSendMut<FrameInput>, mut input: ResMut<Input>) {
    if let Some(cursor_icon) = input.take_cursor_icon() {
        let winit_cursor_icon = match cursor_icon {
            CursorIcon::Default => WinitCursorIcon::Default,
            CursorIcon::Crosshair => WinitCursorIcon::Crosshair,
            CursorIcon::Hand => WinitCursorIcon::Hand,
            CursorIcon::Arrow => WinitCursorIcon::Arrow,
            CursorIcon::Move => WinitCursorIcon::Move,
            CursorIcon::Text => WinitCursorIcon::Text,
            CursorIcon::Wait => WinitCursorIcon::Wait,
            CursorIcon::Help => WinitCursorIcon::Help,
            CursorIcon::Progress => WinitCursorIcon::Progress,
            CursorIcon::NotAllowed => WinitCursorIcon::NotAllowed,
            CursorIcon::ContextMenu => WinitCursorIcon::ContextMenu,
            CursorIcon::Cell => WinitCursorIcon::Cell,
            CursorIcon::VerticalText => WinitCursorIcon::VerticalText,
            CursorIcon::Alias => WinitCursorIcon::Alias,
            CursorIcon::Copy => WinitCursorIcon::Copy,
            CursorIcon::NoDrop => WinitCursorIcon::NoDrop,
            CursorIcon::Grab => WinitCursorIcon::Grab,
            CursorIcon::Grabbing => WinitCursorIcon::Grabbing,
            CursorIcon::AllScroll => WinitCursorIcon::AllScroll,
            CursorIcon::ZoomIn => WinitCursorIcon::ZoomIn,
            CursorIcon::ZoomOut => WinitCursorIcon::ZoomOut,
            CursorIcon::EResize => WinitCursorIcon::EResize,
            CursorIcon::NResize => WinitCursorIcon::NResize,
            CursorIcon::NeResize => WinitCursorIcon::NeResize,
            CursorIcon::NwResize => WinitCursorIcon::NwResize,
            CursorIcon::SResize => WinitCursorIcon::SResize,
            CursorIcon::SeResize => WinitCursorIcon::SeResize,
            CursorIcon::SwResize => WinitCursorIcon::SwResize,
            CursorIcon::WResize => WinitCursorIcon::WResize,
            CursorIcon::EwResize => WinitCursorIcon::EwResize,
            CursorIcon::NsResize => WinitCursorIcon::NsResize,
            CursorIcon::NeswResize => WinitCursorIcon::NeswResize,
            CursorIcon::NwseResize => WinitCursorIcon::NwseResize,
            CursorIcon::ColResize => WinitCursorIcon::ColResize,
            CursorIcon::RowResize => WinitCursorIcon::RowResize,
        };
        frame_input
            .outgoing_events
            .push(OutgoingEvent::CursorChanged(winit_cursor_icon));
    }
}
