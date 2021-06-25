use crate::opengl::rect::RectRenderer;
use crate::opengl::text::TextRenderer;
use crate::ui::font::Font;
use super::coordinate::{Anchor, Size};

use crate::opengl::{Renderable};
use crate::ui::coordinate::Coordinate;
use std::fmt::Formatter;

pub trait Viewable {
    fn set_anchor(&mut self, anchor: Anchor);
    fn resize(&mut self, width: i32, height: i32);
    fn update(&mut self, renderable: Box<dyn Renderable>);
}

#[derive(PartialEq, Clone, Copy, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct ViewId(pub u32);

impl std::ops::Deref for ViewId {
    type Target = u32;

    fn deref<'a>(&'a self) -> &'a u32 {
        &self.0
    }
}

impl Into<ViewId> for u32 {
    fn into(self) -> ViewId {
        ViewId(self)
    }
}

pub struct View<'a> {
    pub name: String,
    pub id: ViewId,
    pub text_renderer: TextRenderer<'a>,
    pub window_renderer: RectRenderer,
    pub buffer_id: u32,
    pub size: Size,
    pub anchor: Anchor,
    pub topmost_line_in_buffer: i32,
    displayable_lines: i32,
    row_height: i32,
    panel_id: Option<u32>,
}

pub struct Popup<'a> {
    pub visible: bool,
    pub view: View<'a>
}


 impl<'a> std::ops::Deref for Popup<'a> {
    type Target = View<'a>;
    fn deref(&self) -> &Self::Target {
        &self.view
    }
}

impl<'a> std::fmt::Debug for View<'a>{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("View")
            .field("id", &self.id)
            .field("size", &self.size)
            .field("anchor", &self.anchor)
            .field("top buffer line", &self.topmost_line_in_buffer)
            .field("displayable lines", &self.displayable_lines)
            .field("layout by", &self.panel_id)
            .finish()
    }
}

impl<'a> View<'a>{
    pub fn new(name: &str, view_id: ViewId, text_renderer: TextRenderer<'a>, window_renderer: RectRenderer, buffer_id: u32, width: i32, height: i32, row_height: i32) -> View<'a>{
        let mut v = View {
            name: name.to_string(),
            id: view_id,
            text_renderer,
            window_renderer,
            buffer_id,
            size: Size::new(width, height),
            anchor: Anchor(0, 0),
            topmost_line_in_buffer: 0,
            displayable_lines: 0,
            row_height,
            panel_id: None,
        };
        v.window_renderer.update_rectangle(v.anchor, v.size);
        v
    }

    pub fn update(&mut self) {
        self.window_renderer.update_rectangle(self.anchor, self.size);
    }

    pub fn draw(&mut self) {
        self.window_renderer.draw();
        self.text_renderer.bind();
        let Anchor(top_x, top_y) = self.anchor;
        self.text_renderer.draw_text("fee fi fo fum, MOTHERFUCKER!", top_x, top_y);
    }

    pub fn set_anchor(&mut self, anchor: Anchor) {
        self.anchor = anchor;
    }

    pub fn resize(&mut self, width: i32, height: i32) {
        self.size.width = width;
        self.size.height = height;
        self.displayable_lines = self.size.height / self.row_height;
    }

    pub fn calc_displayable_lines(&mut self, font: &Font) {
        self.row_height = font.row_height();
        self.displayable_lines = self.size.height / font.row_height();
    }
} 