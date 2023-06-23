use input::{
    Key as RenderGlowKey, Modifiers as RenderGlowModifiers, MouseButton as RenderGlowMouseButton,
};

pub fn glow_to_egui_key(key: &RenderGlowKey) -> egui::Key {
    use egui::Key;
    use input::Key::*;
    match key {
        ArrowDown => Key::ArrowDown,
        ArrowLeft => Key::ArrowLeft,
        ArrowRight => Key::ArrowRight,
        ArrowUp => Key::ArrowUp,
        Escape => Key::Escape,
        Tab => Key::Tab,
        Backspace => Key::Backspace,
        Enter => Key::Enter,
        Space => Key::Space,
        Insert => Key::Insert,
        Delete => Key::Delete,
        Home => Key::Home,
        End => Key::End,
        PageUp => Key::PageUp,
        PageDown => Key::PageDown,
        Num0 => Key::Num0,
        Num1 => Key::Num1,
        Num2 => Key::Num2,
        Num3 => Key::Num3,
        Num4 => Key::Num4,
        Num5 => Key::Num5,
        Num6 => Key::Num6,
        Num7 => Key::Num7,
        Num8 => Key::Num8,
        Num9 => Key::Num9,
        A => Key::A,
        B => Key::B,
        C => Key::C,
        D => Key::D,
        E => Key::E,
        F => Key::F,
        G => Key::G,
        H => Key::H,
        I => Key::I,
        J => Key::J,
        K => Key::K,
        L => Key::L,
        M => Key::M,
        N => Key::N,
        O => Key::O,
        P => Key::P,
        Q => Key::Q,
        R => Key::R,
        S => Key::S,
        T => Key::T,
        U => Key::U,
        V => Key::V,
        W => Key::W,
        X => Key::X,
        Y => Key::Y,
        Z => Key::Z,
    }
}

pub fn glow_to_egui_modifiers(modifiers: &RenderGlowModifiers) -> egui::Modifiers {
    egui::Modifiers {
        alt: modifiers.alt,
        ctrl: modifiers.ctrl,
        shift: modifiers.shift,
        command: modifiers.command,
        mac_cmd: false,
    }
}

pub fn glow_to_egui_mouse_button(button: &RenderGlowMouseButton) -> egui::PointerButton {
    match button {
        RenderGlowMouseButton::Left => egui::PointerButton::Primary,
        RenderGlowMouseButton::Right => egui::PointerButton::Secondary,
        RenderGlowMouseButton::Middle => egui::PointerButton::Middle,
    }
}
