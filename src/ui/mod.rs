use crate::{datastructure::generic::Vec2i, opengl::types::RGBAColor};
use glfw::{Action, Key, Modifiers};


/// Widgets that do user input 
pub mod input;

pub mod eventhandling;
pub mod boundingbox;
pub mod coordinate;
pub mod font;

pub mod frame;
pub mod panel;
pub mod statusbar;
pub mod view;
pub mod inputbox;
pub mod listview;

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

pub static ACTIVE_VIEW_BACKGROUND: RGBAColor = RGBAColor { r: 0.071, g: 0.102, b: 0.1242123, a: 1.0 };