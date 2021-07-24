/// Basic building blocks of UI elements
pub mod basic;
use basic::{boundingbox, coordinate, frame};

use crate::{
    datastructure::generic::{Vec2d, Vec2i},
    opengl::types::RGBAColor,
};
use glfw::{Action, Key, Modifiers};

use self::{boundingbox::BoundingBox, coordinate::Size, view::ViewId};

pub mod eventhandling;
pub mod font;

pub mod inputbox;
pub mod panel;
pub mod statusbar;
pub mod view;

pub mod debug_view;

#[derive(Clone, Copy, Debug)]
pub enum UID {
    View(u32),
    Panel(u32),
    None,
}

#[derive(Debug, Clone, Copy)]
pub enum MouseState {
    Click(glfw::MouseButton, Vec2d),
    Clicked(Option<ViewId>, glfw::MouseButton, Vec2d),
    Drag(Option<ViewId>, glfw::MouseButton, Vec2d),
    Released(glfw::MouseButton, Vec2d),
    None,
}

impl MouseState {
    pub fn position(&self) -> Option<Vec2i> {
        match self {
            MouseState::Click(.., pos) => Some(pos.to_i32()),
            MouseState::Drag(_, _, pos) => Some(pos.to_i32()),
            MouseState::Released(_, pos) => Some(pos.to_i32()),
            MouseState::Clicked(.., pos) => Some(pos.to_i32()),
            MouseState::None => None,
        }
    }
}

pub enum UIAction {
    MouseMove(Vec2i),
    MouseClick(glfw::MouseButton, Vec2i),
    MouseScroll,
    KeyPress(Key, Action, Modifiers),
    KeyRelease,
}

pub static ACTIVE_VIEW_BACKGROUND: RGBAColor = RGBAColor { r: 0.071, g: 0.102, b: 0.1242123, a: 1.0 };

pub trait Viewable {
    fn resize(&mut self, size: Size);
    fn set_anchor(&mut self, anchor: Vec2i);
    fn bounding_box(&self) -> BoundingBox;
    fn mouse_clicked(&mut self, screen_coordinate: Vec2i);
}
