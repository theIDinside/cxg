use std::fmt::Display;

use glfw::ffi as glfwffi;
use serde::Deserialize;

bitflags::bitflags! {
    #[doc = "Key modifiers (e.g., Shift, Control, Alt, Super)"]
    #[derive(Deserialize)]
    pub struct ModifiersImpl: ::std::os::raw::c_int {
        const SHIFT       = glfwffi::MOD_SHIFT;
        const CONTROL     = glfwffi::MOD_CONTROL;
        const ALT         = glfwffi::MOD_ALT;
        const SUPER       = glfwffi::MOD_SUPER;
        const CAPS_LOCK    = glfwffi::MOD_CAPS_LOCK;
        const NUM_LOCK     = glfwffi::MOD_NUM_LOCK;
    }
}

impl Display for ModifiersImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if *self == ModifiersImpl::CONTROL {
            write!(f, "ctrl")
        } else if *self == ModifiersImpl::SHIFT {
            write!(f, "shift")
        } else if *self == ModifiersImpl::ALT {
            write!(f, "alt")
        } else if *self == ModifiersImpl::SUPER {
            write!(f, "meta")
        } else if *self == ModifiersImpl::CONTROL | ModifiersImpl::ALT | ModifiersImpl::SHIFT | ModifiersImpl::SUPER {
            write!(f, "ctrl+alt+shift+meta")
        } else if *self == ModifiersImpl::CONTROL | ModifiersImpl::ALT | ModifiersImpl::SHIFT {
            write!(f, "ctrl+alt+shift")
        } else if *self == ModifiersImpl::CONTROL | ModifiersImpl::SHIFT | ModifiersImpl::SUPER {
            write!(f, "ctrl+shift+meta")
        } else if *self == ModifiersImpl::CONTROL | ModifiersImpl::ALT | ModifiersImpl::SUPER {
            write!(f, "ctrl+alt+meta")
        } else if *self == ModifiersImpl::ALT | ModifiersImpl::SHIFT | ModifiersImpl::SUPER {
            write!(f, "alt+shift+meta")
        } else if *self == ModifiersImpl::CONTROL | ModifiersImpl::ALT {
            write!(f, "ctrl+alt")
        } else if *self == ModifiersImpl::CONTROL | ModifiersImpl::SHIFT {
            write!(f, "ctrl+shift")
        } else if *self == ModifiersImpl::CONTROL | ModifiersImpl::SUPER {
            write!(f, "ctrl+meta")
        } else if *self == ModifiersImpl::ALT | ModifiersImpl::SHIFT {
            write!(f, "alt+shift")
        } else if *self == ModifiersImpl::ALT | ModifiersImpl::SUPER {
            write!(f, "alt+meta")
        } else if *self == ModifiersImpl::SHIFT | ModifiersImpl::SUPER {
            write!(f, "shift+meta")
        } else {
            Ok(())
        }
    }
}

impl std::str::FromStr for ModifiersImpl {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ctrl" => Ok(ModifiersImpl::CONTROL),
            "shift" => Ok(ModifiersImpl::SHIFT),
            "alt" => Ok(ModifiersImpl::ALT),
            "meta" => Ok(ModifiersImpl::SUPER),
            "ctrl+alt+shift+meta" => Ok(ModifiersImpl::CONTROL | ModifiersImpl::ALT | ModifiersImpl::SHIFT | ModifiersImpl::SUPER),
            "ctrl+alt+shift" => Ok(ModifiersImpl::CONTROL | ModifiersImpl::ALT | ModifiersImpl::SHIFT),
            "ctrl+shift+meta" => Ok(ModifiersImpl::CONTROL | ModifiersImpl::SHIFT | ModifiersImpl::SUPER),
            "ctrl+alt+meta" => Ok(ModifiersImpl::CONTROL | ModifiersImpl::ALT | ModifiersImpl::SUPER),
            "alt+shift+meta" => Ok(ModifiersImpl::ALT | ModifiersImpl::SHIFT | ModifiersImpl::SUPER),
            "ctrl+alt" => Ok(ModifiersImpl::CONTROL | ModifiersImpl::ALT),
            "ctrl+shift" => Ok(ModifiersImpl::CONTROL | ModifiersImpl::SHIFT),
            "ctrl+meta" => Ok(ModifiersImpl::CONTROL | ModifiersImpl::SUPER),
            "alt+shift" => Ok(ModifiersImpl::ALT | ModifiersImpl::SHIFT),
            "alt+meta" => Ok(ModifiersImpl::ALT | ModifiersImpl::SUPER),
            "shift+meta" => Ok(ModifiersImpl::SHIFT | ModifiersImpl::SUPER),
            _ => Err("could not modifiers impl"),
        }
    }
}

/// Input keys.
#[repr(i32)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum KeyImpl {
    Space = glfwffi::KEY_SPACE,
    Apostrophe = glfwffi::KEY_APOSTROPHE,
    Comma = glfwffi::KEY_COMMA,
    Minus = glfwffi::KEY_MINUS,
    Period = glfwffi::KEY_PERIOD,
    Slash = glfwffi::KEY_SLASH,
    Num0 = glfwffi::KEY_0,
    Num1 = glfwffi::KEY_1,
    Num2 = glfwffi::KEY_2,
    Num3 = glfwffi::KEY_3,
    Num4 = glfwffi::KEY_4,
    Num5 = glfwffi::KEY_5,
    Num6 = glfwffi::KEY_6,
    Num7 = glfwffi::KEY_7,
    Num8 = glfwffi::KEY_8,
    Num9 = glfwffi::KEY_9,
    Semicolon = glfwffi::KEY_SEMICOLON,
    Equal = glfwffi::KEY_EQUAL,
    A = glfwffi::KEY_A,
    B = glfwffi::KEY_B,
    C = glfwffi::KEY_C,
    D = glfwffi::KEY_D,
    E = glfwffi::KEY_E,
    F = glfwffi::KEY_F,
    G = glfwffi::KEY_G,
    H = glfwffi::KEY_H,
    I = glfwffi::KEY_I,
    J = glfwffi::KEY_J,
    K = glfwffi::KEY_K,
    L = glfwffi::KEY_L,
    M = glfwffi::KEY_M,
    N = glfwffi::KEY_N,
    O = glfwffi::KEY_O,
    P = glfwffi::KEY_P,
    Q = glfwffi::KEY_Q,
    R = glfwffi::KEY_R,
    S = glfwffi::KEY_S,
    T = glfwffi::KEY_T,
    U = glfwffi::KEY_U,
    V = glfwffi::KEY_V,
    W = glfwffi::KEY_W,
    X = glfwffi::KEY_X,
    Y = glfwffi::KEY_Y,
    Z = glfwffi::KEY_Z,
    LeftBracket = glfwffi::KEY_LEFT_BRACKET,
    Backslash = glfwffi::KEY_BACKSLASH,
    RightBracket = glfwffi::KEY_RIGHT_BRACKET,
    GraveAccent = glfwffi::KEY_GRAVE_ACCENT,
    World1 = glfwffi::KEY_WORLD_1,
    World2 = glfwffi::KEY_WORLD_2,

    Escape = glfwffi::KEY_ESCAPE,
    Enter = glfwffi::KEY_ENTER,
    Tab = glfwffi::KEY_TAB,
    Backspace = glfwffi::KEY_BACKSPACE,
    Insert = glfwffi::KEY_INSERT,
    Delete = glfwffi::KEY_DELETE,
    Right = glfwffi::KEY_RIGHT,
    Left = glfwffi::KEY_LEFT,
    Down = glfwffi::KEY_DOWN,
    Up = glfwffi::KEY_UP,
    PageUp = glfwffi::KEY_PAGE_UP,
    PageDown = glfwffi::KEY_PAGE_DOWN,
    Home = glfwffi::KEY_HOME,
    End = glfwffi::KEY_END,
    CapsLock = glfwffi::KEY_CAPS_LOCK,
    ScrollLock = glfwffi::KEY_SCROLL_LOCK,
    NumLock = glfwffi::KEY_NUM_LOCK,
    PrintScreen = glfwffi::KEY_PRINT_SCREEN,
    Pause = glfwffi::KEY_PAUSE,
    F1 = glfwffi::KEY_F1,
    F2 = glfwffi::KEY_F2,
    F3 = glfwffi::KEY_F3,
    F4 = glfwffi::KEY_F4,
    F5 = glfwffi::KEY_F5,
    F6 = glfwffi::KEY_F6,
    F7 = glfwffi::KEY_F7,
    F8 = glfwffi::KEY_F8,
    F9 = glfwffi::KEY_F9,
    F10 = glfwffi::KEY_F10,
    F11 = glfwffi::KEY_F11,
    F12 = glfwffi::KEY_F12,
    F13 = glfwffi::KEY_F13,
    F14 = glfwffi::KEY_F14,
    F15 = glfwffi::KEY_F15,
    F16 = glfwffi::KEY_F16,
    F17 = glfwffi::KEY_F17,
    F18 = glfwffi::KEY_F18,
    F19 = glfwffi::KEY_F19,
    F20 = glfwffi::KEY_F20,
    F21 = glfwffi::KEY_F21,
    F22 = glfwffi::KEY_F22,
    F23 = glfwffi::KEY_F23,
    F24 = glfwffi::KEY_F24,
    F25 = glfwffi::KEY_F25,
    Kp0 = glfwffi::KEY_KP_0,
    Kp1 = glfwffi::KEY_KP_1,
    Kp2 = glfwffi::KEY_KP_2,
    Kp3 = glfwffi::KEY_KP_3,
    Kp4 = glfwffi::KEY_KP_4,
    Kp5 = glfwffi::KEY_KP_5,
    Kp6 = glfwffi::KEY_KP_6,
    Kp7 = glfwffi::KEY_KP_7,
    Kp8 = glfwffi::KEY_KP_8,
    Kp9 = glfwffi::KEY_KP_9,
    KpDecimal = glfwffi::KEY_KP_DECIMAL,
    KpDivide = glfwffi::KEY_KP_DIVIDE,
    KpMultiply = glfwffi::KEY_KP_MULTIPLY,
    KpSubtract = glfwffi::KEY_KP_SUBTRACT,
    KpAdd = glfwffi::KEY_KP_ADD,
    KpEnter = glfwffi::KEY_KP_ENTER,
    KpEqual = glfwffi::KEY_KP_EQUAL,
    LeftShift = glfwffi::KEY_LEFT_SHIFT,
    LeftControl = glfwffi::KEY_LEFT_CONTROL,
    LeftAlt = glfwffi::KEY_LEFT_ALT,
    LeftSuper = glfwffi::KEY_LEFT_SUPER,
    RightShift = glfwffi::KEY_RIGHT_SHIFT,
    RightControl = glfwffi::KEY_RIGHT_CONTROL,
    RightAlt = glfwffi::KEY_RIGHT_ALT,
    RightSuper = glfwffi::KEY_RIGHT_SUPER,
    Menu = glfwffi::KEY_MENU,
    Unknown = glfwffi::KEY_UNKNOWN,
}

impl Display for KeyImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::str::FromStr for KeyImpl {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Space" => Ok(KeyImpl::Space),
            "Apostroph" => Ok(KeyImpl::Apostrophe),
            "Comm" => Ok(KeyImpl::Comma),
            "Minu" => Ok(KeyImpl::Minus),
            "Perio" => Ok(KeyImpl::Period),
            "Slas" => Ok(KeyImpl::Slash),
            "Num0" => Ok(KeyImpl::Num0),
            "Num1" => Ok(KeyImpl::Num1),
            "Num2" => Ok(KeyImpl::Num2),
            "Num3" => Ok(KeyImpl::Num3),
            "Num4" => Ok(KeyImpl::Num4),
            "Num5" => Ok(KeyImpl::Num5),
            "Num6" => Ok(KeyImpl::Num6),
            "Num7" => Ok(KeyImpl::Num7),
            "Num8" => Ok(KeyImpl::Num8),
            "Num9" => Ok(KeyImpl::Num9),
            "Semicolon" => Ok(KeyImpl::Semicolon),
            "Equal" => Ok(KeyImpl::Equal),
            "A" => Ok(KeyImpl::A),
            "B" => Ok(KeyImpl::B),
            "C" => Ok(KeyImpl::C),
            "D" => Ok(KeyImpl::D),
            "E" => Ok(KeyImpl::E),
            "F" => Ok(KeyImpl::F),
            "G" => Ok(KeyImpl::G),
            "H" => Ok(KeyImpl::H),
            "I" => Ok(KeyImpl::I),
            "J" => Ok(KeyImpl::J),
            "K" => Ok(KeyImpl::K),
            "L" => Ok(KeyImpl::L),
            "M" => Ok(KeyImpl::M),
            "N" => Ok(KeyImpl::N),
            "O" => Ok(KeyImpl::O),
            "P" => Ok(KeyImpl::P),
            "Q" => Ok(KeyImpl::Q),
            "R" => Ok(KeyImpl::R),
            "S" => Ok(KeyImpl::S),
            "T" => Ok(KeyImpl::T),
            "U" => Ok(KeyImpl::U),
            "V" => Ok(KeyImpl::V),
            "W" => Ok(KeyImpl::W),
            "X" => Ok(KeyImpl::X),
            "Y" => Ok(KeyImpl::Y),
            "Z" => Ok(KeyImpl::Z),
            "LeftBracket" => Ok(KeyImpl::LeftBracket),
            "Backslash" => Ok(KeyImpl::Backslash),
            "RightBracket" => Ok(KeyImpl::RightBracket),
            "GraveAccent" => Ok(KeyImpl::GraveAccent),
            "World1" => Ok(KeyImpl::World1),
            "World2" => Ok(KeyImpl::World2),
            "Escape" => Ok(KeyImpl::Escape),
            "Enter" => Ok(KeyImpl::Enter),
            "Tab" => Ok(KeyImpl::Tab),
            "Backspace" => Ok(KeyImpl::Backspace),
            "Insert" => Ok(KeyImpl::Insert),
            "Delete" => Ok(KeyImpl::Delete),
            "Right" => Ok(KeyImpl::Right),
            "Left" => Ok(KeyImpl::Left),
            "Down" => Ok(KeyImpl::Down),
            "Up" => Ok(KeyImpl::Up),
            "PageUp" => Ok(KeyImpl::PageUp),
            "PageDown" => Ok(KeyImpl::PageDown),
            "Home" => Ok(KeyImpl::Home),
            "End" => Ok(KeyImpl::End),
            "CapsLock" => Ok(KeyImpl::CapsLock),
            "ScrollLock" => Ok(KeyImpl::ScrollLock),
            "NumLock" => Ok(KeyImpl::NumLock),
            "PrintScreen" => Ok(KeyImpl::PrintScreen),
            "Pause" => Ok(KeyImpl::Pause),
            "F1" => Ok(KeyImpl::F1),
            "F2" => Ok(KeyImpl::F2),
            "F3" => Ok(KeyImpl::F3),
            "F4" => Ok(KeyImpl::F4),
            "F5" => Ok(KeyImpl::F5),
            "F6" => Ok(KeyImpl::F6),
            "F7" => Ok(KeyImpl::F7),
            "F8" => Ok(KeyImpl::F8),
            "F9" => Ok(KeyImpl::F9),
            "F10" => Ok(KeyImpl::F10),
            "F11" => Ok(KeyImpl::F11),
            "F12" => Ok(KeyImpl::F12),
            "F13" => Ok(KeyImpl::F13),
            "F14" => Ok(KeyImpl::F14),
            "F15" => Ok(KeyImpl::F15),
            "F16" => Ok(KeyImpl::F16),
            "F17" => Ok(KeyImpl::F17),
            "F18" => Ok(KeyImpl::F18),
            "F19" => Ok(KeyImpl::F19),
            "F20" => Ok(KeyImpl::F20),
            "F21" => Ok(KeyImpl::F21),
            "F22" => Ok(KeyImpl::F22),
            "F23" => Ok(KeyImpl::F23),
            "F24" => Ok(KeyImpl::F24),
            "F25" => Ok(KeyImpl::F25),
            "Kp0" => Ok(KeyImpl::Kp0),
            "Kp1" => Ok(KeyImpl::Kp1),
            "Kp2" => Ok(KeyImpl::Kp2),
            "Kp3" => Ok(KeyImpl::Kp3),
            "Kp4" => Ok(KeyImpl::Kp4),
            "Kp5" => Ok(KeyImpl::Kp5),
            "Kp6" => Ok(KeyImpl::Kp6),
            "Kp7" => Ok(KeyImpl::Kp7),
            "Kp8" => Ok(KeyImpl::Kp8),
            "Kp9" => Ok(KeyImpl::Kp9),
            "KpDecimal" => Ok(KeyImpl::KpDecimal),
            "KpDivide" => Ok(KeyImpl::KpDivide),
            "KpMultiply" => Ok(KeyImpl::KpMultiply),
            "KpSubtract" => Ok(KeyImpl::KpSubtract),
            "KpAdd" => Ok(KeyImpl::KpAdd),
            "KpEnter" => Ok(KeyImpl::KpEnter),
            "KpEqual" => Ok(KeyImpl::KpEqual),
            "LeftShift" => Ok(KeyImpl::LeftShift),
            "LeftControl" => Ok(KeyImpl::LeftControl),
            "LeftAlt" => Ok(KeyImpl::LeftAlt),
            "LeftSuper" => Ok(KeyImpl::LeftSuper),
            "RightShift" => Ok(KeyImpl::RightShift),
            "RightControl" => Ok(KeyImpl::RightControl),
            "RightAlt" => Ok(KeyImpl::RightAlt),
            "RightSuper" => Ok(KeyImpl::RightSuper),
            "Menu" => Ok(KeyImpl::Menu),
            "Unknown" => Ok(KeyImpl::Unknown),
            _ => Err("could not do fromstr for keyimpl"),
        }
    }
}
