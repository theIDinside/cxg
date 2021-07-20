use crate::datastructure::generic::Vec2i;
use glfw::{Action, Key, Modifiers};

pub mod input;
pub mod boundingbox;
pub mod coordinate;
pub mod font;
pub mod panel;
pub mod statusbar;
pub mod view;
pub mod inputbox;

pub mod debug_view;

#[derive(Clone, Copy, Debug)]
pub enum UID {
    View(u32),
    Panel(u32),
    None,
}

pub enum MouseButton {
    Left,
    Right,
    Middle,
    Custom(i32),
}
pub enum UIAction {
    MouseMove(Vec2i),
    MouseClick(MouseButton, Vec2i),
    MouseScroll,
    KeyPress(Key, Action, Modifiers),
    KeyRelease,
}
