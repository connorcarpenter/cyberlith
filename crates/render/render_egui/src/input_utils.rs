use input::{
    Key as RenderglKey, Modifiers as RenderglModifiers, MouseButton as RenderglMouseButton,
};

pub fn gl_to_egui_key(key: &RenderglKey) -> Option<egui::Key> {
    use egui::Key;
    use input::Key::*;
    match key {
        ArrowDown => Some(Key::ArrowDown),
        ArrowLeft => Some(Key::ArrowLeft),
        ArrowRight => Some(Key::ArrowRight),
        ArrowUp => Some(Key::ArrowUp),
        Escape => Some(Key::Escape),
        Tab => Some(Key::Tab),
        Backspace => Some(Key::Backspace),
        Enter => Some(Key::Enter),
        Space => Some(Key::Space),
        Insert => Some(Key::Insert),
        Delete => Some(Key::Delete),
        Home => Some(Key::Home),
        End => Some(Key::End),
        PageUp => Some(Key::PageUp),
        PageDown => Some(Key::PageDown),
        Num0 => Some(Key::Num0),
        Num1 => Some(Key::Num1),
        Num2 => Some(Key::Num2),
        Num3 => Some(Key::Num3),
        Num4 => Some(Key::Num4),
        Num5 => Some(Key::Num5),
        Num6 => Some(Key::Num6),
        Num7 => Some(Key::Num7),
        Num8 => Some(Key::Num8),
        Num9 => Some(Key::Num9),
        A => Some(Key::A),
        B => Some(Key::B),
        C => Some(Key::C),
        D => Some(Key::D),
        E => Some(Key::E),
        F => Some(Key::F),
        G => Some(Key::G),
        H => Some(Key::H),
        I => Some(Key::I),
        J => Some(Key::J),
        K => Some(Key::K),
        L => Some(Key::L),
        M => Some(Key::M),
        N => Some(Key::N),
        O => Some(Key::O),
        P => Some(Key::P),
        Q => Some(Key::Q),
        R => Some(Key::R),
        S => Some(Key::S),
        T => Some(Key::T),
        U => Some(Key::U),
        V => Some(Key::V),
        W => Some(Key::W),
        X => Some(Key::X),
        Y => Some(Key::Y),
        Z => Some(Key::Z),
        _ => None,
    }
}

pub fn gl_to_egui_modifiers(modifiers: &RenderglModifiers) -> egui::Modifiers {
    egui::Modifiers {
        alt: modifiers.alt,
        ctrl: modifiers.ctrl,
        shift: modifiers.shift,
        command: modifiers.command,
        mac_cmd: false,
    }
}

pub fn gl_to_egui_mouse_button(button: &RenderglMouseButton) -> egui::PointerButton {
    match button {
        RenderglMouseButton::Left => egui::PointerButton::Primary,
        RenderglMouseButton::Right => egui::PointerButton::Secondary,
        RenderglMouseButton::Middle => egui::PointerButton::Middle,
    }
}
