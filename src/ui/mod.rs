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
pub mod view;

pub mod clipboard;
pub mod debug_view;
pub mod scrollbar;

#[derive(Clone, Copy, Debug)]
pub enum UID {
    View(u32),
    Panel(u32),
    Overlay(u32),
    None,
}

/// Mouse state. Note that all Vec2d values *must* be translated to Application local understanding
/// of the coordinate system (which has the Y-axis reveresed from GLFW). Not translating the GLFW -> Application coordinates
/// will make the rendering etc involved with this state data, behave wrong.
#[derive(Debug, Clone, Copy)]
pub enum MouseState {
    /// Mouse state that immediately gets translated to, when a mouse click is registered
    Click(glfw::MouseButton, Vec2d),
    /// Represents the mouse state when a UI element has been clicked and when Application has verified that MouseState::Click
    /// was inside a UI Element
    UIElementClicked(ViewId, glfw::MouseButton, Vec2d),
    /// Mouse state representing a mouse drag action, involving the layout of an Element in the
    /// window. Thus, the behavior manager of this state, is the Application itself and not the individual UI element.
    UIElementDrag(ViewId, glfw::MouseButton, Vec2d),
    /// UIElementDragAction is a mouse state that represents a mouse click and drag
    /// that the UI element should register itself, and handle what decision to take.
    /// In contrast with UIElementDrag, which is a MouseState that Application<'app> should handle
    /// Since it involves how the Application lays element out in the UI, etc
    UIElementDragAction(ViewId, glfw::MouseButton, Vec2d, Vec2d),
    /// Mouse state for when/where the mouse button was released
    Released(glfw::MouseButton, Vec2d),
    None,
}

impl MouseState {
    pub fn position(&self) -> Option<Vec2i> {
        match self {
            MouseState::Click(.., pos) => Some(pos.to_i32()),
            MouseState::UIElementDrag(_, _, pos) => Some(pos.to_i32()),
            MouseState::UIElementDragAction(_, _, _, current) => Some(current.to_i32()),
            MouseState::Released(_, pos) => Some(pos.to_i32()),
            MouseState::UIElementClicked(.., pos) => Some(pos.to_i32()),
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
    /// Mouse click handler. Must take a screen coordinate that is validated to be inside this view element
    /// * `screen_coordinate` - coordinate where the mouse was clicked. Must be validated to actually be inside this view element, or cause UB
    fn mouse_clicked(&mut self, screen_coordinate: Vec2i);
    /// Mouse click handler. Must take a screen coordinate that is validated to be inside this view element
    /// * `begin_coordinate` - The begin coordinate of this mouse drag action (i.e. prior mouse position to this mouse movement)
    /// * `current_coordinate` - The current coordinate of this mouse drag action (i.e. current mouse position)
    fn mouse_dragged(&mut self, begin_coordinate: Vec2i, current_coordinated: Vec2i) -> Option<Vec2i>;
}
