/// Keyboard key input.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub enum Key {
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    ArrowUp,

    Escape,
    Tab,
    Backspace,
    Enter,
    Space,

    Insert,
    Delete,
    Home,
    End,
    PageUp,
    PageDown,

    /// Either from the main row or from the numpad.
    Num0,
    /// Either from the main row or from the numpad.
    Num1,
    /// Either from the main row or from the numpad.
    Num2,
    /// Either from the main row or from the numpad.
    Num3,
    /// Either from the main row or from the numpad.
    Num4,
    /// Either from the main row or from the numpad.
    Num5,
    /// Either from the main row or from the numpad.
    Num6,
    /// Either from the main row or from the numpad.
    Num7,
    /// Either from the main row or from the numpad.
    Num8,
    /// Either from the main row or from the numpad.
    Num9,

    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,

    Plus,
    Slash,
    Period,
    Equals,
    Asterisk,
    Minus,
    Grave,
    LBracket,
    RBracket,
    Backslash,
    Semicolon,
    Apostrophe,
    Comma,

    LShift,
    LCtrl,
    LAlt,
    RShift,
    RCtrl,
    RAlt,

    VolumeDown,
    VolumeUp,
}

impl Key {
    pub fn is_char(&self) -> bool {
        match self {
            Key::A
            | Key::B
            | Key::C
            | Key::D
            | Key::E
            | Key::F
            | Key::G
            | Key::H
            | Key::I
            | Key::J
            | Key::K
            | Key::L
            | Key::M
            | Key::N
            | Key::O
            | Key::P
            | Key::Q
            | Key::R
            | Key::S
            | Key::T
            | Key::U
            | Key::V
            | Key::W
            | Key::X
            | Key::Y
            | Key::Z
            | Key::Num0
            | Key::Num1
            | Key::Num2
            | Key::Num3
            | Key::Num4
            | Key::Num5
            | Key::Num6
            | Key::Num7
            | Key::Num8
            | Key::Num9
            | Key::Space
            | Key::Plus
            | Key::Slash
            | Key::Period
            | Key::Equals
            | Key::Asterisk
            | Key::Minus
            | Key::Grave
            | Key::LBracket
            | Key::RBracket
            | Key::Backslash
            | Key::Semicolon
            | Key::Apostrophe
            | Key::Comma => true,
            _ => false,
        }
    }

    pub fn to_char(&self, shift: bool) -> Option<char> {
        if shift {
            return Some(match self {
                Key::A => 'A',
                Key::B => 'B',
                Key::C => 'C',
                Key::D => 'D',
                Key::E => 'E',
                Key::F => 'F',
                Key::G => 'G',
                Key::H => 'H',
                Key::I => 'I',
                Key::J => 'J',
                Key::K => 'K',
                Key::L => 'L',
                Key::M => 'M',
                Key::N => 'N',
                Key::O => 'O',
                Key::P => 'P',
                Key::Q => 'Q',
                Key::R => 'R',
                Key::S => 'S',
                Key::T => 'T',
                Key::U => 'U',
                Key::V => 'V',
                Key::W => 'W',
                Key::X => 'X',
                Key::Y => 'Y',
                Key::Z => 'Z',
                Key::Num0 => ')',
                Key::Num1 => '!',
                Key::Num2 => '@',
                Key::Num3 => '#',
                Key::Num4 => '$',
                Key::Num5 => '%',
                Key::Num6 => '^',
                Key::Num7 => '&',
                Key::Num8 => '*',
                Key::Num9 => '(',
                Key::Space => ' ',
                Key::Plus => '+',
                Key::Slash => '?',
                Key::Period => '>',
                Key::Equals => '+',
                Key::Asterisk => '*',
                Key::Minus => '_',
                Key::Grave => '~',
                Key::LBracket => '{',
                Key::RBracket => '}',
                Key::Backslash => '|',
                Key::Semicolon => ':',
                Key::Apostrophe => '"',
                Key::Comma => '<',
                _ => {
                    return None;
                }
            });
        } else {
            return Some(match self {
                Key::A => 'a',
                Key::B => 'b',
                Key::C => 'c',
                Key::D => 'd',
                Key::E => 'e',
                Key::F => 'f',
                Key::G => 'g',
                Key::H => 'h',
                Key::I => 'i',
                Key::J => 'j',
                Key::K => 'k',
                Key::L => 'l',
                Key::M => 'm',
                Key::N => 'n',
                Key::O => 'o',
                Key::P => 'p',
                Key::Q => 'q',
                Key::R => 'r',
                Key::S => 's',
                Key::T => 't',
                Key::U => 'u',
                Key::V => 'v',
                Key::W => 'w',
                Key::X => 'x',
                Key::Y => 'y',
                Key::Z => 'z',
                Key::Num0 => '0',
                Key::Num1 => '1',
                Key::Num2 => '2',
                Key::Num3 => '3',
                Key::Num4 => '4',
                Key::Num5 => '5',
                Key::Num6 => '6',
                Key::Num7 => '7',
                Key::Num8 => '8',
                Key::Num9 => '9',
                Key::Space => ' ',
                Key::Plus => '+',
                Key::Slash => '/',
                Key::Period => '.',
                Key::Equals => '=',
                Key::Asterisk => '*',
                Key::Minus => '-',
                Key::Grave => '`',
                Key::LBracket => '[',
                Key::RBracket => ']',
                Key::Backslash => '\\',
                Key::Semicolon => ';',
                Key::Apostrophe => '\'',
                Key::Comma => ',',
                _ => {
                    return None;
                }
            });
        }
    }
}
