use glfw::{Key, Action, Modifiers};
use crate::datastructure::generic::Vec2i;

pub mod boundingbox;
pub mod coordinate;
pub mod panel;
pub mod view;
pub mod statusbar;
pub mod font;

#[derive(Clone, Copy, Debug)]
pub enum UID {
    View(u32),
    Panel(u32),
}

pub enum MouseButton {
    Left,
    Right,
    Middle,
    Custom(i32)
}


pub enum UIAction {
    MouseMove(Vec2i),
    MouseClick(MouseButton, Vec2i),
    MouseScroll,
    KeyPress(Key, Action, Modifiers),
    KeyRelease,
}